use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure for the web crawler
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CrawlerConfig {
    /// List of domains that require SPA (Chrome) mode
    pub spa_domains: Vec<String>,
    /// List of domains that use SSR (HTTP request) mode
    pub ssr_domains: Vec<String>,
    /// Output directory for saved files
    pub output_directory: String,
    /// Spider-specific configuration
    pub spider_config: SpiderConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Configuration for spider crawling behavior
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpiderConfig {
    /// Maximum crawl depth
    pub depth: u32,
    /// Delay between requests in milliseconds
    pub delay_ms: u64,
    /// User agent string for requests
    pub user_agent: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
    /// Whether to respect robots.txt
    pub respect_robots_txt: bool,
}

/// Configuration for logging behavior
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Whether to enable file logging
    pub enable_file_logging: bool,
    /// Log file path (if file logging is enabled)
    pub log_file: String,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            spa_domains: vec![
                "www.heygoody.com".to_string(),
                "app.example.com".to_string(),
            ],
            ssr_domains: vec![
                "www.rust-lang.org".to_string(),
                "docs.rs".to_string(),
                "github.com".to_string(),
            ],
            output_directory: "output".to_string(),
            spider_config: SpiderConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for SpiderConfig {
    fn default() -> Self {
        Self {
            depth: 3,
            delay_ms: 200,
            user_agent: "RustWebCrawler/1.0".to_string(),
            timeout_seconds: 30,
            max_concurrent_requests: 10,
            respect_robots_txt: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            enable_file_logging: false,
            log_file: "crawler.log".to_string(),
        }
    }
}

impl CrawlerConfig {
    /// Load configuration from a YAML file
    pub fn load_from_yaml(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = std::fs::read_to_string(file_path)?;
        let config: CrawlerConfig = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }

    /// Load configuration with fallback to default if file doesn't exist
    pub fn load_or_default(file_path: &str) -> Self {
        match Self::load_from_yaml(file_path) {
            Ok(config) => {
                log::info!("Loaded configuration from {}", file_path);
                config
            }
            Err(e) => {
                log::warn!("Failed to load configuration from {}: {}. Using default configuration.", file_path, e);
                Self::default()
            }
        }
    }

    /// Save configuration to a YAML file
    pub fn save_to_yaml(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_content = serde_yaml::to_string(self)?;
        
        // Ensure directory exists
        if let Some(parent) = PathBuf::from(file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(file_path, yaml_content)?;
        Ok(())
    }

    /// Get output directory as PathBuf
    pub fn get_output_path(&self) -> PathBuf {
        PathBuf::from(&self.output_directory)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        // Validate output directory
        if self.output_directory.is_empty() {
            return Err("Output directory cannot be empty".to_string());
        }

        // Validate spider config
        if self.spider_config.depth == 0 {
            return Err("Spider depth must be greater than 0".to_string());
        }

        if self.spider_config.delay_ms > 10000 {
            return Err("Delay between requests should not exceed 10 seconds".to_string());
        }

        if self.spider_config.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0 seconds".to_string());
        }

        if self.spider_config.max_concurrent_requests == 0 {
            return Err("Max concurrent requests must be greater than 0".to_string());
        }

        if self.spider_config.user_agent.is_empty() {
            return Err("User agent cannot be empty".to_string());
        }

        // Validate logging config
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            return Err(format!("Invalid log level '{}'. Must be one of: {:?}", 
                self.logging.level, valid_log_levels));
        }

        if self.logging.enable_file_logging && self.logging.log_file.is_empty() {
            return Err("Log file path cannot be empty when file logging is enabled".to_string());
        }

        Ok(())
    }

    /// Initialize logging based on configuration
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        use log::LevelFilter;
        
        let log_level = match self.logging.level.as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Info,
        };

        if self.logging.enable_file_logging {
            // TODO: Implement file logging if needed
            // For now, just use env_logger
            env_logger::Builder::from_default_env()
                .filter_level(log_level)
                .init();
        } else {
            env_logger::Builder::from_default_env()
                .filter_level(log_level)
                .init();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = CrawlerConfig::default();
        assert!(!config.spa_domains.is_empty());
        assert!(!config.ssr_domains.is_empty());
        assert_eq!(config.output_directory, "output");
        assert_eq!(config.spider_config.depth, 3);
        assert_eq!(config.spider_config.delay_ms, 200);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = CrawlerConfig::default();
        
        // Valid config should pass
        assert!(config.validate().is_ok());
        
        // Invalid output directory
        config.output_directory = "".to_string();
        assert!(config.validate().is_err());
        
        // Reset and test invalid depth
        config = CrawlerConfig::default();
        config.spider_config.depth = 0;
        assert!(config.validate().is_err());
        
        // Reset and test invalid log level
        config = CrawlerConfig::default();
        config.logging.level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_save_and_load_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");
        
        let original_config = CrawlerConfig::default();
        
        // Save config
        original_config.save_to_yaml(config_path.to_str().unwrap()).unwrap();
        
        // Load config
        let loaded_config = CrawlerConfig::load_from_yaml(config_path.to_str().unwrap()).unwrap();
        
        // Compare key fields
        assert_eq!(original_config.output_directory, loaded_config.output_directory);
        assert_eq!(original_config.spider_config.depth, loaded_config.spider_config.depth);
        assert_eq!(original_config.logging.level, loaded_config.logging.level);
    }

    #[test]
    fn test_load_or_default() {
        // Test with non-existent file
        let config = CrawlerConfig::load_or_default("non_existent_file.yaml");
        assert_eq!(config.output_directory, "output");
        
        // Test with existing file
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");
        
        let yaml_content = r#"
spa_domains:
  - "test.com"
ssr_domains:
  - "example.com"
output_directory: "custom_output"
spider_config:
  depth: 5
  delay_ms: 500
  user_agent: "TestAgent/1.0"
  timeout_seconds: 60
  max_concurrent_requests: 5
  respect_robots_txt: false
logging:
  level: "debug"
  enable_file_logging: true
  log_file: "test.log"
"#;
        
        fs::write(&config_path, yaml_content).unwrap();
        
        let config = CrawlerConfig::load_or_default(config_path.to_str().unwrap());
        assert_eq!(config.output_directory, "custom_output");
        assert_eq!(config.spider_config.depth, 5);
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn test_get_output_path() {
        let config = CrawlerConfig::default();
        let path = config.get_output_path();
        assert_eq!(path, PathBuf::from("output"));
    }
}