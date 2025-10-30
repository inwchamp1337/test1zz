use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::crawler::errors::{DomainDetectionError, CrawlerResult};
use log::{debug, error, info, trace, warn};
use url::Url;

/// Enum representing the fetch mode for different types of websites
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchMode {
    /// Use HTTP requests for server-side rendered websites
    HttpRequest,
    /// Use Chrome browser for single-page applications requiring JavaScript
    Chrome,
}

/// Configuration structure for domain detection
#[derive(Debug, Deserialize, Serialize)]
pub struct DomainConfig {
    pub spa_domains: Vec<String>,
    pub ssr_domains: Vec<String>,
}

/// Domain detector that determines the appropriate fetch mode for websites
pub struct DomainDetector {
    spa_domains: HashSet<String>,
    ssr_domains: HashSet<String>,
}

impl DomainDetector {
    /// Create a new DomainDetector with empty domain lists
    pub fn new() -> Self {
        Self {
            spa_domains: HashSet::new(),
            ssr_domains: HashSet::new(),
        }
    }

    /// Create a DomainDetector from configuration
    pub fn from_config(config: DomainConfig) -> Self {
        let mut detector = Self::new();
        
        for domain in config.spa_domains {
            detector.add_spa_domain(domain);
        }
        
        for domain in config.ssr_domains {
            detector.add_ssr_domain(domain);
        }
        
        detector
    }

    /// Load configuration from a YAML file
    pub fn load_from_yaml(file_path: &str) -> CrawlerResult<Self> {
        trace!("Loading domain detector configuration from: {}", file_path);
        
        let config_content = std::fs::read_to_string(file_path)
            .map_err(|e| {
                error!("Failed to read domain configuration file '{}': {}", file_path, e);
                DomainDetectionError::ConfigurationLoadFailed(format!("File read error: {}", e))
            })?;
        
        let config: DomainConfig = serde_yaml::from_str(&config_content)
            .map_err(|e| {
                error!("Failed to parse domain configuration YAML: {}", e);
                DomainDetectionError::ConfigurationLoadFailed(format!("YAML parse error: {}", e))
            })?;
        
        info!("Successfully loaded domain configuration with {} SPA domains and {} SSR domains", 
              config.spa_domains.len(), config.ssr_domains.len());
        
        Ok(Self::from_config(config))
    }

    /// Check if a domain is configured as SPA
    pub fn is_spa_domain(&self, domain: &str) -> bool {
        // Normalize domain by removing protocol and www prefix
        let normalized_domain = self.normalize_domain(domain);
        self.spa_domains.contains(&normalized_domain)
    }

    /// Check if a domain is configured as SSR
    pub fn is_ssr_domain(&self, domain: &str) -> bool {
        let normalized_domain = self.normalize_domain(domain);
        self.ssr_domains.contains(&normalized_domain)
    }

    /// Get the appropriate fetch mode for a domain
    pub fn get_fetch_mode(&self, domain: &str) -> FetchMode {
        trace!("Determining fetch mode for domain: {}", domain);
        
        // Validate domain format
        if domain.is_empty() {
            warn!("Empty domain provided, defaulting to HttpRequest mode");
            return FetchMode::HttpRequest;
        }

        let mode = if self.is_spa_domain(domain) {
            debug!("Domain '{}' configured as SPA, using Chrome mode", domain);
            FetchMode::Chrome
        } else if self.is_ssr_domain(domain) {
            debug!("Domain '{}' configured as SSR, using HttpRequest mode", domain);
            FetchMode::HttpRequest
        } else {
            debug!("Domain '{}' not configured, defaulting to HttpRequest mode", domain);
            FetchMode::HttpRequest
        };

        trace!("Selected fetch mode for '{}': {:?}", domain, mode);
        mode
    }

    /// Get the appropriate fetch mode for a domain with error handling
    pub fn get_fetch_mode_safe(&self, domain: &str) -> CrawlerResult<FetchMode> {
        if domain.is_empty() {
            error!("Cannot determine fetch mode for empty domain");
            return Err(DomainDetectionError::InvalidDomain("Empty domain".to_string()).into());
        }

        // Extract domain from URL if needed
        let clean_domain = self.extract_domain_from_url(domain);
        
        // Basic domain validation
        if !self.is_valid_domain_format(&clean_domain) {
            error!("Invalid domain format: {}", clean_domain);
            return Err(DomainDetectionError::InvalidDomain(format!("Invalid format: {}", clean_domain)).into());
        }

        Ok(self.get_fetch_mode(domain))
    }

    /// Extract domain from URL or return as-is if already a domain
    fn extract_domain_from_url(&self, input: &str) -> String {
        // If it looks like a URL, extract the domain
        if input.starts_with("http://") || input.starts_with("https://") {
            if let Ok(parsed) = Url::parse(input) {
                if let Some(host) = parsed.host_str() {
                    return host.to_string();
                }
            }
        }
        
        // Otherwise, assume it's already a domain
        input.to_string()
    }

    /// Validate domain format
    fn is_valid_domain_format(&self, domain: &str) -> bool {
        // Basic domain validation
        if domain.is_empty() || domain.len() > 253 {
            return false;
        }

        // Check for valid characters (simplified validation)
        // Allow alphanumeric, dots, hyphens, and underscores
        domain.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
    }

    /// Add a domain to the SPA domains list
    pub fn add_spa_domain(&mut self, domain: String) {
        let normalized_domain = self.normalize_domain(&domain);
        self.spa_domains.insert(normalized_domain);
    }

    /// Add a domain to the SSR domains list
    pub fn add_ssr_domain(&mut self, domain: String) {
        let normalized_domain = self.normalize_domain(&domain);
        self.ssr_domains.insert(normalized_domain);
    }

    /// Remove a domain from SPA domains list
    pub fn remove_spa_domain(&mut self, domain: &str) -> bool {
        let normalized_domain = self.normalize_domain(domain);
        self.spa_domains.remove(&normalized_domain)
    }

    /// Remove a domain from SSR domains list
    pub fn remove_ssr_domain(&mut self, domain: &str) -> bool {
        let normalized_domain = self.normalize_domain(domain);
        self.ssr_domains.remove(&normalized_domain)
    }

    /// Get all configured SPA domains
    pub fn get_spa_domains(&self) -> Vec<String> {
        self.spa_domains.iter().cloned().collect()
    }

    /// Get all configured SSR domains
    pub fn get_ssr_domains(&self) -> Vec<String> {
        self.ssr_domains.iter().cloned().collect()
    }

    /// Normalize domain by removing protocol, www prefix, and trailing slashes
    fn normalize_domain(&self, domain: &str) -> String {
        let mut normalized = domain.to_lowercase();
        
        // Remove protocol
        if normalized.starts_with("https://") {
            normalized = normalized[8..].to_string();
        } else if normalized.starts_with("http://") {
            normalized = normalized[7..].to_string();
        }
        
        // Remove www prefix
        if normalized.starts_with("www.") {
            normalized = normalized[4..].to_string();
        }
        
        // Remove trailing slash and path
        if let Some(slash_pos) = normalized.find('/') {
            normalized = normalized[..slash_pos].to_string();
        }
        
        normalized
    }
}

impl Default for DomainDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_new_domain_detector() {
        let detector = DomainDetector::new();
        assert_eq!(detector.spa_domains.len(), 0);
        assert_eq!(detector.ssr_domains.len(), 0);
    }

    #[test]
    fn test_add_spa_domain() {
        let mut detector = DomainDetector::new();
        detector.add_spa_domain("example.com".to_string());
        assert!(detector.is_spa_domain("example.com"));
        assert_eq!(detector.get_fetch_mode("example.com"), FetchMode::Chrome);
    }

    #[test]
    fn test_add_ssr_domain() {
        let mut detector = DomainDetector::new();
        detector.add_ssr_domain("example.org".to_string());
        assert!(detector.is_ssr_domain("example.org"));
        assert_eq!(detector.get_fetch_mode("example.org"), FetchMode::HttpRequest);
    }

    #[test]
    fn test_domain_normalization() {
        let mut detector = DomainDetector::new();
        detector.add_spa_domain("example.com".to_string());
        
        // Test various URL formats
        assert!(detector.is_spa_domain("https://example.com"));
        assert!(detector.is_spa_domain("http://example.com"));
        assert!(detector.is_spa_domain("www.example.com"));
        assert!(detector.is_spa_domain("https://www.example.com"));
        assert!(detector.is_spa_domain("example.com/path"));
        assert!(detector.is_spa_domain("EXAMPLE.COM"));
    }

    #[test]
    fn test_unknown_domain_defaults_to_http() {
        let detector = DomainDetector::new();
        assert_eq!(detector.get_fetch_mode("unknown.com"), FetchMode::HttpRequest);
    }

    #[test]
    fn test_from_config() {
        let config = DomainConfig {
            spa_domains: vec!["spa1.com".to_string(), "spa2.com".to_string()],
            ssr_domains: vec!["ssr1.com".to_string(), "ssr2.com".to_string()],
        };
        
        let detector = DomainDetector::from_config(config);
        
        assert!(detector.is_spa_domain("spa1.com"));
        assert!(detector.is_spa_domain("spa2.com"));
        assert!(detector.is_ssr_domain("ssr1.com"));
        assert!(detector.is_ssr_domain("ssr2.com"));
    }

    #[test]
    fn test_remove_domains() {
        let mut detector = DomainDetector::new();
        detector.add_spa_domain("example.com".to_string());
        detector.add_ssr_domain("test.org".to_string());
        
        assert!(detector.remove_spa_domain("example.com"));
        assert!(detector.remove_ssr_domain("test.org"));
        assert!(!detector.is_spa_domain("example.com"));
        assert!(!detector.is_ssr_domain("test.org"));
    }

    #[test]
    fn test_load_from_yaml() {
        // Create a temporary YAML file for testing
        let yaml_content = r#"
spa_domains:
  - "test-spa.com"
  - "another-spa.com"
ssr_domains:
  - "test-ssr.com"
  - "another-ssr.com"
"#;
        
        let temp_file = "test_config.yaml";
        fs::write(temp_file, yaml_content).expect("Failed to write test config");
        
        // Test loading from YAML
        let detector = DomainDetector::load_from_yaml(temp_file)
            .expect("Failed to load from YAML");
        
        assert!(detector.is_spa_domain("test-spa.com"));
        assert!(detector.is_spa_domain("another-spa.com"));
        assert!(detector.is_ssr_domain("test-ssr.com"));
        assert!(detector.is_ssr_domain("another-ssr.com"));
        
        assert_eq!(detector.get_fetch_mode("test-spa.com"), FetchMode::Chrome);
        assert_eq!(detector.get_fetch_mode("test-ssr.com"), FetchMode::HttpRequest);
        
        // Clean up
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_get_domains_lists() {
        let mut detector = DomainDetector::new();
        detector.add_spa_domain("spa1.com".to_string());
        detector.add_spa_domain("spa2.com".to_string());
        detector.add_ssr_domain("ssr1.com".to_string());
        
        let spa_domains = detector.get_spa_domains();
        let ssr_domains = detector.get_ssr_domains();
        
        assert_eq!(spa_domains.len(), 2);
        assert_eq!(ssr_domains.len(), 1);
        assert!(spa_domains.contains(&"spa1.com".to_string()));
        assert!(spa_domains.contains(&"spa2.com".to_string()));
        assert!(ssr_domains.contains(&"ssr1.com".to_string()));
    }
}