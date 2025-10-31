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
    pub chrome_executable: Option<String>,
    pub native_download_mode: Option<String>,
    pub fetch_mode: Option<String>, // override auto-detection: "Chrome" or "HttpRequest"
    pub depth: Option<usize>,
    pub max_pages: Option<usize>,
    pub sitemap_max_depth: Option<usize>, // สำหรับ recursive sitemap loading
    pub max_sitemap_urls: Option<usize>, // จำกัดจำนวน URL จาก sitemap
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            user_agent: Some("MyRustCrawler/1.0".into()),
            delay_ms: Some(250),
            whitelist_path: Some("src/config/whitelist.yaml".into()),
            chrome_executable: None,
            native_download_mode: Some("HttpRequest".into()),
            fetch_mode: None, // None = auto-detect, Some("Chrome") or Some("HttpRequest") = force mode
            depth: Some(3),
            max_pages: Some(200),
            sitemap_max_depth: Some(5), // รองรับ sitemap ซ้อนได้ 5 ชั้น
            max_sitemap_urls: Some(100), // default 100 URLs from sitemap
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
                        if let Some(ref exe) = cfg.chrome_executable {
                            unsafe { env::set_var("CHROME_EXECUTABLE", exe); }
                            println!("[config] set CHROME_EXECUTABLE={}", exe);
                        }
                        println!("[config] loaded {}", p);
                        // Print all known config fields for visibility
                        println!(
                            "[config] values: user_agent={:?}, delay_ms={:?}, whitelist_path={:?}, chrome_executable={:?}, native_download_mode={:?}, depth={:?}, max_pages={:?}, sitemap_max_depth={:?}",
                            cfg.user_agent,
                            cfg.delay_ms,
                            cfg.whitelist_path,
                            cfg.chrome_executable,
                            cfg.native_download_mode,
                            cfg.depth,
                            cfg.max_pages,
                            cfg.sitemap_max_depth
                        );
                        return cfg; // ensure we return the parsed config
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
    let default_cfg = AppConfig::default();
    // Print default values as well
    println!(
        "[config] default values: user_agent={:?}, delay_ms={:?}, whitelist_path={:?}, chrome_executable={:?}, native_download_mode={:?}, depth={:?}, max_pages={:?}, sitemap_max_depth={:?}",
        default_cfg.user_agent,
        default_cfg.delay_ms,
        default_cfg.whitelist_path,
        default_cfg.chrome_executable,
        default_cfg.native_download_mode,
        default_cfg.depth,
        default_cfg.max_pages,
        default_cfg.sitemap_max_depth
    );
    default_cfg
}