use crate::crawler::chrome_fetcher;

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
    let mode_label = match mode {
        FetchMode::Chrome => "SPA (Chrome/JavaScript)",
        FetchMode::HttpRequest => "SSR (HttpRequest)",
    };
    println!("[html_fetcher] fetch_html_from_urls mode={:?} [{}] total_urls={}", mode, mode_label, urls.len());

    match mode {
        FetchMode::Chrome => {
            // ใช้ chrome_fetcher สำหรับโหมด Chrome
            println!("[html_fetcher] ⚡ SPA Mode - using fetch_with_chrome function");
            chrome_fetcher::fetch_with_chrome(urls, user_agent, delay_ms).await
        }
        FetchMode::HttpRequest => {
            // ใช้ HttpRequest แบบเดิมสำหรับโหมด SSR
            println!("[html_fetcher] 📄 SSR Mode - using basic HTTP fetch (no JavaScript)");
            fetch_with_http_request(urls, user_agent, delay_ms).await
        }
    }
}

/// โหลด HTML โดยใช้ HttpRequest (สำหรับ SSR)
async fn fetch_with_http_request(
    urls: Vec<String>,
    user_agent: &str,
    delay_ms: u64,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    use std::time::Duration;
    use spider::website::Website;
    use spider::compact_str::CompactString;
    use tokio::time::sleep;

    let mut results = Vec::new();

    for url in urls {
        println!("[html_fetcher] start -> {}", url);

        let mut website = Website::new(&url);

        // ตั้ง user-agent (Box<CompactString> ตามที่ spider ต้องการ)
        website.configuration.user_agent = Some(Box::new(CompactString::new(user_agent)));

        // โหลดแค่หน้านั้น ๆ
        website.with_depth(0);

        // ตั้ง delay ถ้ามี (spider configuration)
        website.configuration.delay = delay_ms;

        // Log internal configuration for visibility
        println!(
            "[html_fetcher] config -> user_agent={:?}, delay_ms={}, depth={}",
            website.configuration.user_agent.as_ref().map(|b| b.as_ref()),
            website.configuration.delay,
            website.configuration.depth
        );

        // เรียก scrape / crawl (spider API) — ใช้ await
        let t0 = std::time::Instant::now();
        println!("[html_fetcher] scrape start: {}", url);
        website.scrape().await;
        let took = t0.elapsed();
        println!("[html_fetcher] scrape done: {} (took {:?})", url, took);

        // พิมพ์ข้อมูล pages ที่ได้ (debug)
        if let Some(pages) = website.get_pages() {
            println!("[html_fetcher] pages returned: {}", pages.len());
            for (i, page) in pages.iter().enumerate() {
                println!("  [page {}] url={} (html_len={})", i + 1, page.get_url(), page.get_html().len());
            }
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