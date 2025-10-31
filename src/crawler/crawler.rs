use super::domain_detector::DomainDetector;
use super::html_fetcher::{fetch_html_from_urls, FetchMode};
use super::html_to_markdown::html_to_markdown;
use super::markdown_writer::write_markdown_file;
use super::robots::{crawl_with_spider, get_sitemaps_from_robots, fetch_sitemap_recursive};
use std::collections::HashSet;

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
    let sitemap_max_depth = cfg.sitemap_max_depth.unwrap_or(5);
    let max_sitemap_urls = cfg.max_sitemap_urls.unwrap_or(100);

    // load whitelist detector (if available)
    let mut detector = DomainDetector::from_file(cfg.whitelist_path.as_deref().unwrap_or("src/config/whitelist.yaml"))
        .unwrap_or_else(|_| {
            println!("[domain_detector] no whitelist found, using empty detector");
            DomainDetector::default()
        });

    // determine fetch mode automatically based on domain whitelist
    let chosen_mode = detector.get_fetch_mode_for_domain(domain);
    let mode_name = match chosen_mode {
        FetchMode::Chrome => "SPA (Chrome/JavaScript)",
        FetchMode::HttpRequest => "SSR (HttpRequest)",
    };
    println!(
        "[domain_detector] domain={} -> chosen fetch mode={:?} [{}]",
        domain, chosen_mode, mode_name
    );

    // gather sitemap URLs
    let mut sitemap_urls: Vec<String> = Vec::new();

    match get_sitemaps_from_robots(domain).await {
        Ok(sitemaps) => {
            println!("[log] get_sitemaps_from_robots returned {} entry(ies)", sitemaps.len());
            if sitemaps.is_empty() {
                println!("-> ไม่พบ Sitemap URL ใน robots.txt");
                
                // ลอง /sitemap.xml ตรง ๆ แบบ recursive
                let parsed = spider::url::Url::parse(domain)?;
                let sitemap_url = parsed.join("/sitemap.xml")?.to_string();
                let mut visited = HashSet::new();
                
                match fetch_sitemap_recursive(&sitemap_url, &user_agent, delay_ms, &mut visited, 0, sitemap_max_depth).await {
                    Ok(recursive_sitemaps) => {
                        println!(
                            "[log] fetch_sitemap_recursive returned {} entry(ies)",
                            recursive_sitemaps.len()
                        );
                        if recursive_sitemaps.is_empty() {
                            let pretty = domain.trim_end_matches('/');
                            println!("-> ไม่พบ sitemap.xml ที่ {}/sitemap.xml", pretty);
                            // fallback to native spider crawl
                            crawl_with_spider(domain).await?;
                            return Ok(());
                        } else {
                            println!(
                                "-> พบ {} URL(s) จาก sitemap recursive:",
                                recursive_sitemaps.len()
                            );
                            sitemap_urls.extend(recursive_sitemaps);
                        }
                    }
                    Err(e) => {
                        eprintln!("-> ไม่สามารถโหลด sitemap.xml ได้: {:?}", e);
                        return Err(e);
                    }
                }
            } else {
                println!("-> พบ {} Sitemap URL(s) จาก robots.txt:", sitemaps.len());
                
                // โหลดแต่ละ sitemap แบบ recursive
                let mut visited = HashSet::new();
                for sitemap_url in sitemaps {
                    println!("   - กำลังโหลด sitemap: {}", sitemap_url);
                    match fetch_sitemap_recursive(&sitemap_url, &user_agent, delay_ms, &mut visited, 0, sitemap_max_depth).await {
                        Ok(urls) => {
                            println!("     -> พบ {} URL(s)", urls.len());
                            sitemap_urls.extend(urls);
                        }
                        Err(e) => {
                            eprintln!("     -> ไม่สามารถโหลด {} ได้: {:?}", sitemap_url, e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!(
                "[log] get_sitemaps_from_robots returned error: {:?}\n   -> ลองโหลด sitemap.xml ตรง ๆ แทน...",
                e
            );
            
            // ลอง /sitemap.xml ตรง ๆ แบบ recursive
            let parsed = spider::url::Url::parse(domain)?;
            let sitemap_url = parsed.join("/sitemap.xml")?.to_string();
            let mut visited = HashSet::new();
            
            match fetch_sitemap_recursive(&sitemap_url, &user_agent, delay_ms, &mut visited, 0, sitemap_max_depth).await {
                Ok(sitemaps) => {
                    println!("[log] fetch_sitemap_recursive returned {} entry(ies)", sitemaps.len());
                    if sitemaps.is_empty() {
                        let pretty = domain.trim_end_matches('/');
                        println!("-> ไม่พบ sitemap.xml ที่ {}/sitemap.xml", pretty);
                        crawl_with_spider(domain).await?;
                        return Ok(());
                    } else {
                        println!("-> พบ {} URL(s) จาก sitemap recursive:", sitemaps.len());
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
        // Apply URL limit from config
        if sitemap_urls.len() > max_sitemap_urls {
            println!("-> จำกัดจำนวน URL จาก {} เป็น {} URLs (ตาม config max_sitemap_urls)", 
                sitemap_urls.len(), max_sitemap_urls);
            sitemap_urls.truncate(max_sitemap_urls);
        }

        let mode_str = match chosen_mode {
            FetchMode::Chrome => "SPA (Chrome/JavaScript)",
            FetchMode::HttpRequest => "SSR (HttpRequest)",
        };
        println!(
            "\n--- เริ่มโหลด HTML จาก {} sitemap URLs (mode: {}) ---",
            sitemap_urls.len(),
            mode_str
        );
        
        // Process URLs one by one: download -> convert -> save immediately
        let total = sitemap_urls.len() as f64;
        for (idx, url) in sitemap_urls.iter().enumerate() {
            let current = idx + 1;
            let percent = if total > 0.0 { (current as f64 / total) * 100.0 } else { 0.0 };
            println!("\n[{}/{}] ({:.1}%) กำลังดาวน์โหลด: {}", current, sitemap_urls.len(), percent, url);

            // Fetch single URL
            let html_results = fetch_html_from_urls(vec![url.clone()], chosen_mode, &user_agent, delay_ms).await?;

            // Process result immediately
            if let Some((fetched_url, html)) = html_results.into_iter().next() {
                println!("✓ ดาวน์โหลดแล้ว: {} ({} bytes) — {:.1}%", fetched_url, html.len(), percent);

                // Convert to markdown
                let markdown = html_to_markdown(&fetched_url, &html);

                // Save immediately
                match write_markdown_file(&fetched_url, &markdown) {
                    Ok(path) => println!("✓ บันทึกแล้ว: {} — {:.1}%", path.display(), percent),
                    Err(err) => eprintln!("✗ บันทึกไม่สำเร็จ {}: {:?} — {:.1}%", fetched_url, err, percent),
                }
            } else {
                eprintln!("✗ ดาวน์โหลดไม่สำเร็จ: {} — {:.1}%", url, percent);
            }
        }
    }

    Ok(())
}