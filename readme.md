## Installation

### Prerequisites

- Rust 1.70+ (2024 edition)
- Chrome/Chromium browser (for SPA crawling)

### Build

```bash
cargo build --release
```

## Usage

### Basic Usage

```bash
# Crawl a website (auto-detects SPA/SSR mode)
cargo run -- "https://www.example.com"

# Force Chrome mode for SPA sites
cargo run -- "https://react-app.com"

# Force HttpRequest mode for SSR sites
cargo run -- "https://ssr-site.com"
```

### Test Cases

```bash
# Case 1: Website with robots.txt
cargo run -- "https://www.muangthai.co.th/th"

# Case 2: Direct sitemap.xml testing
cargo run -- "https://thewhitemarketing.com/"

# Case 3: Native crawler mode
cargo run -- "https://www.rust-lang.org/"

# SSR Mode
cargo run -- "https://thewhitemarketing.com/"

# SPA Mode
cargo run -- "https://leerob.com/"
```


### App Configuration (`src/config/app.yaml`)

```yaml
depth: 1                          # Crawling depth
user_agent: "SSS/1.0"            # User agent string
delay_ms: 50                     # Delay between requests (ms)
max_pages: 5                     # Maximum pages to crawl
fetch_mode: "Chrome"             # Force fetch mode: "Chrome" or "HttpRequest"
whitelist_path: "src/config/whitelist.yaml"  # Path to domain whitelist
sitemap_max_depth: 5             # Max sitemap nesting depth
max_sitemap_urls: 5              # Max URLs to extract from sitemaps
```

### Domain Whitelist (`src/config/whitelist.yaml`)

Configure how different domains are handled:

```yaml
auto_mode: true          # Enable automatic mode detection
default_mode: "SPA"      # Default mode for unknown domains
match_on: "domain_only"  # Match criteria

whitelist:
  - domain: "www.heygoody.com"
    mode: "SPA"
    handler: "chrome"
    match: "exact"

  - domain: "reactjs.org"
    mode: "SPA"
    handler: "chrome"
    match: "subdomain"
```

### Project Structure

```
src/
├── main.rs              # Entry point
├── config/
│   ├── mod.rs
│   ├── config.rs        # Configuration structs
│   ├── app.yaml         # App configuration
│   └── whitelist.yaml   # Domain whitelist
└── crawler/
    ├── mod.rs
    ├── crawler.rs       # Main crawling logic
    ├── robots.rs        # Robots.txt processing
    ├── html_fetcher.rs  # Fetch mode dispatcher
    ├── chrome_fetcher.rs # Chrome-based fetching
    ├── html_to_markdown.rs # HTML to Markdown conversion
    ├── markdown_writer.rs # File writing
    └── domain_detector.rs # Domain classification
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

## Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `depth` | Crawling depth | 1 |
| `user_agent` | HTTP user agent | "SSS/1.0" |
| `delay_ms` | Delay between requests | 50 |
| `max_pages` | Maximum pages to crawl | 5 |
| `fetch_mode` | Force fetch mode | "Chrome" |
| `sitemap_max_depth` | Max sitemap nesting | 5 |
| `max_sitemap_urls` | Max URLs from sitemaps | 5 |

