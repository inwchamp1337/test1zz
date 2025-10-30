use spider::url::Url;
use spider::website::Website;
use serde::Deserialize;
use std::fs;
use crate::crawler::domain_detector::FetchMode;

/// โหลด `robots.txt` จาก base_url และคืน Vec<String> ของ sitemap URLs
pub async fn get_sitemaps_from_robots(
    base_url: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let parsed = Url::parse(base_url)?;
    let robots_url = parsed.join("/robots.txt")?.to_string();

    println!("- กำลังโหลด: {}", robots_url);

    let mut website = Website::new(&robots_url);
    website.with_user_agent(Some("MyRustCrawler/1.0".into()));
    website.with_depth(0);

    // scrape() คืนค่า () เมื่อสำเร็จ
    website.scrape().await;

    let pages = website
        .get_pages()
        .ok_or("ไม่มีหน้าที่ถูกดาวน์โหลดจากเว็บไซต์")?;

    let page = pages.first().ok_or("ไม่พบหน้าในเวกเตอร์ pages")?;
    let content = page.get_html();

    let sitemaps: Vec<String> = content
        .lines()
        .filter_map(|line| {
            let t = line.trim();
            if t.to_lowercase().starts_with("sitemap:") {
                t.get(8..).map(|u| u.trim().to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(sitemaps)
}

/// ลองดึง sitemap.xml โดยตรงจาก https://<host>/sitemap.xml
/// คืน Vec<String> ของ URL ที่เจอภายใน <loc> tags (และพิมพ์ออกมาทันทีเมื่อเจอ)
pub async fn fetch_sitemap_direct(base_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let parsed = Url::parse(base_url)?;
    let sitemap_url = parsed.join("/sitemap.xml")?.to_string();

    println!("- ลองโหลด sitemap ตรง ๆ: {}", sitemap_url);

    let mut website = Website::new(&sitemap_url);
    website.with_user_agent(Some("MyRustCrawler/1.0".into()));
    website.with_depth(0);

    website.scrape().await;

    let pages = website.get_pages();
    if pages.is_none() || pages.as_ref().unwrap().is_empty() {
        // ไม่พบหรือดึงไม่ได้
        return Ok(Vec::new());
    }

    let page = pages.unwrap().first().ok_or("ไม่พบหน้าในเวกเตอร์ pages")?;
    let content = page.get_html();

    // หา <loc> ... </loc> แบบไม่ขึ้นกับ case
    let mut sitemap_urls = Vec::new();
    let content_lower = content.to_lowercase();
    let mut pos = 0usize;
    while let Some(start_rel) = content_lower[pos..].find("<loc") {
        let start = pos + start_rel;
        // หาตำแหน่ง '>' ของแท็กเปิด
        if let Some(gt_rel) = content_lower[start..].find('>') {
            let content_start = start + gt_rel + 1;
            // หาตัวปิด </loc>
            if let Some(end_rel) = content_lower[content_start..].find("</loc>") {
                let content_end = content_start + end_rel;
                let url_text = content[content_start..content_end].trim().to_string();
                if !url_text.is_empty() {
                    println!("-> พบ URL ใน sitemap.xml: {}", url_text);
                    sitemap_urls.push(url_text);
                }
                pos = content_end + 6; // ข้าม "</loc>"
                continue;
            }
        }
        break;
    }

    Ok(sitemap_urls)
}

/// Config สำหรับการ crawl แบบ native โดย spider
#[derive(Debug, Deserialize)]
struct SpiderConfig {
    depth: Option<usize>,
    user_agent: Option<String>,
    delay_ms: Option<u64>,
    max_pages: Option<usize>,
}

impl Default for SpiderConfig {
    fn default() -> Self {
        Self {
            depth: Some(3),
            user_agent: Some("MyRustCrawler/1.0".into()),
            delay_ms: Some(200),
            max_pages: Some(100),
        }
    }
}

/// ถ้าไม่มี robots.txt และไม่มี sitemap.xml -> ใช้ spider native crawl
/// จะอ่าน config จาก "config/app.yaml" (ถ้าไฟล์มี) แล้วเริ่ม crawl จากหน้าแรกของ base_url
pub async fn crawl_with_spider(base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // โหลด config จากไฟล์ ถ้ามี
    let cfg: SpiderConfig = match fs::read_to_string("config/app.yaml") {
        Ok(s) => match serde_yaml::from_str(&s) {
            Ok(parsed) => parsed,
            Err(e) => {
                eprintln!("-> ไม่สามารถ parse config/app.yaml: {}  -> ใช้ค่า default", e);
                SpiderConfig::default()
            }
        },
        Err(_) => {
            println!("-> ไม่พบ config/app.yaml, ใช้ค่า default");
            SpiderConfig::default()
        }
    };

    println!("- เริ่ม native spider crawl ที่: {}", base_url);
    println!("- config: depth={:?}, user_agent={:?}, delay_ms={:?}, max_pages={:?}",
        cfg.depth, cfg.user_agent, cfg.delay_ms, cfg.max_pages);

    let mut website = Website::new(base_url);
    website.with_user_agent(cfg.user_agent.as_deref());
    if let Some(d) = cfg.depth {
        website.with_depth(d);
    }
    // ตั้งค่า delay ถ้า config มี (ใช้ u64 โดยตรง)
    if let Some(ms) = cfg.delay_ms {
        website.configuration.delay = ms;
    }

    website.scrape().await;

    // แสดง URL ที่ถูกดาวน์โหลด
    if let Some(pages) = website.get_pages() {
        for page in pages {
            let url = page.get_url();
            println!("-> visited: {}", url);
        }
    } else {
        println!("-> spider ไม่ได้ดาวน์โหลดหน้าใด ๆ");
    }

    Ok(())
}

/// Fetch HTML from URLs with specified fetch mode
/// Returns Vec<(String, String)> of (url, html_content)
pub async fn fetch_html_from_urls(
    urls: Vec<String>,
    fetch_mode: FetchMode,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let total_urls = urls.len();

    let mode_str = match fetch_mode {
        FetchMode::HttpRequest => "HttpRequest",
        FetchMode::Chrome => "Chrome Browser",
    };
    
    println!("- เริ่มโหลด HTML จาก {} URL(s) โดยใช้ {}", total_urls, mode_str);

    for url in urls {
        println!("  -> กำลังโหลด: {}", url);

        // Configure website based on fetch mode
        let mut website = Website::new(&url);
        website.with_user_agent(Some("MyRustCrawler/1.0"));
        website.with_depth(0);  // โหลดเฉพาะหน้าเดียว

        match fetch_mode {
            FetchMode::HttpRequest => {
                // Default HTTP request mode - no additional configuration needed
            }
            FetchMode::Chrome => {
                // Enable Chrome browser mode for JavaScript rendering
                println!("    -> Chrome mode requested for: {}", url);
                // Note: Chrome configuration will be implemented when the API is available
                // For now, this serves as a placeholder for Chrome-specific configuration
            }
        }

        website.scrape().await;

        // ดึง HTML content
        if let Some(pages) = website.get_pages() {
            if let Some(page) = pages.first() {
                let html = page.get_html();
                println!("  -> โหลดสำเร็จ: {} ({} bytes)", url, html.len());
                results.push((url.clone(), html.to_string()));
            } else {
                eprintln!("  -> ไม่พบหน้าสำหรับ URL: {}", url);
            }
        } else {
            eprintln!("  -> ไม่สามารถโหลดหน้าได้: {}", url);
        }
    }

    println!("- โหลด HTML เสร็จสิ้น: {}/{} URL(s)", results.len(), total_urls);
    Ok(results)
}

/// Fetch HTML from URLs using domain detector for automatic mode selection
/// Returns Vec<(String, String)> of (url, html_content)
pub async fn fetch_html_from_urls_with_detection(
    urls: Vec<String>,
    domain_detector: &crate::crawler::domain_detector::DomainDetector,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let total_urls = urls.len();

    println!("- เริ่มโหลด HTML จาก {} URL(s) โดยใช้ auto-detection", total_urls);

    for url in urls {
        // Extract domain from URL for mode detection
        let parsed_url = Url::parse(&url)?;
        let domain = parsed_url.host_str().unwrap_or("");
        let fetch_mode = domain_detector.get_fetch_mode(domain);
        
        let mode_str = match fetch_mode {
            FetchMode::HttpRequest => "HttpRequest",
            FetchMode::Chrome => "Chrome Browser",
        };
        
        println!("  -> กำลังโหลด: {} (mode: {})", url, mode_str);

        // Configure website based on detected fetch mode
        let mut website = Website::new(&url);
        website.with_user_agent(Some("MyRustCrawler/1.0"));
        website.with_depth(0);  // โหลดเฉพาะหน้าเดียว

        match fetch_mode {
            FetchMode::HttpRequest => {
                // Default HTTP request mode - no additional configuration needed
            }
            FetchMode::Chrome => {
                // Enable Chrome browser mode for JavaScript rendering
                println!("    -> Chrome mode requested for: {}", url);
                // Note: Chrome configuration will be implemented when the API is available
                // For now, this serves as a placeholder for Chrome-specific configuration
            }
        }

        website.scrape().await;

        // ดึง HTML content
        if let Some(pages) = website.get_pages() {
            if let Some(page) = pages.first() {
                let html = page.get_html();
                println!("  -> โหลดสำเร็จ: {} ({} bytes)", url, html.len());
                results.push((url.clone(), html.to_string()));
            } else {
                eprintln!("  -> ไม่พบหน้าสำหรับ URL: {}", url);
            }
        } else {
            eprintln!("  -> ไม่สามารถโหลดหน้าได้: {}", url);
        }
    }

    println!("- โหลด HTML เสร็จสิ้น: {}/{} URL(s)", results.len(), total_urls);
    Ok(results)
}