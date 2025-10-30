use std::fmt;

/// Comprehensive error types for the web crawler system
#[derive(Debug)]
pub enum CrawlerError {
    /// Domain detection related errors
    DomainDetection(DomainDetectionError),
    /// HTML conversion related errors
    HtmlConversion(HtmlConversionError),
    /// File operation related errors
    FileOperation(FileOperationError),
    /// Spider/web scraping related errors
    Spider(SpiderError),
    /// Configuration related errors
    Configuration(ConfigurationError),
    /// Network related errors
    Network(NetworkError),
    /// Validation related errors
    Validation(ValidationError),
}

/// Domain detection specific errors
#[derive(Debug)]
pub enum DomainDetectionError {
    InvalidDomain(String),
    ConfigurationLoadFailed(String),
    ModeSelectionFailed(String),
}

/// HTML conversion specific errors
#[derive(Debug)]
pub enum HtmlConversionError {
    ParseError(String),
    ProcessingError(String),
    EmptyContent,
    InvalidHtml(String),
    TagConversionFailed(String),
}

/// File operation specific errors
#[derive(Debug)]
pub enum FileOperationError {
    DirectoryCreationFailed(std::io::Error),
    FileWriteFailed(std::io::Error),
    FileReadFailed(std::io::Error),
    InvalidPath(String),
    PermissionDenied(String),
    DiskSpaceFull,
}

/// Spider/web scraping specific errors
#[derive(Debug)]
pub enum SpiderError {
    RequestFailed(String),
    TimeoutError(String),
    InvalidUrl(String),
    RobotsTxtError(String),
    SitemapParsingError(String),
    ChromeModeError(String),
    RateLimitExceeded,
}

/// Configuration specific errors
#[derive(Debug)]
pub enum ConfigurationError {
    FileNotFound(String),
    ParseError(String),
    ValidationFailed(String),
    InvalidLogLevel(String),
    MissingRequiredField(String),
}

/// Network specific errors
#[derive(Debug)]
pub enum NetworkError {
    ConnectionFailed(String),
    DnsResolutionFailed(String),
    SslError(String),
    ProxyError(String),
    TimeoutError(String),
}

/// Validation specific errors
#[derive(Debug)]
pub enum ValidationError {
    InvalidUrl(String),
    InvalidDomain(String),
    InvalidConfiguration(String),
    InvalidInput(String),
}

// Display implementations for user-friendly error messages
impl fmt::Display for CrawlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrawlerError::DomainDetection(e) => write!(f, "Domain detection error: {}", e),
            CrawlerError::HtmlConversion(e) => write!(f, "HTML conversion error: {}", e),
            CrawlerError::FileOperation(e) => write!(f, "File operation error: {}", e),
            CrawlerError::Spider(e) => write!(f, "Spider error: {}", e),
            CrawlerError::Configuration(e) => write!(f, "Configuration error: {}", e),
            CrawlerError::Network(e) => write!(f, "Network error: {}", e),
            CrawlerError::Validation(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl fmt::Display for DomainDetectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainDetectionError::InvalidDomain(domain) => {
                write!(f, "Invalid domain format: {}", domain)
            }
            DomainDetectionError::ConfigurationLoadFailed(msg) => {
                write!(f, "Failed to load domain configuration: {}", msg)
            }
            DomainDetectionError::ModeSelectionFailed(msg) => {
                write!(f, "Failed to select appropriate fetch mode: {}", msg)
            }
        }
    }
}

impl fmt::Display for HtmlConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HtmlConversionError::ParseError(msg) => write!(f, "HTML parsing failed: {}", msg),
            HtmlConversionError::ProcessingError(msg) => write!(f, "HTML processing failed: {}", msg),
            HtmlConversionError::EmptyContent => write!(f, "HTML content is empty"),
            HtmlConversionError::InvalidHtml(msg) => write!(f, "Invalid HTML structure: {}", msg),
            HtmlConversionError::TagConversionFailed(tag) => {
                write!(f, "Failed to convert HTML tag: {}", tag)
            }
        }
    }
}

impl fmt::Display for FileOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileOperationError::DirectoryCreationFailed(e) => {
                write!(f, "Failed to create directory: {}", e)
            }
            FileOperationError::FileWriteFailed(e) => write!(f, "Failed to write file: {}", e),
            FileOperationError::FileReadFailed(e) => write!(f, "Failed to read file: {}", e),
            FileOperationError::InvalidPath(path) => write!(f, "Invalid file path: {}", path),
            FileOperationError::PermissionDenied(path) => {
                write!(f, "Permission denied for path: {}", path)
            }
            FileOperationError::DiskSpaceFull => write!(f, "Insufficient disk space"),
        }
    }
}

impl fmt::Display for SpiderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpiderError::RequestFailed(url) => write!(f, "HTTP request failed for: {}", url),
            SpiderError::TimeoutError(url) => write!(f, "Request timeout for: {}", url),
            SpiderError::InvalidUrl(url) => write!(f, "Invalid URL format: {}", url),
            SpiderError::RobotsTxtError(msg) => write!(f, "robots.txt error: {}", msg),
            SpiderError::SitemapParsingError(msg) => write!(f, "Sitemap parsing error: {}", msg),
            SpiderError::ChromeModeError(msg) => write!(f, "Chrome browser mode error: {}", msg),
            SpiderError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
        }
    }
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigurationError::FileNotFound(path) => {
                write!(f, "Configuration file not found: {}", path)
            }
            ConfigurationError::ParseError(msg) => write!(f, "Configuration parse error: {}", msg),
            ConfigurationError::ValidationFailed(msg) => {
                write!(f, "Configuration validation failed: {}", msg)
            }
            ConfigurationError::InvalidLogLevel(level) => {
                write!(f, "Invalid log level: {}", level)
            }
            ConfigurationError::MissingRequiredField(field) => {
                write!(f, "Missing required configuration field: {}", field)
            }
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::ConnectionFailed(url) => write!(f, "Connection failed to: {}", url),
            NetworkError::DnsResolutionFailed(domain) => {
                write!(f, "DNS resolution failed for: {}", domain)
            }
            NetworkError::SslError(msg) => write!(f, "SSL/TLS error: {}", msg),
            NetworkError::ProxyError(msg) => write!(f, "Proxy error: {}", msg),
            NetworkError::TimeoutError(url) => write!(f, "Network timeout for: {}", url),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            ValidationError::InvalidDomain(domain) => write!(f, "Invalid domain: {}", domain),
            ValidationError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            ValidationError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

// Standard Error trait implementations
impl std::error::Error for CrawlerError {}
impl std::error::Error for DomainDetectionError {}
impl std::error::Error for HtmlConversionError {}
impl std::error::Error for FileOperationError {}
impl std::error::Error for SpiderError {}
impl std::error::Error for ConfigurationError {}
impl std::error::Error for NetworkError {}
impl std::error::Error for ValidationError {}

// Conversion implementations for common error types
impl From<std::io::Error> for CrawlerError {
    fn from(err: std::io::Error) -> Self {
        CrawlerError::FileOperation(FileOperationError::FileWriteFailed(err))
    }
}

impl From<url::ParseError> for CrawlerError {
    fn from(err: url::ParseError) -> Self {
        CrawlerError::Validation(ValidationError::InvalidUrl(err.to_string()))
    }
}

impl From<serde_yaml::Error> for CrawlerError {
    fn from(err: serde_yaml::Error) -> Self {
        CrawlerError::Configuration(ConfigurationError::ParseError(err.to_string()))
    }
}

impl From<DomainDetectionError> for CrawlerError {
    fn from(err: DomainDetectionError) -> Self {
        CrawlerError::DomainDetection(err)
    }
}

impl From<HtmlConversionError> for CrawlerError {
    fn from(err: HtmlConversionError) -> Self {
        CrawlerError::HtmlConversion(err)
    }
}

impl From<FileOperationError> for CrawlerError {
    fn from(err: FileOperationError) -> Self {
        CrawlerError::FileOperation(err)
    }
}

impl From<SpiderError> for CrawlerError {
    fn from(err: SpiderError) -> Self {
        CrawlerError::Spider(err)
    }
}

impl From<ConfigurationError> for CrawlerError {
    fn from(err: ConfigurationError) -> Self {
        CrawlerError::Configuration(err)
    }
}

impl From<NetworkError> for CrawlerError {
    fn from(err: NetworkError) -> Self {
        CrawlerError::Network(err)
    }
}

impl From<ValidationError> for CrawlerError {
    fn from(err: ValidationError) -> Self {
        CrawlerError::Validation(err)
    }
}

/// Result type alias for crawler operations
pub type CrawlerResult<T> = Result<T, CrawlerError>;

/// Error recovery strategies
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Determine if an error is recoverable
    pub fn is_recoverable(error: &CrawlerError) -> bool {
        match error {
            CrawlerError::Network(NetworkError::TimeoutError(_)) => true,
            CrawlerError::Network(NetworkError::ConnectionFailed(_)) => true,
            CrawlerError::Spider(SpiderError::RequestFailed(_)) => true,
            CrawlerError::Spider(SpiderError::TimeoutError(_)) => true,
            CrawlerError::FileOperation(FileOperationError::FileWriteFailed(_)) => true,
            CrawlerError::HtmlConversion(HtmlConversionError::ParseError(_)) => true,
            _ => false,
        }
    }

    /// Get retry count for recoverable errors
    pub fn get_retry_count(error: &CrawlerError) -> usize {
        match error {
            CrawlerError::Network(_) => 3,
            CrawlerError::Spider(SpiderError::RequestFailed(_)) => 2,
            CrawlerError::Spider(SpiderError::TimeoutError(_)) => 2,
            CrawlerError::FileOperation(_) => 1,
            _ => 0,
        }
    }

    /// Get delay before retry (in milliseconds)
    pub fn get_retry_delay(error: &CrawlerError, attempt: usize) -> u64 {
        let base_delay = match error {
            CrawlerError::Network(_) => 1000,
            CrawlerError::Spider(_) => 500,
            CrawlerError::FileOperation(_) => 100,
            _ => 0,
        };
        
        // Exponential backoff
        base_delay * (2_u64.pow(attempt as u32))
    }

    /// Suggest fallback action for non-recoverable errors
    pub fn suggest_fallback(error: &CrawlerError) -> Option<String> {
        match error {
            CrawlerError::HtmlConversion(_) => {
                Some("Save raw HTML content instead of converted Markdown".to_string())
            }
            CrawlerError::Spider(SpiderError::ChromeModeError(_)) => {
                Some("Fallback to HTTP request mode".to_string())
            }
            CrawlerError::Configuration(_) => {
                Some("Use default configuration values".to_string())
            }
            CrawlerError::FileOperation(FileOperationError::PermissionDenied(_)) => {
                Some("Try alternative output directory".to_string())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = CrawlerError::DomainDetection(
            DomainDetectionError::InvalidDomain("invalid-domain".to_string())
        );
        assert!(error.to_string().contains("Domain detection error"));
        assert!(error.to_string().contains("Invalid domain format"));
    }

    #[test]
    fn test_error_recovery_is_recoverable() {
        let recoverable_error = CrawlerError::Network(
            NetworkError::TimeoutError("https://example.com".to_string())
        );
        assert!(ErrorRecovery::is_recoverable(&recoverable_error));

        let non_recoverable_error = CrawlerError::Validation(
            ValidationError::InvalidUrl("invalid-url".to_string())
        );
        assert!(!ErrorRecovery::is_recoverable(&non_recoverable_error));
    }

    #[test]
    fn test_error_recovery_retry_count() {
        let network_error = CrawlerError::Network(
            NetworkError::ConnectionFailed("https://example.com".to_string())
        );
        assert_eq!(ErrorRecovery::get_retry_count(&network_error), 3);

        let validation_error = CrawlerError::Validation(
            ValidationError::InvalidUrl("invalid-url".to_string())
        );
        assert_eq!(ErrorRecovery::get_retry_count(&validation_error), 0);
    }

    #[test]
    fn test_error_recovery_retry_delay() {
        let network_error = CrawlerError::Network(
            NetworkError::TimeoutError("https://example.com".to_string())
        );
        assert_eq!(ErrorRecovery::get_retry_delay(&network_error, 0), 1000);
        assert_eq!(ErrorRecovery::get_retry_delay(&network_error, 1), 2000);
        assert_eq!(ErrorRecovery::get_retry_delay(&network_error, 2), 4000);
    }

    #[test]
    fn test_error_recovery_fallback_suggestions() {
        let html_error = CrawlerError::HtmlConversion(
            HtmlConversionError::ParseError("Invalid HTML".to_string())
        );
        let fallback = ErrorRecovery::suggest_fallback(&html_error);
        assert!(fallback.is_some());
        assert!(fallback.unwrap().contains("raw HTML"));
    }

    #[test]
    fn test_error_conversions() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let crawler_error: CrawlerError = io_error.into();
        
        match crawler_error {
            CrawlerError::FileOperation(FileOperationError::FileWriteFailed(_)) => {},
            _ => panic!("Expected FileOperationError"),
        }
    }
}