mod config;
mod crawler;
use std::env;

#[tokio::main]
async fn main() {
    let domain = env::args()
        .nth(1)
        .unwrap_or_else(|| "https://www.rust-lang.org/".to_string());

    // เรียกผ่านโมดูลย่อยที่อยู่ใน crawler/
    if let Err(e) = crate::crawler::crawler::run_crawler(&domain).await {
        eprintln!("Error: {:?}", e);
    }
}
