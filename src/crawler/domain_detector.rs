use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::html_fetcher::FetchMode;

#[derive(Debug, Deserialize, Clone)]
struct WhitelistEntry {
    domain: String,
    mode: String,
    r#match: String, // Use raw identifier to avoid keyword conflict with 'match'
}

#[derive(Debug, Deserialize, Clone)]
struct DomainWhitelist {
    default_mode: String,
    whitelist: Vec<WhitelistEntry>,
}

impl Default for DomainWhitelist {
    fn default() -> Self {
        Self {
            default_mode: "SSR".to_string(),
            whitelist: vec![],
        }
    }
}

pub struct DomainDetector {
    whitelist: DomainWhitelist,
    cache: HashMap<String, FetchMode>,
}

impl DomainDetector {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let whitelist: DomainWhitelist = serde_yaml::from_str(&content)?;
        Ok(Self {
            whitelist,
            cache: HashMap::new(),
        })
    }

    pub fn default() -> Self {
        Self {
            whitelist: DomainWhitelist::default(),
            cache: HashMap::new(),
        }
    }

    fn normalize_domain(s: &str) -> String {
        let mut v = s.trim().to_lowercase();
        if v.starts_with("http://") {
            v = v.replacen("http://", "", 1);
        } else if v.starts_with("https://") {
            v = v.replacen("https://", "", 1);
        }
        if v.starts_with("www.") {
            v = v.replacen("www.", "", 1);
        }
        if let Some(pos) = v.find('/') {
            v.truncate(pos);
        }
        v
    }

    pub fn get_fetch_mode_for_domain(&mut self, domain: &str) -> FetchMode {
        let normalized = Self::normalize_domain(domain);
        if let Some(m) = self.cache.get(&normalized) {
            return *m;
        }

        for entry in &self.whitelist.whitelist {
            let matches = match entry.r#match.as_str() {
                "exact" => normalized == entry.domain,
                "subdomain" => normalized == entry.domain || normalized.ends_with(&format!(".{}", entry.domain)),
                _ => false,
            };
            if matches {
                let mode = match entry.mode.as_str() {
                    "SPA" => FetchMode::Chrome,
                    "SSR" => FetchMode::HttpRequest,
                    _ => FetchMode::from_str(&self.whitelist.default_mode),
                };
                self.cache.insert(normalized, mode);
                return mode;
            }
        }

        // Default case: no whitelist match, use default mode (fixes the missing return path)
        let mode = FetchMode::from_str(&self.whitelist.default_mode);
        self.cache.insert(normalized, mode);
        mode
    }
}