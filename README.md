# Rust Web Scraper

A powerful, feature-rich web scraper built with Rust that supports concurrent scraping, customizable selectors, and comprehensive data extraction.

## Features

- **Concurrent Scraping**: Scrape multiple URLs simultaneously for maximum performance
- **CLI Support**: Command-line interface with multiple options via `clap`
- **Configuration Files**: Define URLs and selectors in TOML config files
- **Enhanced Error Handling**: Clear error messages with `anyhow`
- **File Output**: Save scraped data to JSON files with timestamps
- **Comprehensive Extraction**:
  - Page titles
  - Content paragraphs
  - Links (text + href)
  - Images (alt + src)
  - Metadata (counts and statistics)
- **Verbose Mode**: Optional detailed logging
- **Flexible Selectors**: Customize CSS selectors for any website structure

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

## Command-Line Options

```
Options:
  -u, --urls <URLS>...   URLs to scrape (can provide multiple)
  -c, --config <CONFIG>  Path to configuration file
  -o, --output <OUTPUT>  Output directory for scraped data [default: output]
  -s, --save             Save output to files
  -v, --verbose          Verbose output
  -h, --help             Print help
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
