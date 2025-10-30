# Requirements Document

## Introduction

Complete the development of a comprehensive web crawler using spider-rs library in Rust. The crawler should fetch HTML content from websites and convert it to Markdown format, with support for both SPA (Single Page Application) and SSR (Server Side Rendered) websites through automatic domain detection and appropriate fetching strategies.

## Glossary

- **Spider_Crawler**: The main web crawler system built with spider-rs library
- **SPA_Mode**: Single Page Application mode using Chrome browser for JavaScript rendering
- **SSR_Mode**: Server Side Rendered mode using HTTP requests for static content
- **Domain_Detector**: Component that determines whether a domain requires SPA or SSR mode
- **HTML_Converter**: Component that transforms HTML content into Markdown format
- **File_Manager**: Component that saves Markdown files with appropriate naming conventions
- **Sitemap_Parser**: Component that processes sitemap XML files recursively

## Requirements

### Requirement 1

**User Story:** As a developer, I want the crawler to automatically detect whether a website is SPA or SSR based on the domain, so that it uses the appropriate fetching method.

#### Acceptance Criteria

1. WHEN the Spider_Crawler receives a domain URL, THE Domain_Detector SHALL determine the appropriate fetch mode based on a predefined domain configuration
2. WHERE a domain is configured as SPA, THE Spider_Crawler SHALL use Chrome browser mode for HTML fetching
3. WHERE a domain is configured as SSR, THE Spider_Crawler SHALL use HTTP request mode for HTML fetching
4. THE Domain_Detector SHALL support an auto-mode that selects fetch mode automatically based on domain patterns
5. THE Spider_Crawler SHALL maintain a whitelist of known SPA domains for accurate mode selection

### Requirement 2

**User Story:** As a developer, I want the crawler to convert HTML content to readable Markdown format, so that the content can be easily processed and stored.

#### Acceptance Criteria

1. WHEN the HTML_Converter receives HTML content, THE HTML_Converter SHALL parse and convert supported HTML tags to Markdown format
2. THE HTML_Converter SHALL support conversion of h1, h2, p, ul, li, ol, a, img, strong, em, br, and blockquote tags
3. THE HTML_Converter SHALL produce readable Markdown output with proper formatting and structure
4. THE HTML_Converter SHALL use only libraries available within spider-rs for HTML parsing and conversion
5. THE HTML_Converter SHALL preserve link URLs and image sources in the Markdown output

### Requirement 3

**User Story:** As a developer, I want the crawler to save each page as a separate Markdown file with meaningful names, so that the content is organized and easily accessible.

#### Acceptance Criteria

1. WHEN the File_Manager receives Markdown content and a source URL, THE File_Manager SHALL generate a filename based on the URL path or slug
2. THE File_Manager SHALL save Markdown files in an "output/" directory with .md extension
3. THE File_Manager SHALL sanitize filenames to remove invalid characters and ensure filesystem compatibility
4. WHERE a URL path contains meaningful segments, THE File_Manager SHALL use those segments for the filename
5. THE File_Manager SHALL handle duplicate filenames by appending appropriate suffixes

### Requirement 4

**User Story:** As a developer, I want the crawler to integrate all components into a single pipeline, so that I can crawl any website with a simple function call.

#### Acceptance Criteria

1. THE Spider_Crawler SHALL execute the complete pipeline: robots.txt → sitemap → URL extraction → mode detection → HTML fetching → Markdown conversion → file saving
2. WHEN the Spider_Crawler processes sitemap URLs, THE Sitemap_Parser SHALL recursively parse nested sitemaps and extract all page URLs
3. THE Spider_Crawler SHALL handle fallback scenarios when robots.txt or sitemaps are unavailable
4. THE Spider_Crawler SHALL process multiple URLs efficiently while respecting rate limiting
5. THE Spider_Crawler SHALL provide clear logging and error handling throughout the pipeline

### Requirement 5

**User Story:** As a developer, I want the crawler to work with multiple websites generically, so that it's not tied to any specific domain or website structure.

#### Acceptance Criteria

1. THE Spider_Crawler SHALL work with any valid domain URL without hardcoded site-specific logic
2. THE Spider_Crawler SHALL handle various sitemap formats and structures commonly found across different websites
3. THE Domain_Detector SHALL be configurable to add new domains without code changes
4. THE Spider_Crawler SHALL gracefully handle websites with different robots.txt formats and sitemap structures
5. THE Spider_Crawler SHALL use only the spider-rs library without external dependencies for web scraping functionality