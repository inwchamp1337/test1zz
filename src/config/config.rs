use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

/// Application configuration loaded from src/config/app.yaml (or fallback paths)
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub user_agent: Option<String>,
    pub delay_ms: Option<u64>,
    pub whitelist_path: Option<String>,
    pub chrome_executable: Option<String>, // new field
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            user_agent: Some("MyRustCrawler/1.0".into()),
            delay_ms: Some(250),
            whitelist_path: Some("src/config/whitelist.yaml".into()),
            chrome_executable: None,
        }
    }
}

/// Try loading app config from common candidate paths.
/// On any error or missing file it returns default config and prints a log message.
pub fn load_app_config() -> AppConfig {
    let candidates = ["src/config/app.yaml", "config/app.yaml", "app.yaml"];
    for p in &candidates {
        if Path::new(p).exists() {
            match fs::read_to_string(p) {
                Ok(s) => match serde_yaml::from_str::<AppConfig>(&s) {
                    Ok(cfg) => {
                        // if yaml specifies chrome_executable, export it to env for chrome_fetcher / chromiumoxide
                        if let Some(ref exe) = cfg.chrome_executable {
                            unsafe {
                                env::set_var("CHROME_EXECUTABLE", exe);
                            }
                            println!("[config] set CHROME_EXECUTABLE={}", exe);
                        }
                        println!("[config] loaded {}", p);
                        return cfg;
                    }
                    Err(e) => {
                        eprintln!("[config] failed parse {}: {:?}", p, e);
                    }
                },
                Err(e) => {
                    eprintln!("[config] failed read {}: {:?}", p, e);
                }
            }
        }
    }
    println!("[config] using default app config");
    AppConfig::default()
}