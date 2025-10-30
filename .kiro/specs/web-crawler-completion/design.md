# Design Document

## Overview

The web crawler completion focuses on adding the missing components to transform the existing spider-rs based crawler into a complete HTML-to-Markdown conversion system. The design builds upon the existing robots.txt and sitemap parsing functionality, adding domain-based SPA/SSR detection, HTML-to-Markdown conversion, and file management capabilities.

## Architecture

The system follows a modular pipeline architecture with the following flow:

```
Domain URL → Domain Detection → HTML Fetching → HTML Conversion → File Saving
     ↑              ↓               ↓              ↓           ↓
Existing Pipeline → Mode Selection → Content → Markdown → Output Files
```

### Core Components

1. **Domain Detector Module** (`src/crawler/domain_detector.rs`)
   - Maintains domain-to-mode mapping
   - Provides auto-detection logic
   - Configurable through external configuration

2. **HTML Converter Module** (`src/crawler/html_converter.rs`)
   - Parses HTML using spider-rs built-in capabilities
   - Converts supported tags to Markdown
   - Handles text formatting and structure preservation

3. **File Manager Module** (`src/crawler/file_manager.rs`)
   - Generates sanitized filenames from URLs
   - Creates output directory structure
   - Handles file writing and duplicate resolution

4. **Enhanced Crawler Module** (existing `src/crawler/crawler.rs`)
   - Integrates new components into existing pipeline
   - Coordinates the complete crawling workflow

## Components and Interfaces

### Domain Detector

```rust
pub struct DomainDetector {
    spa_domains: HashSet<String>,
    ssr_domains: HashSet<String>,
}

impl DomainDetector {
    pub fn new() -> Self
    pub fn is_spa_domain(&self, domain: &str) -> bool
    pub fn get_fetch_mode(&self, domain: &str) -> FetchMode
    pub fn add_spa_domain(&mut self, domain: String)
    pub fn add_ssr_domain(&mut self, domain: String)
}

pub enum FetchMode {
    HttpRequest,
    Chrome,
}
```

### HTML Converter

```rust
pub struct HtmlConverter;

impl HtmlConverter {
    pub fn new() -> Self
    pub fn convert_to_markdown(&self, html: &str) -> Result<String, ConversionError>
    fn parse_html_tags(&self, content: &str) -> String
    fn convert_headings(&self, content: &str) -> String
    fn convert_paragraphs(&self, content: &str) -> String
    fn convert_lists(&self, content: &str) -> String
    fn convert_links(&self, content: &str) -> String
    fn convert_images(&self, content: &str) -> String
    fn convert_formatting(&self, content: &str) -> String
}
```

### File Manager

```rust
pub struct FileManager {
    output_dir: PathBuf,
}

impl FileManager {
    pub fn new(output_dir: &str) -> Result<Self, std::io::Error>
    pub fn save_markdown(&self, url: &str, content: &str) -> Result<PathBuf, std::io::Error>
    fn generate_filename(&self, url: &str) -> String
    fn sanitize_filename(&self, filename: &str) -> String
    fn ensure_unique_filename(&self, base_path: &Path) -> PathBuf
}
```

## Data Models

### Configuration Structure

```rust
#[derive(Debug, Deserialize)]
pub struct CrawlerConfig {
    pub spa_domains: Vec<String>,
    pub ssr_domains: Vec<String>,
    pub output_directory: String,
    pub spider_config: SpiderConfig,
}

#[derive(Debug)]
pub struct ConversionResult {
    pub url: String,
    pub markdown_content: String,
    pub file_path: PathBuf,
}
```

### Error Types

```rust
#[derive(Debug)]
pub enum CrawlerError {
    DomainDetectionError(String),
    HtmlConversionError(String),
    FileOperationError(std::io::Error),
    SpiderError(Box<dyn std::error::Error>),
}
```

## Error Handling

### Error Recovery Strategy

1. **Domain Detection Failures**: Default to HttpRequest mode for unknown domains
2. **HTML Conversion Failures**: Save raw HTML with .html extension and log conversion errors
3. **File Operation Failures**: Retry with alternative filenames, fallback to timestamp-based naming
4. **Spider Fetching Failures**: Continue with remaining URLs, log failed attempts

### Logging Strategy

- Use structured logging with different levels (INFO, WARN, ERROR)
- Log domain detection decisions for transparency
- Track conversion statistics (success/failure rates)
- Monitor file operations and disk usage

## Testing Strategy

### Unit Testing Approach

1. **Domain Detector Tests**
   - Test known SPA/SSR domain classification
   - Verify fallback behavior for unknown domains
   - Test configuration loading and domain management

2. **HTML Converter Tests**
   - Test individual tag conversion functions
   - Verify complex HTML structure handling
   - Test edge cases (malformed HTML, nested tags)
   - Validate Markdown output formatting

3. **File Manager Tests**
   - Test filename generation from various URL patterns
   - Verify filename sanitization for different operating systems
   - Test duplicate filename resolution
   - Verify directory creation and file writing

4. **Integration Tests**
   - Test complete pipeline with sample websites
   - Verify SPA vs SSR mode selection and execution
   - Test error handling and recovery scenarios
   - Validate end-to-end Markdown file generation

### Test Data Strategy

- Create mock HTML samples representing different website structures
- Use local test servers for integration testing
- Maintain test fixtures for various sitemap formats
- Include edge cases like empty pages, JavaScript-heavy content, and malformed HTML

### Performance Considerations

1. **Memory Management**: Process URLs in batches to avoid memory exhaustion
2. **Rate Limiting**: Respect existing delay configurations in spider
3. **Concurrent Processing**: Leverage tokio for async operations while maintaining order
4. **File I/O Optimization**: Batch file operations where possible

### Configuration Management

The system will use a YAML configuration file (`config/crawler.yaml`) for:

```yaml
spa_domains:
  - "www.heygoody.com"
  - "app.example.com"

ssr_domains:
  - "www.rust-lang.org"
  - "docs.rs"

output_directory: "output"

spider_config:
  depth: 3
  delay_ms: 200
  user_agent: "MyRustCrawler/1.0"
```

This design ensures the crawler remains generic and configurable while providing robust HTML-to-Markdown conversion capabilities using only the spider-rs library ecosystem.