use spider::url::Url;
use spider::website::Website;

/// ฟังก์ชันหลักสำหรับเริ่มต้นกระบวนการ Crawler
async fn run_crawler(domain_url: &str) {
    println!("เริ่มต้น Crawler สำหรับ: {}", domain_url);

    match get_sitemaps_from_robots(domain_url).await {
        Ok(sitemaps) => {
            if sitemaps.is_empty() {
                println!("-> ไม่พบ Sitemap URL ใน robots.txt");
            } else {
                println!("-> พบ {} Sitemap URL(s):", sitemaps.len());
                for sitemap in &sitemaps {
                    println!("   - {}", sitemap);
                }
            }
        }
        Err(e) => {
            eprintln!("-> ไม่สามารถโหลดหรือประมวลผล robots.txt ได้: {:?}", e);
        }
    }
}

/// โหลด `robots.txt` จากโดเมนที่กำหนดและดึง sitemap URLs ทั้งหมด
async fn get_sitemaps_from_robots(
    base_url: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let parsed_url = Url::parse(base_url)?;
    let robots_url = parsed_url.join("/robots.txt")?.to_string();

    println!("- กำลังโหลด: {}", robots_url);

    // สร้าง Website และตั้งค่าแบบง่าย ๆ ผ่านเมธอดบน Website
    let mut website = Website::new(&robots_url);
    website.with_user_agent(Some("MyRustCrawler/1.0".into()));
    website.with_depth(0);

    // เรียก scrape() — คืนค่า () เมื่อสำเร็จ
    website.scrape().await;

    // ดึงหน้าที่โหลดมา (get_pages() ให้ Option<&Vec<Page>>)
    let pages = website
        .get_pages()
        .ok_or("ไม่มีหน้าที่ถูกดาวน์โหลดจากเว็บไซต์")?;

    let page = pages.first().ok_or("ไม่พบหน้าในเวกเตอร์ pages")?;
    let content = page.get_html();

    // ดึงบรรทัดที่ขึ้นต้นด้วย "Sitemap:" (case-insensitive)
    let sitemaps: Vec<String> = content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.to_lowercase().starts_with("sitemap:") {
                trimmed.get(8..).map(|u| u.trim().to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(sitemaps)
}

#[tokio::main]
async fn main() {
    run_crawler("https://www.heygoody.com/").await;
    println!("\n----------------------------------------\n");
    run_crawler("https://www.rust-lang.org/").await;
}
