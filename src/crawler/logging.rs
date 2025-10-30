use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Structured logging utilities for the web crawler
pub struct CrawlerLogger {
    start_time: Instant,
    operation_timers: HashMap<String, Instant>,
    stats: LoggingStats,
}

/// Statistics for logging operations
#[derive(Debug, Default)]
pub struct LoggingStats {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub warnings_count: usize,
    pub errors_count: usize,
}

/// Log levels for different types of operations
#[derive(Debug, Clone)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Context information for structured logging
#[derive(Debug, Clone)]
pub struct LogContext {
    pub operation: String,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub file_path: Option<String>,
    pub additional_data: HashMap<String, String>,
}

impl CrawlerLogger {
    /// Create a new crawler logger
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            operation_timers: HashMap::new(),
            stats: LoggingStats::default(),
        }
    }

    /// Start timing an operation
    pub fn start_operation(&mut self, operation_name: &str) {
        self.operation_timers.insert(operation_name.to_string(), Instant::now());
        info!("🚀 Starting operation: {}", operation_name);
    }

    /// End timing an operation and log the duration
    pub fn end_operation(&mut self, operation_name: &str, success: bool) {
        if let Some(start_time) = self.operation_timers.remove(operation_name) {
            let duration = start_time.elapsed();
            let status = if success { "✅ SUCCESS" } else { "❌ FAILED" };
            
            info!("{} Operation '{}' completed in {:?}", status, operation_name, duration);
            
            self.stats.total_operations += 1;
            if success {
                self.stats.successful_operations += 1;
            } else {
                self.stats.failed_operations += 1;
            }
        }
    }

    /// Log domain detection activity
    pub fn log_domain_detection(&mut self, domain: &str, mode: &str, success: bool) {
        let context = LogContext {
            operation: "domain_detection".to_string(),
            url: None,
            domain: Some(domain.to_string()),
            file_path: None,
            additional_data: {
                let mut data = HashMap::new();
                data.insert("fetch_mode".to_string(), mode.to_string());
                data.insert("success".to_string(), success.to_string());
                data
            },
        };

        if success {
            info!("🎯 Domain detection: {} -> {} mode", domain, mode);
        } else {
            warn!("⚠️  Domain detection failed for: {}", domain);
            self.stats.warnings_count += 1;
        }

        self.log_structured(LogLevel::Info, "Domain detection completed", &context);
    }

    /// Log HTML conversion activity
    pub fn log_html_conversion(&mut self, url: &str, input_size: usize, output_size: usize, success: bool) {
        let context = LogContext {
            operation: "html_conversion".to_string(),
            url: Some(url.to_string()),
            domain: None,
            file_path: None,
            additional_data: {
                let mut data = HashMap::new();
                data.insert("input_size_bytes".to_string(), input_size.to_string());
                data.insert("output_size_bytes".to_string(), output_size.to_string());
                data.insert("compression_ratio".to_string(), 
                    format!("{:.2}", output_size as f64 / input_size.max(1) as f64));
                data.insert("success".to_string(), success.to_string());
                data
            },
        };

        if success {
            info!("📝 HTML conversion: {} ({} bytes -> {} bytes)", 
                  url, input_size, output_size);
        } else {
            error!("❌ HTML conversion failed for: {} ({} bytes input)", url, input_size);
            self.stats.errors_count += 1;
        }

        let level = if success { LogLevel::Info } else { LogLevel::Error };
        self.log_structured(level, "HTML conversion completed", &context);
    }

    /// Log file operation activity
    pub fn log_file_operation(&mut self, operation: &str, file_path: &str, success: bool, error_msg: Option<&str>) {
        let context = LogContext {
            operation: format!("file_{}", operation),
            url: None,
            domain: None,
            file_path: Some(file_path.to_string()),
            additional_data: {
                let mut data = HashMap::new();
                data.insert("operation_type".to_string(), operation.to_string());
                data.insert("success".to_string(), success.to_string());
                if let Some(error) = error_msg {
                    data.insert("error_message".to_string(), error.to_string());
                }
                data
            },
        };

        if success {
            info!("💾 File {}: {}", operation, file_path);
        } else {
            let error_text = error_msg.unwrap_or("Unknown error");
            error!("❌ File {} failed: {} - {}", operation, file_path, error_text);
            self.stats.errors_count += 1;
        }

        let level = if success { LogLevel::Info } else { LogLevel::Error };
        self.log_structured(level, &format!("File {} completed", operation), &context);
    }

    /// Log spider/web scraping activity
    pub fn log_spider_activity(&mut self, operation: &str, url: &str, success: bool, details: Option<&str>) {
        let context = LogContext {
            operation: format!("spider_{}", operation),
            url: Some(url.to_string()),
            domain: None,
            file_path: None,
            additional_data: {
                let mut data = HashMap::new();
                data.insert("spider_operation".to_string(), operation.to_string());
                data.insert("success".to_string(), success.to_string());
                if let Some(detail) = details {
                    data.insert("details".to_string(), detail.to_string());
                }
                data
            },
        };

        if success {
            info!("🕷️  Spider {}: {}", operation, url);
            if let Some(detail) = details {
                debug!("   Details: {}", detail);
            }
        } else {
            let detail_text = details.unwrap_or("No additional details");
            warn!("⚠️  Spider {} failed: {} - {}", operation, url, detail_text);
            self.stats.warnings_count += 1;
        }

        let level = if success { LogLevel::Info } else { LogLevel::Warn };
        self.log_structured(level, &format!("Spider {} completed", operation), &context);
    }

    /// Log configuration activity
    pub fn log_configuration(&mut self, operation: &str, config_path: Option<&str>, success: bool, details: Option<&str>) {
        let context = LogContext {
            operation: format!("config_{}", operation),
            url: None,
            domain: None,
            file_path: config_path.map(|s| s.to_string()),
            additional_data: {
                let mut data = HashMap::new();
                data.insert("config_operation".to_string(), operation.to_string());
                data.insert("success".to_string(), success.to_string());
                if let Some(detail) = details {
                    data.insert("details".to_string(), detail.to_string());
                }
                data
            },
        };

        if success {
            info!("⚙️  Configuration {}: {}", operation, 
                  config_path.unwrap_or("default"));
            if let Some(detail) = details {
                debug!("   Details: {}", detail);
            }
        } else {
            let detail_text = details.unwrap_or("No additional details");
            error!("❌ Configuration {} failed: {} - {}", operation, 
                   config_path.unwrap_or("default"), detail_text);
            self.stats.errors_count += 1;
        }

        let level = if success { LogLevel::Info } else { LogLevel::Error };
        self.log_structured(level, &format!("Configuration {} completed", operation), &context);
    }

    /// Log error with recovery information
    pub fn log_error_with_recovery(&mut self, error: &crate::crawler::errors::CrawlerError, 
                                   recovery_attempted: bool, recovery_success: Option<bool>) {
        let is_recoverable = crate::crawler::errors::ErrorRecovery::is_recoverable(error);
        let retry_count = crate::crawler::errors::ErrorRecovery::get_retry_count(error);
        let fallback_suggestion = crate::crawler::errors::ErrorRecovery::suggest_fallback(error);

        error!("❌ Error occurred: {}", error);
        
        if is_recoverable {
            info!("🔄 Error is recoverable (max retries: {})", retry_count);
            
            if recovery_attempted {
                match recovery_success {
                    Some(true) => info!("✅ Error recovery successful"),
                    Some(false) => error!("❌ Error recovery failed"),
                    None => info!("🔄 Error recovery in progress"),
                }
            }
        } else {
            warn!("⚠️  Error is not recoverable");
            
            if let Some(suggestion) = fallback_suggestion {
                info!("💡 Suggested fallback: {}", suggestion);
            }
        }

        self.stats.errors_count += 1;
    }

    /// Log performance metrics
    pub fn log_performance_metrics(&self, operation: &str, metrics: &PerformanceMetrics) {
        info!("📊 Performance metrics for {}: {:?}", operation, metrics);
        
        debug!("   Duration: {:?}", metrics.duration);
        debug!("   Memory usage: {} bytes", metrics.memory_usage_bytes);
        debug!("   Items processed: {}", metrics.items_processed);
        
        if metrics.items_processed > 0 {
            let items_per_second = metrics.items_processed as f64 / metrics.duration.as_secs_f64();
            debug!("   Processing rate: {:.2} items/second", items_per_second);
        }
    }

    /// Log pipeline progress
    pub fn log_pipeline_progress(&self, stage: &str, current: usize, total: usize, eta: Option<Duration>) {
        let percentage = if total > 0 { (current as f64 / total as f64) * 100.0 } else { 0.0 };
        
        info!("📈 Pipeline progress [{}]: {}/{} ({:.1}%)", stage, current, total, percentage);
        
        if let Some(estimated_time) = eta {
            debug!("   Estimated time remaining: {:?}", estimated_time);
        }
    }

    /// Get current logging statistics
    pub fn get_stats(&self) -> &LoggingStats {
        &self.stats
    }

    /// Get total elapsed time since logger creation
    pub fn get_total_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Log final summary
    pub fn log_final_summary(&self) {
        let total_time = self.get_total_elapsed();
        
        info!("🏁 Crawler session completed in {:?}", total_time);
        info!("📊 Final statistics:");
        info!("   Total operations: {}", self.stats.total_operations);
        info!("   Successful: {}", self.stats.successful_operations);
        info!("   Failed: {}", self.stats.failed_operations);
        info!("   Warnings: {}", self.stats.warnings_count);
        info!("   Errors: {}", self.stats.errors_count);
        
        let success_rate = if self.stats.total_operations > 0 {
            (self.stats.successful_operations as f64 / self.stats.total_operations as f64) * 100.0
        } else {
            0.0
        };
        
        info!("   Success rate: {:.1}%", success_rate);
    }

    /// Internal method for structured logging
    fn log_structured(&self, level: LogLevel, message: &str, context: &LogContext) {
        let log_entry = format!("[{}] {} | URL: {} | Domain: {} | File: {} | Data: {:?}",
            context.operation,
            message,
            context.url.as_deref().unwrap_or("N/A"),
            context.domain.as_deref().unwrap_or("N/A"),
            context.file_path.as_deref().unwrap_or("N/A"),
            context.additional_data
        );

        match level {
            LogLevel::Trace => trace!("{}", log_entry),
            LogLevel::Debug => debug!("{}", log_entry),
            LogLevel::Info => info!("{}", log_entry),
            LogLevel::Warn => warn!("{}", log_entry),
            LogLevel::Error => error!("{}", log_entry),
        }
    }
}

impl Default for CrawlerLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics for logging
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub duration: Duration,
    pub memory_usage_bytes: usize,
    pub items_processed: usize,
    pub bytes_processed: usize,
}

impl PerformanceMetrics {
    pub fn new(duration: Duration, items_processed: usize) -> Self {
        Self {
            duration,
            memory_usage_bytes: 0, // Could be implemented with system metrics
            items_processed,
            bytes_processed: 0,
        }
    }

    pub fn with_memory_usage(mut self, bytes: usize) -> Self {
        self.memory_usage_bytes = bytes;
        self
    }

    pub fn with_bytes_processed(mut self, bytes: usize) -> Self {
        self.bytes_processed = bytes;
        self
    }
}

/// Utility macros for common logging patterns
#[macro_export]
macro_rules! log_operation_start {
    ($logger:expr, $operation:expr) => {
        $logger.start_operation($operation);
    };
}

#[macro_export]
macro_rules! log_operation_end {
    ($logger:expr, $operation:expr, $success:expr) => {
        $logger.end_operation($operation, $success);
    };
}

#[macro_export]
macro_rules! log_with_context {
    ($level:expr, $message:expr, $context:expr) => {
        match $level {
            LogLevel::Info => log::info!("{} | Context: {:?}", $message, $context),
            LogLevel::Warn => log::warn!("{} | Context: {:?}", $message, $context),
            LogLevel::Error => log::error!("{} | Context: {:?}", $message, $context),
            LogLevel::Debug => log::debug!("{} | Context: {:?}", $message, $context),
            LogLevel::Trace => log::trace!("{} | Context: {:?}", $message, $context),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawler_logger_creation() {
        let logger = CrawlerLogger::new();
        assert_eq!(logger.stats.total_operations, 0);
        assert_eq!(logger.stats.successful_operations, 0);
        assert_eq!(logger.stats.failed_operations, 0);
    }

    #[test]
    fn test_operation_timing() {
        let mut logger = CrawlerLogger::new();
        
        logger.start_operation("test_operation");
        std::thread::sleep(Duration::from_millis(10));
        logger.end_operation("test_operation", true);
        
        assert_eq!(logger.stats.total_operations, 1);
        assert_eq!(logger.stats.successful_operations, 1);
        assert_eq!(logger.stats.failed_operations, 0);
    }

    #[test]
    fn test_logging_stats() {
        let mut logger = CrawlerLogger::new();
        
        logger.log_domain_detection("example.com", "HttpRequest", true);
        logger.log_html_conversion("https://example.com", 1000, 500, true);
        logger.log_file_operation("save", "/path/to/file.md", false, Some("Permission denied"));
        
        let stats = logger.get_stats();
        assert_eq!(stats.errors_count, 1);
        assert!(logger.get_total_elapsed() > Duration::from_nanos(1));
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new(Duration::from_secs(5), 100)
            .with_memory_usage(1024)
            .with_bytes_processed(50000);
        
        assert_eq!(metrics.duration, Duration::from_secs(5));
        assert_eq!(metrics.items_processed, 100);
        assert_eq!(metrics.memory_usage_bytes, 1024);
        assert_eq!(metrics.bytes_processed, 50000);
    }

    #[test]
    fn test_log_context_creation() {
        let context = LogContext {
            operation: "test".to_string(),
            url: Some("https://example.com".to_string()),
            domain: Some("example.com".to_string()),
            file_path: Some("/path/to/file.md".to_string()),
            additional_data: HashMap::new(),
        };
        
        assert_eq!(context.operation, "test");
        assert_eq!(context.url.unwrap(), "https://example.com");
        assert_eq!(context.domain.unwrap(), "example.com");
        assert_eq!(context.file_path.unwrap(), "/path/to/file.md");
    }
}