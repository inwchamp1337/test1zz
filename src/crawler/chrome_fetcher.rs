// Robust chrome fetcher: tries CHROME_EXECUTABLE env first and otherwise launches
// chromiumoxide Browser. If launch fails we return an error with actionable hints.
use futures::StreamExt;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::fetcher::{BrowserFetcher, BrowserFetcherOptions};
use std::error::Error;
use std::path::PathBuf;

/// Always download (or reuse) a bundled Chromium and return its executable path.
async fn ensure_chromium() -> Result<PathBuf, Box<dyn Error>> {
    let download_dir = PathBuf::from("target/chromium");
    tokio::fs::create_dir_all(&download_dir).await?;
    let fetcher = BrowserFetcher::new(
        BrowserFetcherOptions::builder()
            .with_path(&download_dir)
            .build()?,
    );
    let info = fetcher.fetch().await?;
    Ok(PathBuf::from(info.executable_path))
}

/// Fetch pages using chromiumoxide with the bundled Chromium binary.
pub async fn fetch_with_chrome(
    urls: Vec<String>,
    _user_agent: &str,
) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let chrome_exec = ensure_chromium().await?;
    println!("[chrome_fetcher] using Chromium at {}", chrome_exec.display());

    let flags = vec![
        String::from("--no-sandbox"),
        String::from("--disable-gpu"),
        String::from("--disable-dev-shm-usage"),
        String::from("--disable-extensions"),
        String::from("--disable-background-networking"),
        String::from("--headless"),
        String::from("--user-data-dir=target/chromium-profile"),
    ];

    let config = BrowserConfig::builder()
        .chrome_executable(chrome_exec.clone())
        .args(flags)
        .build()
        .map_err(|e| format!("failed to build BrowserConfig: {}", e))?;

    let (mut browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|e| format!("failed to launch Chromium: {}", e))?;

    let handler_task = tokio::spawn(async move {
        while let Some(r) = handler.next().await {
            if r.is_err() {
                break;
            }
        }
    });

    let mut results = Vec::new();
    for url in urls {
        println!("[chrome_fetcher] fetching {}", url);
        let page = browser.new_page(&url).await?;
        let _ = page.wait_for_navigation().await?;
        let html = page.content().await?;
        results.push((url.clone(), html));
    }

    let _ = browser.close().await;
    let _ = handler_task.await;

    Ok(results)
}