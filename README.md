# 🕷️ Rust Web Scraper

A powerful, intelligent web scraping tool with **automatic content detection**, modern web interface, and RESTful API. Built with Rust, Actix-Web, and designed for both browser and desktop (Tauri) use.

## ✨ Features

- 🤖 **Automatic Selector Detection** - Scrape any website without manual configuration
- 🌐 **Modern Web Interface** - Beautiful, intuitive dashboard accessible via browser
- 🔌 **RESTful API** - Complete API for integration into other applications
- 🔄 **Smart Pagination** - Automatically follows "Next" links across multiple pages
- ⚡ **Async & Concurrent** - Fast, non-blocking operations powered by Tokio
- 🛡️ **Polite Scraping** - Configurable rate limiting and user agent rotation
- 📊 **Rich Data Extraction**:
  - Titles and headings
  - Content paragraphs and articles
  - Links (with external/internal detection)
  - Images with metadata
  - Page metadata and SEO data
- 🐳 **Docker Support** - Ready for containerized deployment
- 💻 **Tauri Ready** - Can be wrapped as a cross-platform desktop app
- 📦 **Library Mode** - Use as a Rust crate in your own projects
- 🎨 **Clean Architecture** - Well-organized, maintainable code structure

## Installation

```bash
# Clone the repository
cd rust-web-scraper

# Build the project
cargo build --release
```

## Usage

### 1. Default Example
Run without arguments to scrape example.com:
```bash
cargo run
```

### 2. Scrape Single URL
```bash
cargo run -- -u https://example.com
```

### 3. Scrape Multiple URLs Concurrently
```bash
cargo run -- -u https://example.com -u https://www.rust-lang.org
```

### 4. Save Output to Files
```bash
cargo run -- -u https://example.com --save
```

### 5. Custom Output Directory
```bash
cargo run -- -u https://example.com --save --output my-data
```

### 6. Use Configuration File
```bash
cargo run -- --config config.toml --save
```

### 7. Verbose Mode
```bash
cargo run -- -u https://example.com --verbose
```

### 8. Combined Options
```bash
cargo run -- -u https://example.com -u https://www.rust-lang.org --save --output results --verbose
```

### 9. Enable Pagination (Follow "Next" Links)
```bash
cargo run -- -u https://books.toscrape.com/catalogue/category/books/mystery_3/index.html --paginate --save
```

### 10. Pagination with Page Limit
```bash
cargo run -- -u https://books.toscrape.com --paginate -m 10 --save --verbose
```

### 11. Pagination via Configuration File
```bash
cargo run -- --config books-config.toml --save
```

## Configuration File

Create a `config.toml` file to define URLs and custom selectors:

```toml
# List of URLs to scrape
urls = [
    "https://example.com",
    "https://www.rust-lang.org",
]

# CSS Selectors Configuration
[selectors]
title = "h1"              # Main title selector
content = "p"             # Content paragraphs selector
links = "a"               # Links selector (optional)
images = "img"            # Images selector (optional)
```

### Custom Selector Examples

#### For Blogs
```toml
[selectors]
title = "h1.post-title"
content = "div.post-content p"
links = "article a"
images = "article img"
```

#### For News Sites
```toml
[selectors]
title = "h1.article-headline"
content = "div.article-body p"
links = "article a[href]"
images = "figure img"
```

#### For Documentation
```toml
[selectors]
title = "h1#page-title"
content = "div.content p, div.content li"
links = "nav a, article a"
images = "div.content img"
```

## Pagination Support

The scraper can automatically follow "Next" page links to scrape multiple pages recursively.

### Enabling Pagination via CLI

```bash
# Enable pagination and scrape up to 10 pages
cargo run -- -u https://books.toscrape.com --paginate -m 10 --save

# Unlimited pages (until no "Next" link found)
cargo run -- -u https://books.toscrape.com --paginate --save --verbose
```

### Enabling Pagination via Configuration

Add a `[pagination]` section to your config file:

```toml
urls = [
    "https://books.toscrape.com/catalogue/category/books/mystery_3/index.html"
]

[selectors]
title = "h1"
content = "article.product_pod h3 a"
links = "a"
images = "article.product_pod img"

[pagination]
enabled = true
next_selector = "li.next a"  # CSS selector for "Next" button
max_pages = 5                # Optional: limit number of pages
```

### How Pagination Works

1. **Scrapes Current Page**: Extracts all data using configured selectors
2. **Finds Next Link**: Searches for "Next" button using `next_selector`
3. **Converts to Absolute URL**: Handles relative URLs automatically
4. **Repeats**: Continues until max pages reached or no more "Next" links
5. **Duplicate Detection**: Tracks visited URLs to prevent infinite loops

### Common Next Selectors

- **books.toscrape.com**: `li.next a`
- **Generic pagination**: `a.next`, `a[rel="next"]`, `.pagination .next`
- **Numbered pages**: `a.page-next`, `.pager-next a`

## Command-Line Options

```
Options:
  -u, --urls <URLS>...      URLs to scrape (can provide multiple)
  -c, --config <CONFIG>     Path to configuration file
  -o, --output <OUTPUT>     Output directory for scraped data [default: output]
  -s, --save                Save output to files
  -v, --verbose             Verbose output
  -p, --paginate            Enable pagination (follow "Next" links)
  -m, --max-pages <NUMBER>  Maximum pages to scrape per URL (0 = unlimited) [default: 0]
  -h, --help                Print help
```

## Output Format

Scraped data is displayed in the terminal and optionally saved as JSON:

```json
{
  "url": "https://example.com",
  "timestamp": "2025-10-25T23:13:51.625995+02:00",
  "title": "Example Domain",
  "content": [
    "This domain is for use in documentation examples..."
  ],
  "links": [
    {
      "text": "Learn more",
      "href": "https://iana.org/domains/example"
    }
  ],
  "images": [
    {
      "alt": "Logo",
      "src": "/images/logo.png"
    }
  ],
  "metadata": {
    "content_count": 2,
    "links_count": 1,
    "images_count": 0
  }
}
```

## Terminal Output Example

```
Scraping 2 URL(s)...

============================================================
URL: https://example.com
Timestamp: 2025-10-25T23:13:51.625995+02:00
============================================================

Title: Example Domain

Content (2 items):
  1. This domain is for use in documentation examples...
  2. Learn more

Links (1 items):
  1. Learn more -> https://iana.org/domains/example

Images (0 items):

Metadata:
  Content items: 2
  Links: 1
  Images: 0

✓ Data saved to: "output/scraped_examplecom_20251025_231351.json"

============================================================
Summary: 1 successful, 0 failed
============================================================
```

## Project Structure

```
rust-web-scraper/
├── src/
│   └── main.rs           # Main application code
├── Cargo.toml            # Dependencies configuration
├── config.toml           # Example configuration file
├── README.md             # This file
└── output/               # Default output directory (created automatically)
    └── *.json            # Scraped data files
```

## Dependencies

- **tokio**: Async runtime for concurrent operations
- **reqwest**: HTTP client for fetching web pages
- **scraper**: HTML parsing with CSS selectors
- **serde & serde_json**: Data serialization
- **clap**: Command-line argument parsing
- **anyhow**: Enhanced error handling
- **toml**: Configuration file parsing
- **chrono**: Timestamp generation
- **futures**: Concurrent task management
- **url**: URL parsing and manipulation for pagination

## Error Handling

The scraper uses `anyhow` for comprehensive error handling:

- Clear error messages for network failures
- Invalid selector detection
- File system error reporting
- Graceful handling of individual URL failures in batch operations

## Performance

- **Concurrent Execution**: All URLs are scraped in parallel using Tokio's async runtime
- **Efficient Parsing**: Uses the fast `scraper` crate built on `html5ever`
- **Minimal Memory Footprint**: Streams data processing where possible

## Use Cases

- Data collection for research projects
- Competitor analysis and monitoring
- Content aggregation
- SEO analysis (extracting links and metadata)
- Web archiving
- Testing and development

## Best Practices

1. **Respect robots.txt**: Check site policies before scraping
2. **Rate Limiting**: Don't overwhelm servers (add delays if needed)
3. **User Agent**: Consider setting a proper user agent
4. **Legal Compliance**: Ensure your use case complies with local laws and site terms

## Future Enhancements

Potential features for future versions:
- Rate limiting / delay between requests
- Custom user-agent configuration
- Proxy support
- JavaScript rendering (via headless browser)
- Export to CSV, XML, or other formats
- Recursive crawling with depth limits
- Pattern-based URL discovery
- Retry logic with exponential backoff

## Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest new features
- Submit pull requests
- Improve documentation

## License

This project is open source and available for educational and commercial use.

## Support

For issues, questions, or suggestions, please open an issue on the project repository.

---

**Built with Rust 🦀 - Fast, Safe, Concurrent**
