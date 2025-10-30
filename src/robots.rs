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