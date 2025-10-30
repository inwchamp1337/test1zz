use crate::crawler::domain_detector::{DomainDetector, FetchMode};
use crate::crawler::html_converter::HtmlConverter;
use crate::crawler::file_manager::FileManager;
use crate::crawler::config::CrawlerConfig;
use crate::crawler::errors::{CrawlerError, CrawlerResult, ErrorRecovery};
use crate::crawler::logging::{CrawlerLogger, PerformanceMetrics};
use log::{info, warn, error, debug, trace};
use std::time::Instant;

// Remove old error types - now using comprehensive error system from errors module

/// Result of processing a single URL
#[derive(Debug)]
pub struct ProcessingResult {
    pub url: String,
    pub success: bool,
    pub file_path: Option<std::path::PathBuf>,
    pub error: Option<String>,
}

/// Statistics for the crawling session
#[derive(Debug, Default)]
pub struct CrawlingStats {
    pub total_urls: usize,
    pub successful_conversions: usize,
    pub failed_conversions: usize,
    pub files_saved: usize,
}

/// Main crawler orchestration function with integrated pipeline
pub async fn run_crawler(domain: &str) -> CrawlerResult<CrawlingStats> {
    run_crawler_with_config(domain, None).await
}

/// Main crawler orchestration function with optional configuration file path
pub async fn run_crawler_with_config(domain: &str, config_path: Option<&str>) -> CrawlerResult<CrawlingStats> {
    let start_time = Instant::now();
    let mut logger = CrawlerLogger::new();
    
    logger.start_operation("crawler_initialization");
    
    // Load configuration
    let config = load_crawler_config(config_path, &mut logger)?;
    
    // Initialize logging based on configuration
    if let Err(e) = config.init_logging() {
        logger.log_configuration("logging_init", None, false, Some(&e.to_string()));
        eprintln!("Warning: Failed to initialize logging: {}", e);
    } else {
        logger.log_configuration("logging_init", None, true, None);
    }
    
    info!("🚀 Starting crawler for domain: {} with configuration", domain);
    debug!("Configuration: {:?}", config);
    
    logger.end_operation("crawler_initialization", true);
    
    // Initialize components with configuration
    logger.start_operation("component_initialization");
    
    let domain_detector = initialize_domain_detector_with_config(&config, &mut logger)?;
    let html_converter = HtmlConverter::new();
    let mut file_manager = FileManager::new(&config.output_directory)?;
    
    logger.end_operation("component_initialization", true);
    
    // Determine fetch mode for the domain
    logger.start_operation("domain_detection");
    let fetch_mode = domain_detector.get_fetch_mode_safe(domain)
        .map_err(|e| {
            logger.log_error_with_recovery(&e, false, None);
            e
        })?;
    
    let mode_str = match fetch_mode {
        FetchMode::Chrome => "Chrome",
        FetchMode::HttpRequest => "HttpRequest",
    };
    
    logger.log_domain_detection(domain, mode_str, true);
    logger.end_operation("domain_detection", true);
    
    info!("🎯 Selected fetch mode for {}: {:?} (delay: {}ms, timeout: {}s)", 
          domain, fetch_mode, config.spider_config.delay_ms, config.spider_config.timeout_seconds);
    
    let mut stats = CrawlingStats::default();

    // Step 1: Extract URLs from robots.txt and sitemaps
    logger.start_operation("url_extraction");
    let sitemap_urls = match extract_sitemap_urls_with_recovery(domain, &mut logger).await {
        Ok(urls) => {
            stats.total_urls = urls.len();
            info!("📋 Found {} URLs to process", stats.total_urls);
            logger.end_operation("url_extraction", true);
            urls
        }
        Err(e) => {
            logger.log_error_with_recovery(&e, true, None);
            logger.end_operation("url_extraction", false);
            
            // Fallback to spider native crawl
            warn!("🔄 Falling back to spider native crawl");
            logger.start_operation("spider_fallback");
            
            match super::robots::crawl_with_spider(domain).await {
                Ok(()) => {
                    logger.end_operation("spider_fallback", true);
                    logger.log_final_summary();
                    return Ok(stats);
                }
                Err(fallback_error) => {
                    logger.end_operation("spider_fallback", false);
                    error!("❌ Spider fallback also failed: {:?}", fallback_error);
                    logger.log_final_summary();
                    return Err(CrawlerError::Spider(crate::crawler::errors::SpiderError::RequestFailed(
                        format!("Both sitemap extraction and spider fallback failed: {}", fallback_error)
                    )));
                }
            }
        }
    };

    // Step 2: Process URLs through the complete pipeline
    if !sitemap_urls.is_empty() {
        logger.start_operation("html_processing_pipeline");
        info!("🔄 Starting HTML processing pipeline for {} URLs", sitemap_urls.len());
        
        let processing_results = process_urls_pipeline_with_recovery(
            sitemap_urls,
            fetch_mode,
            &html_converter,
            &mut file_manager,
            &config,
            &mut logger,
        ).await?;
        
        logger.end_operation("html_processing_pipeline", true);
        
        // Update statistics and log results
        logger.start_operation("statistics_compilation");
        for result in processing_results {
            if result.success {
                stats.successful_conversions += 1;
                if result.file_path.is_some() {
                    stats.files_saved += 1;
                }
            } else {
                stats.failed_conversions += 1;
                if let Some(error) = &result.error {
                    warn!("⚠️  Failed to process {}: {}", result.url, error);
                }
            }
        }
        logger.end_operation("statistics_compilation", true);
    }

    // Log performance metrics
    let total_duration = start_time.elapsed();
    let performance_metrics = PerformanceMetrics::new(total_duration, stats.total_urls)
        .with_bytes_processed(0); // Could be enhanced to track actual bytes
    
    logger.log_performance_metrics("complete_crawler_session", &performance_metrics);
    
    // Log final statistics and summary
    info!("🏁 Crawling completed. Stats: {:?}", stats);
    logger.log_final_summary();
    
    Ok(stats)
}

/// Load crawler configuration from file or use default
fn load_crawler_config(config_path: Option<&str>, logger: &mut CrawlerLogger) -> CrawlerResult<CrawlerConfig> {
    let path = config_path.unwrap_or("config/crawler.yaml");
    
    logger.start_operation("config_loading");
    
    let config = CrawlerConfig::load_or_default(path);
    
    // Validate configuration
    match config.validate() {
        Ok(()) => {
            logger.log_configuration("load_and_validate", Some(path), true, None);
            logger.end_operation("config_loading", true);
            info!("⚙️  Configuration loaded and validated successfully");
            Ok(config)
        }
        Err(validation_error) => {
            let error_msg = format!("Configuration validation failed: {}", validation_error);
            logger.log_configuration("load_and_validate", Some(path), false, Some(&error_msg));
            logger.end_operation("config_loading", false);
            
            Err(CrawlerError::Configuration(
                crate::crawler::errors::ConfigurationError::ValidationFailed(error_msg)
            ))
        }
    }
}

/// Initialize domain detector with configuration
fn initialize_domain_detector_with_config(config: &CrawlerConfig, logger: &mut CrawlerLogger) -> CrawlerResult<DomainDetector> {
    logger.start_operation("domain_detector_init");
    
    let mut detector = DomainDetector::new();
    
    // Add domains from configuration with validation
    let mut spa_added = 0;
    let mut ssr_added = 0;
    
    for domain in &config.spa_domains {
        if domain.is_empty() {
            warn!("⚠️  Skipping empty SPA domain in configuration");
            continue;
        }
        detector.add_spa_domain(domain.clone());
        spa_added += 1;
    }
    
    for domain in &config.ssr_domains {
        if domain.is_empty() {
            warn!("⚠️  Skipping empty SSR domain in configuration");
            continue;
        }
        detector.add_ssr_domain(domain.clone());
        ssr_added += 1;
    }
    
    info!("🎯 Initialized domain detector with {} SPA domains and {} SSR domains", 
          spa_added, ssr_added);
    debug!("SPA domains: {:?}", config.spa_domains);
    debug!("SSR domains: {:?}", config.ssr_domains);
    
    logger.end_operation("domain_detector_init", true);
    Ok(detector)
}

/// Initialize domain detector with configuration (legacy function for backward compatibility)
fn initialize_domain_detector() -> CrawlerResult<DomainDetector> {
    let mut logger = CrawlerLogger::new();
    let config = load_crawler_config(None, &mut logger)?;
    initialize_domain_detector_with_config(&config, &mut logger)
}

/// Extract sitemap URLs with comprehensive error recovery
async fn extract_sitemap_urls_with_recovery(domain: &str, logger: &mut CrawlerLogger) -> CrawlerResult<Vec<String>> {
    let mut retry_count = 0;
    let max_retries = 3;
    
    while retry_count < max_retries {
        match extract_sitemap_urls(domain, logger).await {
            Ok(urls) => return Ok(urls),
            Err(e) => {
                retry_count += 1;
                
                if ErrorRecovery::is_recoverable(&e) && retry_count < max_retries {
                    let delay = ErrorRecovery::get_retry_delay(&e, retry_count - 1);
                    warn!("🔄 Sitemap extraction attempt {} failed, retrying in {}ms: {}", 
                          retry_count, delay, e);
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    logger.log_error_with_recovery(&e, true, None);
                } else {
                    error!("❌ Sitemap extraction failed after {} attempts: {}", retry_count, e);
                    logger.log_error_with_recovery(&e, true, Some(false));
                    return Err(e);
                }
            }
        }
    }
    
    unreachable!()
}

/// Extract sitemap URLs from robots.txt and direct sitemap access
async fn extract_sitemap_urls(domain: &str, logger: &mut CrawlerLogger) -> CrawlerResult<Vec<String>> {
    info!("🔍 Extracting sitemap URLs for domain: {}", domain);
    logger.log_spider_activity("robots_txt_fetch", domain, true, Some("Starting robots.txt extraction"));
    
    // Try robots.txt first
    match super::robots::get_sitemaps_from_robots(domain).await {
        Ok(sitemaps) => {
            if sitemaps.is_empty() {
                logger.log_spider_activity("robots_txt_fetch", domain, true, Some("No sitemaps in robots.txt"));
                info!("📋 No sitemap URLs found in robots.txt, trying direct sitemap access");
                
                // Try direct sitemap.xml access
                logger.log_spider_activity("direct_sitemap_fetch", domain, true, Some("Starting direct sitemap.xml fetch"));
                match super::robots::fetch_sitemap_direct(domain).await {
                    Ok(direct_sitemaps) => {
                        if direct_sitemaps.is_empty() {
                            let error_msg = format!("No sitemap.xml found at {}/sitemap.xml", domain);
                            logger.log_spider_activity("direct_sitemap_fetch", domain, false, Some(&error_msg));
                            warn!("⚠️  {}", error_msg);
                            
                            Err(CrawlerError::Spider(
                                crate::crawler::errors::SpiderError::SitemapParsingError(
                                    format!("No sitemaps found for domain: {}", domain)
                                )
                            ))
                        } else {
                            logger.log_spider_activity("direct_sitemap_fetch", domain, true, 
                                Some(&format!("Found {} URLs", direct_sitemaps.len())));
                            info!("✅ Found {} sitemap URLs from direct access", direct_sitemaps.len());
                            Ok(direct_sitemaps)
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to load direct sitemap.xml: {:?}", e);
                        logger.log_spider_activity("direct_sitemap_fetch", domain, false, Some(&error_msg));
                        error!("❌ {}", error_msg);
                        
                        Err(CrawlerError::Spider(
                            crate::crawler::errors::SpiderError::SitemapParsingError(error_msg)
                        ))
                    }
                }
            } else {
                logger.log_spider_activity("robots_txt_fetch", domain, true, 
                    Some(&format!("Found {} sitemap URLs", sitemaps.len())));
                info!("✅ Found {} sitemap URLs from robots.txt", sitemaps.len());
                Ok(sitemaps)
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to load robots.txt: {:?}", e);
            logger.log_spider_activity("robots_txt_fetch", domain, false, Some(&error_msg));
            warn!("⚠️  {}, trying direct sitemap access", error_msg);
            
            // Fallback to direct sitemap access
            logger.log_spider_activity("direct_sitemap_fallback", domain, true, Some("Starting fallback sitemap fetch"));
            match super::robots::fetch_sitemap_direct(domain).await {
                Ok(sitemaps) => {
                    if sitemaps.is_empty() {
                        let fallback_error = format!("No sitemap.xml found at {}/sitemap.xml", domain);
                        logger.log_spider_activity("direct_sitemap_fallback", domain, false, Some(&fallback_error));
                        error!("❌ {}", fallback_error);
                        
                        Err(CrawlerError::Spider(
                            crate::crawler::errors::SpiderError::SitemapParsingError(
                                format!("No sitemaps found for domain: {} (robots.txt and direct access failed)", domain)
                            )
                        ))
                    } else {
                        logger.log_spider_activity("direct_sitemap_fallback", domain, true, 
                            Some(&format!("Found {} URLs (robots.txt fallback)", sitemaps.len())));
                        info!("✅ Found {} sitemap URLs from direct access (robots.txt fallback)", sitemaps.len());
                        Ok(sitemaps)
                    }
                }
                Err(e2) => {
                    let fallback_error = format!("Failed to load direct sitemap.xml: {:?}", e2);
                    logger.log_spider_activity("direct_sitemap_fallback", domain, false, Some(&fallback_error));
                    error!("❌ {}", fallback_error);
                    
                    Err(CrawlerError::Spider(
                        crate::crawler::errors::SpiderError::SitemapParsingError(fallback_error)
                    ))
                }
            }
        }
    }
}

/// Process URLs through the complete pipeline with comprehensive error recovery
async fn process_urls_pipeline_with_recovery(
    urls: Vec<String>,
    fetch_mode: FetchMode,
    html_converter: &HtmlConverter,
    file_manager: &mut FileManager,
    config: &CrawlerConfig,
    logger: &mut CrawlerLogger,
) -> CrawlerResult<Vec<ProcessingResult>> {
    logger.start_operation("url_processing_pipeline");
    
    let result = process_urls_pipeline(urls, fetch_mode, html_converter, file_manager, config, logger).await;
    
    match &result {
        Ok(results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            logger.end_operation("url_processing_pipeline", true);
            info!("✅ Pipeline completed: {}/{} URLs processed successfully", success_count, results.len());
        }
        Err(e) => {
            logger.end_operation("url_processing_pipeline", false);
            logger.log_error_with_recovery(e, false, None);
        }
    }
    
    result
}

/// Process URLs through the complete pipeline: fetch HTML -> convert to Markdown -> save files
async fn process_urls_pipeline(
    urls: Vec<String>,
    fetch_mode: FetchMode,
    html_converter: &HtmlConverter,
    file_manager: &mut FileManager,
    config: &CrawlerConfig,
    logger: &mut CrawlerLogger,
) -> CrawlerResult<Vec<ProcessingResult>> {
    info!("🔄 Processing {} URLs through HTML->Markdown->File pipeline with {}ms delay", 
          urls.len(), config.spider_config.delay_ms);
    
    // Fetch HTML content from all URLs
    logger.start_operation("html_fetching");
    let html_results = match super::robots::fetch_html_from_urls(urls.clone(), fetch_mode).await {
        Ok(results) => {
            logger.end_operation("html_fetching", true);
            logger.log_spider_activity("bulk_html_fetch", &format!("{} URLs", urls.len()), true, 
                Some(&format!("Fetched {} pages", results.len())));
            results
        }
        Err(e) => {
            logger.end_operation("html_fetching", false);
            let error_msg = format!("Failed to fetch HTML from URLs: {:?}", e);
            logger.log_spider_activity("bulk_html_fetch", &format!("{} URLs", urls.len()), false, Some(&error_msg));
            error!("❌ {}", error_msg);
            
            return Err(CrawlerError::Spider(
                crate::crawler::errors::SpiderError::RequestFailed(error_msg)
            ));
        }
    };
    
    let mut processing_results = Vec::new();
    let total_urls = html_results.len();
    
    logger.start_operation("content_processing");
    info!("📝 Processing {} HTML pages through conversion and file saving", total_urls);
    
    // Process each HTML result through conversion and file saving
    for (index, (url, html_content)) in html_results.into_iter().enumerate() {
        // Log progress
        if index % 10 == 0 || index == total_urls - 1 {
            logger.log_pipeline_progress("content_processing", index + 1, total_urls, None);
        }
        let mut result = ProcessingResult {
            url: url.clone(),
            success: false,
            file_path: None,
            error: None,
        };
        
        debug!("🔍 Processing URL: {} ({} bytes of HTML)", url, html_content.len());
        
        // Convert HTML to Markdown with recovery
        let markdown_content = match html_converter.convert_to_markdown_with_recovery(&html_content) {
            Ok(markdown) => {
                logger.log_html_conversion(&url, html_content.len(), markdown.len(), true);
                debug!("✅ Successfully converted HTML to Markdown for {}", url);
                markdown
            }
            Err(e) => {
                logger.log_html_conversion(&url, html_content.len(), 0, false);
                let error_msg = format!("HTML conversion failed: {}", e);
                error!("❌ {}", error_msg);
                result.error = Some(error_msg.clone());
                
                // Fallback: save raw HTML with .html extension
                logger.log_file_operation("fallback_html_save", &format!("{}.html", url), true, None);
                match save_fallback_html(&url, &html_content, file_manager, logger) {
                    Ok(path) => {
                        warn!("💾 Saved raw HTML as fallback: {:?}", path);
                        result.file_path = Some(path);
                        result.success = true;
                    }
                    Err(fallback_err) => {
                        let fallback_error = format!("Fallback HTML save failed: {}", fallback_err);
                        logger.log_file_operation("fallback_html_save", &format!("{}.html", url), false, Some(&fallback_error));
                        error!("❌ {}", fallback_error);
                        result.error = Some(format!("{}, {}", result.error.unwrap_or_default(), fallback_error));
                    }
                }
                processing_results.push(result);
                continue;
            }
        };
        
        // Save Markdown file
        match file_manager.save_markdown(&url, &markdown_content) {
            Ok(file_path) => {
                logger.log_file_operation("markdown_save", &file_path.to_string_lossy(), true, None);
                info!("💾 Successfully saved Markdown file: {:?}", file_path);
                result.success = true;
                result.file_path = Some(file_path);
            }
            Err(e) => {
                let error_msg = format!("File save failed: {}", e);
                logger.log_file_operation("markdown_save", &url, false, Some(&error_msg));
                error!("❌ {}", error_msg);
                result.error = Some(error_msg);
                
                // Log error for potential recovery
                logger.log_error_with_recovery(&e, false, None);
            }
        }
        
        processing_results.push(result);
    }
    
    logger.end_operation("content_processing", true);
    
    let successful = processing_results.iter().filter(|r| r.success).count();
    let failed = processing_results.len() - successful;
    
    info!("🏁 Pipeline processing completed for {} URLs ({} successful, {} failed)", 
          processing_results.len(), successful, failed);
    
    Ok(processing_results)
}

/// Save raw HTML as fallback when Markdown conversion fails
fn save_fallback_html(
    url: &str,
    html_content: &str,
    _file_manager: &mut FileManager,
    logger: &mut CrawlerLogger,
) -> CrawlerResult<std::path::PathBuf> {
    trace!("Saving fallback HTML for URL: {} ({} bytes)", url, html_content.len());
    
    // Generate filename similar to FileManager but with .html extension
    let filename = generate_html_filename(url);
    let output_dir = std::path::Path::new("output");
    let file_path = output_dir.join(&filename);
    
    // Ensure output directory exists
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)
            .map_err(|e| {
                logger.log_file_operation("create_fallback_dir", "output", false, Some(&e.to_string()));
                crate::crawler::errors::FileOperationError::DirectoryCreationFailed(e)
            })?;
    }
    
    // Write HTML content with retry logic
    let mut attempts = 0;
    let max_attempts = 3;
    
    while attempts < max_attempts {
        match std::fs::write(&file_path, html_content) {
            Ok(()) => {
                if attempts > 0 {
                    info!("✅ Fallback HTML save succeeded on attempt {}", attempts + 1);
                }
                return Ok(file_path);
            }
            Err(e) => {
                attempts += 1;
                if attempts < max_attempts {
                    warn!("⚠️  Fallback HTML save attempt {} failed: {}. Retrying...", attempts, e);
                    std::thread::sleep(std::time::Duration::from_millis(100 * attempts as u64));
                } else {
                    error!("❌ Fallback HTML save failed after {} attempts: {}", max_attempts, e);
                    return Err(crate::crawler::errors::FileOperationError::FileWriteFailed(e).into());
                }
            }
        }
    }
    
    unreachable!()
}

/// Generate HTML filename from URL (similar to FileManager logic)
fn generate_html_filename(url: &str) -> String {
    use url::Url;
    
    if let Ok(parsed_url) = Url::parse(url) {
        let path = parsed_url.path();
        
        let filename = if path == "/" || path.is_empty() {
            format!("{}_index", parsed_url.host_str().unwrap_or("unknown"))
        } else {
            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            if segments.is_empty() {
                format!("{}_index", parsed_url.host_str().unwrap_or("unknown"))
            } else {
                segments.join("_")
            }
        };
        
        let sanitized = sanitize_html_filename(&filename);
        format!("{}.html", sanitized)
    } else {
        "unknown_page.html".to_string()
    }
}

/// Sanitize filename for HTML files
fn sanitize_html_filename(filename: &str) -> String {
    let mut sanitized = filename
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
            '/' | '\\' => '_',
            c if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' => c,
            _ => '_',
        })
        .collect::<String>();

    if sanitized.starts_with('.') || sanitized.starts_with('-') {
        sanitized = format!("page_{}", sanitized);
    }

    if sanitized.len() > 200 {
        sanitized.truncate(200);
    }

    if sanitized.is_empty() {
        sanitized = "page".to_string();
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_integrated_pipeline_with_mock_data() {
        // Initialize logging for test
        let _ = env_logger::try_init();
        
        // Create temporary directory for test output
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_output");
        
        // Test HTML content
        let test_html = r#"
        <html>
        <head><title>Test Page</title></head>
        <body>
            <h1>Main Title</h1>
            <h2>Subtitle</h2>
            <p>This is a test paragraph with <strong>bold text</strong> and <em>italic text</em>.</p>
            <ul>
                <li>First item</li>
                <li>Second item</li>
            </ul>
            <a href="https://example.com">Example Link</a>
            <img src="test.jpg" alt="Test Image">
            <blockquote>This is a quote</blockquote>
        </body>
        </html>
        "#;
        
        // Initialize components
        let domain_detector = DomainDetector::new();
        let html_converter = HtmlConverter::new();
        let mut file_manager = FileManager::new(output_path.to_str().unwrap()).unwrap();
        
        // Test domain detection
        let fetch_mode = domain_detector.get_fetch_mode("example.com");
        assert_eq!(fetch_mode, FetchMode::HttpRequest); // Should default to HttpRequest
        
        // Test HTML conversion
        let markdown = html_converter.convert_to_markdown(test_html).unwrap();
        assert!(markdown.contains("# Main Title"));
        assert!(markdown.contains("## Subtitle"));
        assert!(markdown.contains("**bold text**"));
        assert!(markdown.contains("*italic text*"));
        assert!(markdown.contains("- First item"));
        assert!(markdown.contains("[Example Link](https://example.com)"));
        assert!(markdown.contains("![Test Image](test.jpg)"));
        assert!(markdown.contains("> This is a quote"));
        
        // Test file saving
        let file_path = file_manager.save_markdown("https://example.com/test", &markdown).unwrap();
        assert!(file_path.exists());
        
        // Verify file content
        let saved_content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(saved_content, markdown);
        
        println!("✅ Integrated pipeline test passed!");
    }

    #[test]
    fn test_processing_result_creation() {
        let result = ProcessingResult {
            url: "https://example.com".to_string(),
            success: true,
            file_path: Some(std::path::PathBuf::from("test.md")),
            error: None,
        };
        
        assert_eq!(result.url, "https://example.com");
        assert!(result.success);
        assert!(result.file_path.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_crawling_stats_default() {
        let stats = CrawlingStats::default();
        assert_eq!(stats.total_urls, 0);
        assert_eq!(stats.successful_conversions, 0);
        assert_eq!(stats.failed_conversions, 0);
        assert_eq!(stats.files_saved, 0);
    }

    #[test]
    fn test_generate_html_filename() {
        let filename = generate_html_filename("https://example.com/docs/getting-started");
        assert_eq!(filename, "docs_getting-started.html");
        
        let filename2 = generate_html_filename("https://example.com/");
        assert_eq!(filename2, "example.com_index.html");
    }

    #[test]
    fn test_sanitize_html_filename() {
        let sanitized = sanitize_html_filename("file<>:\"name|?*");
        assert_eq!(sanitized, "file____name___");
        
        let sanitized2 = sanitize_html_filename(".hidden-file");
        assert_eq!(sanitized2, "page_.hidden-file");
    }
}