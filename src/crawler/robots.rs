use spider::url::Url;
use spider::website::Website;
use serde::Deserialize;
use std::fs;

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
    native_download_mode: Option<String>, // <- NEW
}

impl Default for SpiderConfig {
    fn default() -> Self {
        Self {
            depth: Some(3),
            user_agent: Some("MyRustCrawler/1.0".into()),
            delay_ms: Some(250),
            max_pages: Some(200),
            native_download_mode: Some("HttpRequest".into()), // <- NEW
        }
    }
}

/// ถ้าไม่มี robots.txt และไม่มี sitemap.xml -> ใช้ spider native crawl
/// จะอ่าน config จาก "config/app.yaml" (ถ้าไฟล์มี) แล้วเริ่ม crawl จากหน้าแรกของ base_url
pub async fn crawl_with_spider(base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // โหลด config จากไฟล์ ถ้ามี
    let cfg: SpiderConfig = match fs::read_to_string("src/config/app.yaml") {
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
    println!("- config: depth={:?}, user_agent={:?}, delay_ms={:?}, max_pages={:?}, fetch_mode={:?}",
        cfg.depth, cfg.user_agent, cfg.delay_ms, cfg.max_pages, cfg.native_download_mode);

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
        println!("[log] pages downloaded count: {}", pages.len());
        let urls: Vec<String> = pages.iter().map(|page| page.get_url().to_string()).collect();
        let mode_str = cfg.native_download_mode.clone().unwrap_or_else(|| "HttpRequest".into());
        let delay = cfg.delay_ms.unwrap_or(0);
        let ua = cfg.user_agent.clone().unwrap_or_else(|| "MyRustCrawler/1.0".into());

        if !urls.is_empty() {
            match super::html_fetcher::FetchMode::from_str(&mode_str) {
                super::html_fetcher::FetchMode::HttpRequest => {
                    let _ = super::html_fetcher::fetch_html_from_urls(urls.clone(), super::html_fetcher::FetchMode::HttpRequest, &ua, delay).await?;
                }
                super::html_fetcher::FetchMode::Chrome => {
                    let _ = crate::crawler::chrome_fetcher::fetch_with_chrome(urls.clone(), &ua, delay).await?;
                }
            }
        }

        for page in pages {
            println!("-> visited: {}", page.get_url());
        }
    } else {
        println!("-> spider ไม่ได้ดาวน์โหลดหน้าใด ๆ");
    }

    Ok(())
}


