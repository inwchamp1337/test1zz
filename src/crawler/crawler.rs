use super::domain_detector::DomainDetector;
use super::html_fetcher::{fetch_html_from_urls};
use super::robots::{crawl_with_spider, fetch_sitemap_direct, get_sitemaps_from_robots};

// use centralized config loader
use crate::config::config::load_app_config;



/// Orchestration: เรียกขั้นตอนต่างๆ ของ crawler
pub async fn run_crawler(domain: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("เริ่มต้น Crawler สำหรับ: {}", domain);
    println!("[log] run_crawler() - checking robots and sitemap for: {}", domain);

    // load app config (centralized)
    let cfg = load_app_config();
    let user_agent = cfg.user_agent.clone().unwrap_or_else(|| "MyRustCrawler/1.0".into());
    let delay_ms = cfg.delay_ms.unwrap_or(250);

    // load whitelist detector (if available)
    let mut detector = DomainDetector::from_file(cfg.whitelist_path.as_deref().unwrap_or("src/config/whitelist.yaml"))
        .unwrap_or_else(|_| {
            println!("[domain_detector] no whitelist found, using empty detector");
            DomainDetector::default()
        });

    // determine fetch mode automatically based on domain whitelist
    let chosen_mode = detector.get_fetch_mode_for_domain(domain);
    println!(
        "[domain_detector] domain={} -> chosen fetch mode={:?}",
        domain, chosen_mode
    );

    // gather sitemap URLs
    let mut sitemap_urls: Vec<String> = Vec::new();

    match get_sitemaps_from_robots(domain).await {
        Ok(sitemaps) => {
            println!("[log] get_sitemaps_from_robots returned {} entry(ies)", sitemaps.len());
            if sitemaps.is_empty() {
                println!("-> ไม่พบ Sitemap URL ใน robots.txt");
                match fetch_sitemap_direct(domain).await {
                    Ok(direct_sitemaps) => {
                        println!(
                            "[log] fetch_sitemap_direct returned {} entry(ies)",
                            direct_sitemaps.len()
                        );
                        if direct_sitemaps.is_empty() {
                            let pretty = domain.trim_end_matches('/');
                            println!("-> ไม่พบ sitemap.xml ที่ {}/sitemap.xml", pretty);
                            // fallback to native spider crawl
                            crawl_with_spider(domain).await?;
                            return Ok(());
                        } else {
                            println!(
                                "-> พบ {} Sitemap URL(s) จาก sitemap.xml ตรง ๆ:",
                                direct_sitemaps.len()
                            );
                            for s in &direct_sitemaps {
                                println!("   - {}", s);
                            }
                            sitemap_urls.extend(direct_sitemaps);
                        }
                    }
                    Err(e) => {
                        eprintln!("-> ไม่สามารถโหลด sitemap.xml ได้: {:?}", e);
                        return Err(e);
                    }
                }
            } else {
                println!("-> พบ {} Sitemap URL(s) จาก robots.txt:", sitemaps.len());
                for s in &sitemaps {
                    println!("   - {}", s);
                }
                sitemap_urls.extend(sitemaps);
            }
        }
        Err(e) => {
            println!(
                "[log] get_sitemaps_from_robots returned error: {:?}\n   -> ลองโหลด sitemap.xml ตรง ๆ แทน...",
                e
            );
            match fetch_sitemap_direct(domain).await {
                Ok(sitemaps) => {
                    println!("[log] fetch_sitemap_direct returned {} entry(ies)", sitemaps.len());
                    if sitemaps.is_empty() {
                        let pretty = domain.trim_end_matches('/');
                        println!("-> ไม่พบ sitemap.xml ที่ {}/sitemap.xml", pretty);
                        crawl_with_spider(domain).await?;
                        return Ok(());
                    } else {
                        println!("-> พบ {} Sitemap URL(s) จาก sitemap.xml ตรง ๆ:", sitemaps.len());
                        for s in &sitemaps {
                            println!("   - {}", s);
                        }
                        sitemap_urls.extend(sitemaps);
                    }
                }
                Err(e2) => {
                    eprintln!("-> ไม่สามารถโหลด sitemap.xml ได้: {:?}", e2);
                    return Err(e2);
                }
            }
        }
    }

    // If we have sitemap URLs -> fetch HTML using chosen fetch mode
    if !sitemap_urls.is_empty() {
        println!("-> URLs to fetch ({}):", sitemap_urls.len());
        for u in &sitemap_urls {
            println!("   - {}", u);
        }

        println!(
            "\n--- เริ่มโหลด HTML จาก {} sitemap URLs (mode: {:?}) ---",
            sitemap_urls.len(),
            chosen_mode
        );

        let html_results =
            fetch_html_from_urls(sitemap_urls, chosen_mode, &user_agent, delay_ms).await?;

        for (url, html) in html_results {
            println!("✓ ดาวน์โหลดแล้ว: {} ({} bytes)", url, html.len());
            // TODO: save or process html
        }
    }

    Ok(())
}