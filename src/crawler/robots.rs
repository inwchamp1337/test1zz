use spider::url::Url;
use spider::website::Website;

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