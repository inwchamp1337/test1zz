use std::time::Duration;
use spider::website::Website;
use spider::compact_str::CompactString;
use tokio::time::sleep;

/// โหมดการโหลด HTML
#[derive(Debug, Clone, Copy)]
pub enum FetchMode {
    HttpRequest,
    Chrome,
}

impl FetchMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Chrome" => FetchMode::Chrome,
            _ => FetchMode::HttpRequest,
        }
    }
}

/// โหลด HTML จาก URLs โดยเลือกระหว่าง HttpRequest หรือ Chrome (spider / spider_chrome)
/// - urls: รายการ URL ที่จะโหลด
/// - mode: FetchMode::HttpRequest หรือ FetchMode::Chrome
/// - user_agent: user agent string
/// - delay_ms: delay ระหว่างการโหลดแต่ละ URL
pub async fn fetch_html_from_urls(
    urls: Vec<String>,
    mode: FetchMode,
    user_agent: &str,
    delay_ms: u64,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    println!("[html_fetcher] fetch_html_from_urls mode={:?} total_urls={}", mode, urls.len());

    for url in urls {
        println!("[html_fetcher] start -> {}", url);

        let mut website = Website::new(&url);

        // ตั้ง user-agent (Box<CompactString> ตามที่ spider ต้องการ)
        website.configuration.user_agent = Some(Box::new(CompactString::new(user_agent)));

        // โหลดแค่หน้านั้น ๆ
        website.with_depth(0);

        // ตั้ง delay ถ้ามี (spider configuration)
        website.configuration.delay = delay_ms;

        // Enable Chrome mode if selected (uses headless Chrome for JS rendering)
        if let FetchMode::Chrome = mode {
            // Use the default request intercept configuration from the library
            // to avoid requiring RequestInterceptConfiguration to be in scope.
            website.with_chrome_intercept(Default::default());
            println!("[html_fetcher] Chrome mode enabled - using headless Chrome for JS rendering");
        } else {
            println!("[html_fetcher] HttpRequest mode - using basic HTTP fetch");
        }

        // เรียก scrape / crawl (spider API) — ใช้ await
        website.scrape().await;

        // พิมพ์ข้อมูล pages ที่ได้ (debug)
        if let Some(pages) = website.get_pages() {
            println!("[html_fetcher] pages returned: {}", pages.len());
            if let Some(page) = pages.first() {
                let html = page.get_html().to_string();
                println!("[html_fetcher] fetched {} bytes from {}", html.len(), url);
                results.push((url.clone(), html));
            } else {
                eprintln!("[html_fetcher] no page for url: {}", url);
            }
        } else {
            eprintln!("[html_fetcher] get_pages returned None for url: {}", url);
        }

        // delay ระหว่าง requests
        if delay_ms > 0 {
            sleep(Duration::from_millis(delay_ms)).await;
        }
    }

    println!("[html_fetcher] finished, got {} pages", results.len());
    Ok(results)
}