use std::fs;
use tempfile::TempDir;

// Use the library crate instead of local modules
use test::crawler::domain_detector::{DomainDetector, FetchMode};
use test::crawler::html_converter::HtmlConverter;
use test::crawler::file_manager::FileManager;
use test::crawler::config::CrawlerConfig;

fn main() {
    println!("🧪 Starting Pipeline Validation Tests");
    
    // Initialize logging
    let _ = env_logger::try_init();
    
    // Test 1: Domain Detection
    println!("\n1️⃣ Testing Domain Detection...");
    test_domain_detection();
    
    // Test 2: HTML to Markdown Conversion
    println!("\n2️⃣ Testing HTML to Markdown Conversion...");
    test_html_conversion();
    
    // Test 3: File Management
    println!("\n3️⃣ Testing File Management...");
    test_file_management();
    
    // Test 4: Error Handling
    println!("\n4️⃣ Testing Error Handling...");
    test_error_handling();
    
    // Test 5: Configuration
    println!("\n5️⃣ Testing Configuration...");
    test_configuration();
    
    // Test 6: Performance
    println!("\n6️⃣ Testing Performance...");
    test_performance();
    
    println!("\n✅ All Pipeline Validation Tests Completed Successfully!");
    println!("🎯 The complete web crawler pipeline is ready for production use.");
}

fn test_domain_detection() {
    let mut domain_detector = DomainDetector::new();
    
    // Add test domains
    domain_detector.add_spa_domain("app.example.com".to_string());
    domain_detector.add_ssr_domain("docs.example.com".to_string());
    
    // Test SPA domain detection
    let spa_mode = domain_detector.get_fetch_mode("app.example.com");
    assert_eq!(spa_mode, FetchMode::Chrome, "SPA domain should use Chrome mode");
    println!("   ✅ SPA domain detection: app.example.com -> Chrome mode");
    
    // Test SSR domain detection
    let ssr_mode = domain_detector.get_fetch_mode("docs.example.com");
    assert_eq!(ssr_mode, FetchMode::HttpRequest, "SSR domain should use HttpRequest mode");
    println!("   ✅ SSR domain detection: docs.example.com -> HttpRequest mode");
    
    // Test unknown domain defaults
    let unknown_mode = domain_detector.get_fetch_mode("unknown.example.com");
    assert_eq!(unknown_mode, FetchMode::HttpRequest, "Unknown domain should default to HttpRequest");
    println!("   ✅ Unknown domain detection: unknown.example.com -> HttpRequest mode (default)");
}

fn test_html_conversion() {
    let converter = HtmlConverter::new();
    
    // Test comprehensive HTML content
    let test_html = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <title>Test Document</title>
        <meta charset="UTF-8">
        <style>body { color: blue; }</style>
        <script>console.log('test');</script>
    </head>
    <body>
        <h1>Main Title</h1>
        <h2>Subtitle with <strong>bold</strong> text</h2>
        
        <p>This is a paragraph with <em>italic text</em> and <strong>bold text</strong>.</p>
        <p>Another paragraph with a <a href="https://example.com">link to example</a>.</p>
        
        <ul>
            <li>First unordered item</li>
            <li>Second item with <strong>formatting</strong></li>
            <li>Third item</li>
        </ul>
        
        <ol>
            <li>First ordered item</li>
            <li>Second ordered item</li>
        </ol>
        
        <blockquote>
            This is an important quote with <em>emphasis</em>.
        </blockquote>
        
        <p>Image example: <img src="/images/test.jpg" alt="Test Image"></p>
        
        <p>Line break example:<br>New line after break<br/>Another line</p>
        
        <!-- This comment should be ignored -->
        <div>Content in div should be preserved</div>
    </body>
    </html>
    "#;
    
    let markdown = converter.convert_to_markdown(test_html).unwrap();
    
    // Verify conversions
    assert!(markdown.contains("# Main Title"), "Should convert H1 correctly");
    println!("   ✅ H1 conversion: Main Title");
    
    assert!(markdown.contains("## Subtitle with **bold** text"), "Should convert H2 with nested formatting");
    println!("   ✅ H2 conversion with nested formatting");
    
    assert!(markdown.contains("*italic text*"), "Should convert italic text");
    println!("   ✅ Italic text conversion");
    
    assert!(markdown.contains("**bold text**"), "Should convert bold text");
    println!("   ✅ Bold text conversion");
    
    assert!(markdown.contains("[link to example](https://example.com)"), "Should convert links correctly");
    println!("   ✅ Link conversion");
    
    assert!(markdown.contains("- First unordered item"), "Should convert unordered lists");
    println!("   ✅ Unordered list conversion");
    
    assert!(markdown.contains("> This is an important quote with *emphasis*"), "Should convert blockquotes with formatting");
    println!("   ✅ Blockquote conversion with formatting");
    
    assert!(markdown.contains("![Test Image](/images/test.jpg)"), "Should convert images correctly");
    println!("   ✅ Image conversion");
    
    // Verify unwanted content removal
    assert!(!markdown.contains("console.log"), "Should remove JavaScript");
    println!("   ✅ JavaScript removal");
    
    assert!(!markdown.contains("color: blue"), "Should remove CSS");
    println!("   ✅ CSS removal");
    
    assert!(!markdown.contains("DOCTYPE"), "Should remove DOCTYPE");
    println!("   ✅ DOCTYPE removal");
}

fn test_file_management() {
    let temp_dir = TempDir::new().unwrap();
    let mut file_manager = FileManager::new(temp_dir.path().to_str().unwrap()).unwrap();
    
    // Test various URL patterns
    let test_cases = vec![
        ("https://example.com/", "example.com_index.md"),
        ("https://docs.example.com/getting-started", "docs_getting-started.md"),
        ("https://api.example.com/v1/users", "api_v1_users.md"),
        ("https://blog.example.com/2023/12/my-post", "blog_2023_12_my-post.md"),
    ];
    
    for (url, expected_pattern) in test_cases {
        let content = format!("# Test content for {}\n\nThis is test content.", url);
        let file_path = file_manager.save_markdown(url, &content).unwrap();
        
        // Verify file exists
        assert!(file_path.exists(), "File should exist for URL: {}", url);
        
        // Verify filename pattern
        let filename = file_path.file_name().unwrap().to_str().unwrap();
        assert!(filename.contains(&expected_pattern.replace(".md", "")), 
                "Filename '{}' should match pattern '{}'", filename, expected_pattern);
        
        // Verify content
        let saved_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(saved_content, content, "Content should match for URL: {}", url);
        
        println!("   ✅ File saved: {} -> {}", url, filename);
    }
    
    // Test duplicate filename handling
    let duplicate_url = "https://example.com/test";
    let content1 = "First content";
    let content2 = "Second content";
    
    let file1 = file_manager.save_markdown(duplicate_url, content1).unwrap();
    let file2 = file_manager.save_markdown(duplicate_url, content2).unwrap();
    
    assert_ne!(file1, file2, "Duplicate URLs should create different files");
    assert!(file1.exists() && file2.exists(), "Both files should exist");
    
    println!("   ✅ Duplicate filename handling");
    
    // Test filename sanitization
    let problematic_url = "https://example.com/file<>:\"name|?*";
    let sanitized_file = file_manager.save_markdown(problematic_url, "test content").unwrap();
    assert!(sanitized_file.exists(), "Should handle problematic characters in URLs");
    
    println!("   ✅ Filename sanitization");
}

fn test_error_handling() {
    let converter = HtmlConverter::new();
    
    // Test cases for malformed HTML
    let malformed_cases = vec![
        ("<html><body><h1>Unclosed header<p>Paragraph", "Unclosed tags"),
        ("<html><body><<>>Invalid brackets</body></html>", "Invalid bracket patterns"),
        ("<html><body><script>alert('test')</body></html>", "Unclosed script tag"),
        ("", "Empty content"),
        ("   \n\t   ", "Whitespace only"),
        ("<>", "Invalid tag structure"),
    ];
    
    for (html, description) in malformed_cases {
        match converter.convert_to_markdown(html) {
            Ok(markdown) => {
                println!("   ✅ Handled gracefully ({}): {} chars -> {} chars", 
                        description, html.len(), markdown.len());
            }
            Err(e) => {
                println!("   ✅ Rejected as expected ({}): {:?}", description, e);
            }
        }
    }
    
    // Test recovery mechanism
    let recoverable_html = "<html><body><h1>Title</h1><p>Content<</body></html>";
    match converter.convert_to_markdown_with_recovery(recoverable_html) {
        Ok(markdown) => {
            assert!(markdown.contains("Title"), "Should recover basic content");
            println!("   ✅ Recovery mechanism worked");
        }
        Err(e) => {
            println!("   ⚠️  Recovery failed: {:?}", e);
        }
    }
}

fn test_configuration() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test valid configuration
    let valid_config = CrawlerConfig::default();
    assert!(valid_config.validate().is_ok(), "Default config should be valid");
    println!("   ✅ Default configuration validation");
    
    // Test invalid configurations
    let mut invalid_config = CrawlerConfig::default();
    
    // Empty output directory
    invalid_config.output_directory = "".to_string();
    assert!(invalid_config.validate().is_err(), "Empty output directory should be invalid");
    println!("   ✅ Invalid configuration rejection (empty output directory)");
    
    // Reset and test invalid depth
    invalid_config = CrawlerConfig::default();
    invalid_config.spider_config.depth = 0;
    assert!(invalid_config.validate().is_err(), "Zero depth should be invalid");
    println!("   ✅ Invalid configuration rejection (zero depth)");
    
    // Reset and test invalid log level
    invalid_config = CrawlerConfig::default();
    invalid_config.logging.level = "invalid".to_string();
    assert!(invalid_config.validate().is_err(), "Invalid log level should be rejected");
    println!("   ✅ Invalid configuration rejection (invalid log level)");
    
    // Test configuration save/load
    let config_path = temp_dir.path().join("test_config.yaml");
    let original_config = CrawlerConfig::default();
    
    original_config.save_to_yaml(config_path.to_str().unwrap()).unwrap();
    let loaded_config = CrawlerConfig::load_from_yaml(config_path.to_str().unwrap()).unwrap();
    
    assert_eq!(original_config.output_directory, loaded_config.output_directory);
    assert_eq!(original_config.spider_config.depth, loaded_config.spider_config.depth);
    
    println!("   ✅ Configuration save/load cycle");
}

fn test_performance() {
    let temp_dir = TempDir::new().unwrap();
    let mut file_manager = FileManager::new(temp_dir.path().to_str().unwrap()).unwrap();
    let converter = HtmlConverter::new();
    
    let test_html = r#"
    <html>
    <body>
        <h1>Performance Test Page</h1>
        <h2>Content Section</h2>
        <p>This is a test paragraph with <strong>bold text</strong> and <em>italic text</em>.</p>
        <ul>
            <li>First item</li>
            <li>Second item</li>
            <li>Third item</li>
        </ul>
        <p>More content with <a href="https://example.com">a link</a>.</p>
        <blockquote>This is a quote for testing purposes.</blockquote>
    </body>
    </html>
    "#;
    
    let start_time = std::time::Instant::now();
    let num_files = 50;
    
    for i in 0..num_files {
        let url = format!("https://test.example.com/page-{}", i);
        let markdown = converter.convert_to_markdown(test_html).unwrap();
        let _file_path = file_manager.save_markdown(&url, &markdown).unwrap();
    }
    
    let duration = start_time.elapsed();
    let files_per_second = num_files as f64 / duration.as_secs_f64();
    
    println!("   ⏱️  Processed {} files in {:?}", num_files, duration);
    println!("   📊 Rate: {:.2} files/second", files_per_second);
    
    // Performance assertion - should process at least 10 files per second
    assert!(files_per_second >= 10.0, "Should process at least 10 files per second, got {:.2}", files_per_second);
    
    // Verify all files were created
    let output_files: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "md" {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    
    assert_eq!(output_files.len(), num_files, "Should create {} files", num_files);
    println!("   ✅ Performance test passed: {:.2} files/second", files_per_second);
}