/// Orchestration: เรียกขั้นตอนต่างๆ ของ crawler
pub async fn run_crawler(domain: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("เริ่มต้น Crawler สำหรับ: {}", domain);

    let mut sitemap_urls = Vec::new();

    // พยายามดึงจาก robots.txt ก่อน
    match super::robots::get_sitemaps_from_robots(domain).await {
        Ok(sitemaps) => {
            if sitemaps.is_empty() {
                println!("-> ไม่พบ Sitemap URL ใน robots.txt");
                // ถ้า robots.txt คืนค่าว่าง ให้ลองโหลด sitemap.xml ตรง ๆ
                match super::robots::fetch_sitemap_direct(domain).await {
                    Ok(direct_sitemaps) => {
                        if direct_sitemaps.is_empty() {
                            println!("-> ไม่พบ sitemap.xml ที่ {}/sitemap.xml", domain);
                            // ทั้ง robots และ sitemap ไม่พบ -> ใช้ spider native crawl
                            super::robots::crawl_with_spider(domain).await?;
                            return Ok(());
                        } else {
                            println!("-> พบ {} Sitemap URL(s) จาก sitemap.xml ตรง ๆ:", direct_sitemaps.len());
                            for sitemap in &direct_sitemaps {
                                println!("   - {}", sitemap);
                            }
                            sitemap_urls.extend(direct_sitemaps);  // เก็บ URLs
                        }
                    }
                    Err(e2) => {
                        eprintln!("-> ไม่สามารถโหลด sitemap.xml ได้: {:?}", e2);
                        return Err(e2);
                    }
                }
            } else {
                println!("-> พบ {} Sitemap URL(s) จาก robots.txt:", sitemaps.len());
                for sitemap in &sitemaps {
                    println!("   - {}", sitemap);
                }
                if sitemaps.len() == 5 {
                    println!("-> พบ 5 Sitemap URL จาก: robots.txt");
                }
                sitemap_urls.extend(sitemaps);  // เก็บ URLs
            }
        }
        Err(e) => {
            // ถ้าโหลด robots.txt ไม่ได้ ให้ลองโหลด sitemap.xml ตรง ๆ แทน
            println!(
                "-> ไม่สามารถโหลดหรือประมวลผล robots.txt ได้: {:?}\n   -> ลองโหลด sitemap.xml ตรง ๆ แทน...",
                e
            );
            match super::robots::fetch_sitemap_direct(domain).await {
                Ok(sitemaps) => {
                    if sitemaps.is_empty() {
                        println!("-> ไม่พบ sitemap.xml ที่ {}/sitemap.xml", domain);
                        // ทั้ง robots และ sitemap ไม่พบ -> ใช้ spider native crawl
                        super::robots::crawl_with_spider(domain).await?;
                        return Ok(());
                    } else {
                        println!("-> พบ {} Sitemap URL(s) จาก sitemap.xml ตรง ๆ:", sitemaps.len());
                        for sitemap in &sitemaps {
                            println!("   - {}", sitemap);
                        }
                        if sitemaps.len() == 5 {
                            println!("-> พบ 5 Sitemap URL จาก: sitemap.xml ตรง ๆ");
                        }
                        sitemap_urls.extend(sitemaps);  // เก็บ URLs
                    }
                }
                Err(e2) => {
                    eprintln!("-> ไม่สามารถโหลด sitemap.xml ได้: {:?}", e2);
                    return Err(e2);
                }
            }
        }
    }

    // ถ้าเจอ sitemap URLs ให้โหลด HTML
    if !sitemap_urls.is_empty() {
        println!("\n--- เริ่มโหลด HTML จาก sitemap URLs ---");
        let html_results = super::robots::fetch_html_from_urls(sitemap_urls).await?;
        
        for (url, html) in html_results {
            println!("✓ ดาวน์โหลดแล้ว: {} ({} bytes)", url, html.len());
            // TODO: บันทึก HTML หรือประมวลผลต่อ
            // เช่น: fs::write(format!("output/{}.html", sanitize_filename(&url)), html)?;
        }
    }

    Ok(())
}