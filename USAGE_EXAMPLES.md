# Rust Web Scraper - Complete Usage Examples

## 🎯 Quick Start

### Option 1: Web Interface (Easiest)

```bash
# Build and run
cargo build --release
cargo run --bin scraper-web

# Open browser
# Navigate to http://localhost:8080
# Enter Etsy URL and click "Start Scraping"
```

### Option 2: Docker (Production)

```bash
# One command deployment
docker-compose up --build

# Access at http://localhost:8080
# Output saved to ./output/ directory
```

### Option 3: Command Line

```bash
# General scraping
cargo run --bin scraper-cli -- \
  -u https://example.com \
  --save

# Etsy-specific (Note: Limited without browser automation)
cargo run --bin scraper-cli -- \
  -u "https://www.etsy.com/se-en/c/bath-and-beauty/soaps/bath-salts-and-scrubs" \
  --paginate \
  --max-pages 5 \
  --save \
  --verbose
```

## 🛍️ Etsy Scraping Examples

### Web Interface Method (Recommended)

1. **Start Server**
   ```bash
   cargo run --bin scraper-web
   ```

2. **Open Browser**
   - Go to http://localhost:8080

3. **Enter Details**
   - Category URL: `https://www.etsy.com/se-en/c/bath-and-beauty/soaps/bath-salts-and-scrubs`
   - Max Pages: `5`

4. **Click "Start Scraping"**

5. **View Results**
   - See stats in browser
   - Check `output/etsy_reviews.json`

### Direct API Call (Advanced)

```bash
curl -X POST http://localhost:8080/api/scrape \
  -H "Content-Type: application/json" \
  -d '{
    "category_url": "https://www.etsy.com/se-en/c/bath-and-beauty/soaps/bath-salts-and-scrubs",
    "max_pages": 5
  }'
```

## 📚 General Web Scraping Examples

### Books to Scrape

```bash
# With configuration file
cargo run --bin scraper-cli -- --config books-config.toml --save --verbose

# Direct CLI
cargo run --bin scraper-cli -- \
  -u "https://books.toscrape.com/catalogue/category/books/mystery_3/index.html" \
  --paginate \
  -m 5 \
  --save
```

### Multiple URLs Concurrently

```bash
cargo run --bin scraper-cli -- \
  -u https://example.com \
  -u https://www.rust-lang.org \
  -u https://books.toscrape.com \
  --save \
  --verbose
```

### Custom Output Directory

```bash
cargo run --bin scraper-cli -- \
  -u https://example.com \
  --save \
  --output ./my-data \
  --verbose
```

## 🐳 Docker Examples

### Basic Docker Run

```bash
# Build image
docker build -t etsy-scraper .

# Run container
docker run -p 8080:8080 \
  -v $(pwd)/output:/app/output \
  etsy-scraper
```

### Docker Compose

```bash
# Start in background
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down

# Rebuild and restart
docker-compose up --build
```

### Docker with Custom Environment

```bash
# Set log level
docker run -p 8080:8080 \
  -e RUST_LOG=debug \
  -v $(pwd)/output:/app/output \
  etsy-scraper
```

## 🔧 Configuration File Examples

### Etsy Configuration (`etsy-config.toml`)

```toml
urls = [
    "https://www.etsy.com/se-en/c/bath-and-beauty/soaps/bath-salts-and-scrubs"
]

[selectors]
title = "h1"
content = "h3, .v2-listing-card__title"
links = "a"
images = "img"

[pagination]
enabled = true
next_selector = "a[rel='next']"
max_pages = 10
```

### Books Configuration (`books-config.toml`)

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
next_selector = "li.next a"
max_pages = 5
```

## 📊 Output Examples

### Etsy Output Structure

```json
{
  "timestamp": "2025-10-26T00:15:30+02:00",
  "category_url": "https://www.etsy.com/se-en/c/bath-and-beauty/soaps/bath-salts-and-scrubs",
  "total_products": 50,
  "total_reviews": 0,
  "products": [
    {
      "name": "Lavender Bath Salts",
      "price": "$24.99",
      "rating": "4.8",
      "review_count": "245",
      "product_url": "https://www.etsy.com/listing/...",
      "image_url": "https://i.etsystatic.com/...",
      "reviews": []
    }
  ],
  "summary": {
    "pages_scraped": 5,
    "products_with_reviews": 0,
    "average_rating": 4.7,
    "time_taken_seconds": 125
  }
}
```

### General Scraping Output

```json
{
  "url": "https://books.toscrape.com/",
  "timestamp": "2025-10-26T00:16:45+02:00",
  "title": "All products",
  "content": ["Book Title 1", "Book Title 2"],
  "links": [
    {"text": "Home", "href": "https://books.toscrape.com/index.html"}
  ],
  "images": [
    {"alt": "Book Cover", "src": "https://books.toscrape.com/media/..."}
  ],
  "metadata": {
    "content_count": 20,
    "links_count": 94,
    "images_count": 20
  }
}
```

## 🚀 Performance Tips

### Optimize Rate Limiting

Edit `src/rate_limit.rs`:
```rust
impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(5.0) // 5 requests per second
    }
}
```

### Increase Timeout

Edit `src/etsy.rs`:
```rust
let client = reqwest::Client::builder()
    .cookie_store(true)
    .timeout(std::time::Duration::from_secs(60)) // 60 seconds
    .build()?;
```

### Add More User Agents

Edit `src/user_agents.rs`:
```rust
pub const USER_AGENTS: &[&str] = &[
    // Add more user agents here
    "Your Custom User Agent",
];
```

## 🐛 Troubleshooting

### Empty Results

```bash
# Enable verbose logging
export RUST_LOG=debug
cargo run --bin scraper-web

# Check selectors
cargo run --bin scraper-cli -- -u URL --verbose
```

### Build Errors

```bash
# Update Rust
rustup update

# Clean build
cargo clean
cargo build --release
```

### Docker Issues

```bash
# Check logs
docker-compose logs -f

# Restart
docker-compose restart

# Full rebuild
docker-compose down
docker-compose build --no-cache
docker-compose up
```

## 📝 Notes

### Etsy Limitations

- Etsy uses heavy JavaScript rendering
- Full review extraction requires headless browser
- Rate limiting prevents quick large scrapes
- Consider Etsy's official API for production

### Legal Considerations

- Always check `robots.txt`
- Respect rate limits
- Don't overload servers
- Use data ethically

### Best Practices

1. Start with small page counts
2. Use verbose mode for debugging
3. Check output files regularly
4. Monitor server logs
5. Test selectors first

## 🎓 Learning Examples

### Example 1: Basic Scraping

```bash
cargo run --bin scraper-cli -- \
  -u https://example.com \
  --verbose
```

### Example 2: With Pagination

```bash
cargo run --bin scraper-cli -- \
  -u https://books.toscrape.com \
  --paginate \
  -m 3 \
  --verbose
```

### Example 3: Save to File

```bash
cargo run --bin scraper-cli -- \
  -u https://books.toscrape.com \
  --save \
  --output ./results
```

### Example 4: Full Pipeline

```bash
# 1. Start web server
cargo run --bin scraper-web &

# 2. Make API request
curl -X POST http://localhost:8080/api/scrape \
  -H "Content-Type: application/json" \
  -d '{"category_url": "https://www.etsy.com/...", "max_pages": 5}'

# 3. Check results
cat output/etsy_reviews.json | jq '.summary'
```

## 🎯 Production Deployment

### Environment Setup

```bash
# Set environment variables
export RUST_LOG=info
export PORT=8080

# Run as service
cargo build --release
./target/release/scraper-web
```

### Systemd Service

Create `/etc/systemd/system/scraper-web.service`:

```ini
[Unit]
Description=Etsy Web Scraper
After=network.target

[Service]
Type=simple
User=scraper
WorkingDirectory=/opt/rust-web-scraper
ExecStart=/opt/rust-web-scraper/target/release/scraper-web
Restart=always
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable scraper-web
sudo systemctl start scraper-web
```
