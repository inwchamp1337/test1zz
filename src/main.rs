mod crawler;

use std::env;
use log::info;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    let domain = args.get(1)
        .cloned()
        .unwrap_or_else(|| "https://www.rust-lang.org/".to_string());
    
    let config_path = args.get(2).map(|s| s.as_str());

    // Note: Logging will be initialized by the crawler based on configuration
    println!("Starting web crawler for domain: {}", domain);
    if let Some(path) = config_path {
        println!("Using configuration file: {}", path);
    } else {
        println!("Using default configuration file: config/crawler.yaml");
    }

    // Run the integrated crawler pipeline with configuration
    match crate::crawler::crawler::run_crawler_with_config(&domain, config_path).await {
        Ok(stats) => {
            info!("Crawling completed successfully!");
            info!("Final statistics: {:?}", stats);
            println!("✅ Crawling completed successfully!");
            println!("📊 Statistics:");
            println!("   Total URLs: {}", stats.total_urls);
            println!("   Successful conversions: {}", stats.successful_conversions);
            println!("   Failed conversions: {}", stats.failed_conversions);
            println!("   Files saved: {}", stats.files_saved);
        }
        Err(e) => {
            eprintln!("❌ Crawler error: {:?}", e);
            std::process::exit(1);
        }
    }
}
