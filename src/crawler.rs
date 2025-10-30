/// Orchestration: เรียกขั้นตอนต่างๆ ของ crawler
pub async fn run_crawler(domain: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("เริ่มต้น Crawler สำหรับ: {}", domain);

    match crate::robots::get_sitemaps_from_robots(domain).await {
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
            return Err(e);
        }
    }

    Ok(())
}