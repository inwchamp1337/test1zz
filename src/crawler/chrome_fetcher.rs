use spider::compact_str::CompactString;
use spider::features::chrome_common::RequestInterceptConfiguration;
use spider::website::Website;
use std::error::Error;
use tokio::time::{sleep, Duration};

pub async fn fetch_with_chrome(
    urls: Vec<String>,
    user_agent: &str,
    delay_ms: u64,
) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let mut results = Vec::new();

    for url in urls {
        println!("[chrome_fetcher] fetching {}", url);

        let mut website = Website::new(&url);
        website.configuration.user_agent = Some(Box::new(CompactString::from(user_agent)));
        website.with_depth(0);
        website.with_chrome_intercept(RequestInterceptConfiguration::default());

        website.scrape().await;

        if let Some(page) = website.get_pages().and_then(|p| p.first()) {
            results.push((url.clone(), page.get_html().to_string()));
        }

        if delay_ms > 0 {
            sleep(Duration::from_millis(delay_ms)).await;
        }
    }

    Ok(results)
}