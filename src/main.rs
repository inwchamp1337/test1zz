mod robots;
mod crawler;

use std::env;

#[tokio::main]
async fn main() {
    let domain = env::args()
        .nth(1)
        .unwrap_or_else(|| "https://www.rust-lang.org/".to_string());

    if let Err(e) = crawler::run_crawler(&domain).await {
        eprintln!("Error: {:?}", e);
    }
}
