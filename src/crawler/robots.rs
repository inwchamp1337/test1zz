use spider::url::Url;
use spider::website::Website;
use serde::Deserialize;
use std::fs;
use std::collections::HashSet;

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

/// โหลด sitemap แบบ recursive - รองรับ sitemap index (nested)
/// ใช้ config จาก AppConfig (user_agent, delay_ms)
/// - ถ้า <loc> ชี้ไปที่ .xml -> โหลดต่อแบบ recursive
/// - ถ้า <loc> เป็น URL ปกติ -> เก็บไว้
/// คืนค่า Vec<String> ของ URL ทั้งหมด (ไม่ซ้ำ)
pub async fn fetch_sitemap_recursive(
    sitemap_url: &str,
    user_agent: &str,
    delay_ms: u64,
    visited: &mut HashSet<String>,
    depth: usize,
    max_depth: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // ป้องกัน infinite loop และ depth เกิน
    if visited.contains(sitemap_url) || depth > max_depth {
        return Ok(Vec::new());
    }
    visited.insert(sitemap_url.to_string());

    println!("[sitemap][depth={}] กำลังโหลด: {}", depth, sitemap_url);

    let mut website = Website::new(sitemap_url);
    website.with_user_agent(Some(user_agent.into()));
    website.with_depth(0);
    website.configuration.delay = delay_ms;

    website.scrape().await;

    let pages = website.get_pages();
    if pages.is_none() || pages.as_ref().unwrap().is_empty() {
        println!("[sitemap][depth={}] ไม่พบหน้าที่ดาวน์โหลดได้", depth);
        return Ok(Vec::new());
    }

    let page = pages.unwrap().first().ok_or("ไม่พบหน้าในเวกเตอร์ pages")?;
    let content = page.get_html();

    // หา <loc> ... </loc>
    let mut sitemap_urls = Vec::new();
    let mut page_urls = Vec::new();
    
    let content_lower = content.to_lowercase();
    let mut pos = 0usize;
    
    while let Some(start_rel) = content_lower[pos..].find("<loc") {
        let start = pos + start_rel;
        if let Some(gt_rel) = content_lower[start..].find('>') {
            let content_start = start + gt_rel + 1;
            if let Some(end_rel) = content_lower[content_start..].find("</loc>") {
                let content_end = content_start + end_rel;
                let url_text = content[content_start..content_end].trim().to_string();
                
                if !url_text.is_empty() {
                    // ตรวจสอบว่าเป็น sitemap (.xml) หรือ URL ปกติ
                    if url_text.ends_with(".xml") || url_text.contains(".xml?") {
                        println!("[sitemap][depth={}] -> พบ sitemap nested: {}", depth, url_text);
                        sitemap_urls.push(url_text);
                    } else {
                        println!("[sitemap][depth={}] -> พบ URL: {}", depth, url_text);
                        page_urls.push(url_text);
                    }
                }
                pos = content_end + 6;
                continue;
            }
        }
        break;
    }

    // Recursive: โหลด sitemap ที่ซ้อนกัน (ใช้ Box::pin เพื่อหลีกเลี่ยง infinite size)
    for nested_sitemap in sitemap_urls {
        let result = Box::pin(fetch_sitemap_recursive(
            &nested_sitemap,
            user_agent,
            delay_ms,
            visited,
            depth + 1,
            max_depth,
        )).await;
        
        match result {
            Ok(mut nested_urls) => {
                page_urls.append(&mut nested_urls);
            }
            Err(e) => {
                eprintln!("[sitemap][depth={}] ไม่สามารถโหลด {} ได้: {:?}", depth, nested_sitemap, e);
            }
        }
    }

    Ok(page_urls)
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

    // แสดง URL ที่ถูกดาวน์โหลด และแปลง + บันทึกเป็น Markdown
    if let Some(pages) = website.get_pages() {
        println!("[log] pages downloaded count: {}", pages.len());
        
        // Process each page: convert HTML -> Markdown -> save file
        let total = pages.len() as f64;
        for (idx, page) in pages.iter().enumerate() {
            let current = idx + 1;
            let percent = if total > 0.0 { (current as f64 / total) * 100.0 } else { 0.0 };
            let url = page.get_url().to_string();
            let html = page.get_html();
            
            println!("\n[{}/{}] ({:.1}%) Processing: {}", current, pages.len(), percent, url);
            println!("-> visited: {} ({} bytes HTML)", url, html.len());
            
            // Convert to markdown
            let markdown = super::html_to_markdown::html_to_markdown(&url, &html);
            
            // Save immediately
            match super::markdown_writer::write_markdown_file(&url, &markdown) {
                Ok(path) => println!("✓ บันทึกแล้ว: {} — {:.1}%", path.display(), percent),
                Err(err) => eprintln!("✗ บันทึกไม่สำเร็จ {}: {:?} — {:.1}%", url, err, percent),
            }
        }
    } else {
        println!("-> spider ไม่ได้ดาวน์โหลดหน้าใด ๆ");
    }

    Ok(())
}


