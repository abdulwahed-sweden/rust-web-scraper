# 🚀 Quick Start Guide

Get up and running with Rust Web Scraper in 5 minutes!

## Prerequisites

- Rust 1.75+ installed
- OR Docker installed

## Option 1: Run Locally (Recommended for Development)

### 1. Clone and Build

```bash
git clone https://github.com/yourusername/rust-web-scraper.git
cd rust-web-scraper

# Build the project
cargo build --release
```

### 2. Start the Server

```bash
cargo run --release
```

You should see:
```
🚀 Starting Rust Web Scraper
🌐 Server running at http://127.0.0.1:8080
📖 API docs available at http://127.0.0.1:8080/api/health
```

### 3. Open Browser

Navigate to: **http://localhost:8080**

### 4. Start Scraping!

1. Enter a URL (e.g., `https://example.com`)
2. Click "Start Scraping"
3. View results instantly!

## Option 2: Run with Docker

### Quick Run

```bash
docker-compose up
```

That's it! Open **http://localhost:8080**

### Manual Docker

```bash
# Build
docker build -t rust-web-scraper .

# Run
docker run -p 8080:8080 rust-web-scraper
```

## Option 3: Use as Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-web-scraper = { path = "../rust-web-scraper" }
tokio = { version = "1", features = ["full"] }
```

Use in your code:

```rust
use rust_web_scraper::{ScrapingConfig, WebScraper};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ScrapingConfig {
        urls: vec!["https://example.com".to_string()],
        enable_pagination: false,
        max_pages: 0,
        rate_limit: 2.0,
        custom_selectors: None,
    };

    let scraper = WebScraper::new(&config, true)?;
    let session = scraper.scrape(config).await?;

    for result in session.results {
        println!("Title: {:?}", result.content.title);
        println!("Links found: {}", result.content.links.len());
    }

    Ok(())
}
```

## Basic Usage Examples

### 1. Scrape a Single Page

Web UI:
- URL: `https://example.com`
- Leave pagination unchecked
- Click "Start Scraping"

API:
```bash
curl -X POST http://localhost:8080/api/scrape \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "enable_pagination": false,
    "max_pages": 0,
    "rate_limit": 2.0
  }'
```

### 2. Scrape with Pagination

Web UI:
- URL: `https://books.toscrape.com`
- Check "Enable Pagination"
- Set "Max Pages" to 5
- Click "Start Scraping"

API:
```bash
curl -X POST http://localhost:8080/api/scrape \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://books.toscrape.com"],
    "enable_pagination": true,
    "max_pages": 5,
    "rate_limit": 2.0
  }'
```

### 3. Use Custom Selectors

```bash
curl -X POST http://localhost:8080/api/scrape \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "enable_pagination": false,
    "max_pages": 0,
    "rate_limit": 2.0,
    "custom_selectors": {
      "title": ["h1.main-title", "h1"],
      "content": ["article.post", "main p"],
      "links": ["a[href]"],
      "images": ["img[src]"],
      "metadata": ["meta[name=\"description\"]"]
    }
  }'
```

## Testing the API

### Health Check
```bash
curl http://localhost:8080/api/health
```

### Get Scraping History
```bash
curl http://localhost:8080/api/sessions
```

### Clear History
```bash
curl -X DELETE http://localhost:8080/api/sessions
```

## Common Issues

### Port Already in Use

```bash
# Change port
PORT=3000 cargo run --release
```

Or set in `.env`:
```
PORT=3000
HOST=127.0.0.1
```

### Cannot Connect to Server

1. Check if server is running:
   ```bash
   curl http://localhost:8080/api/health
   ```

2. Check logs for errors
3. Try different port

### Docker Build Fails

```bash
# Clean and rebuild
docker-compose down
docker-compose build --no-cache
docker-compose up
```

## Next Steps

1. **Read the Full Documentation**: See [README.md](README.md)
2. **Explore Architecture**: Check [ARCHITECTURE.md](ARCHITECTURE.md)
3. **View Examples**: Look at [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md)
4. **Customize**: Modify `static/` files for UI changes
5. **Extend**: Add new features to `src/` modules

## Tips for Best Results

### 1. Start Simple
- Test with a simple website first
- Don't enable pagination initially
- Use default selectors

### 2. Be Polite
- Set appropriate rate limits (2.0 req/sec is good)
- Don't scrape too many pages
- Respect robots.txt

### 3. Custom Selectors
- Use browser DevTools to inspect elements
- Test selectors in browser console
- Start broad, then narrow down

### 4. Debugging
- Enable verbose mode for detailed logs
- Check Network tab in browser
- Review API responses

## Get Help

- **GitHub Issues**: Report bugs or request features
- **Documentation**: Check the docs folder
- **Examples**: See example configs in root directory

---

**Ready to scrape? Start the server and open http://localhost:8080!** 🎉
