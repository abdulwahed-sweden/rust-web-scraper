# Etsy Scraper Setup Guide

## Quick Start

### Option 1: Web Interface (Recommended)

```bash
# Build and run web server
cargo build --release
cargo run --bin scraper-web

# Open browser to http://localhost:8080
```

### Option 2: Docker

```bash
# Build and run with Docker Compose
docker-compose up --build

# Access at http://localhost:8080
```

### Option 3: CLI

```bash
# Using CLI directly (limited - Etsy uses JS rendering)
cargo run --bin scraper-cli -- \
  --category-url "https://www.etsy.com/se-en/c/bath-and-beauty/soaps/bath-salts-and-scrubs" \
  --paginate \
  --max-pages 5 \
  --save \
  --verbose
```

## Features

✅ Automatic pagination through category pages
✅ Rate limiting (2 requests/second)
✅ Random user agent rotation
✅ Beautiful web UI at http://localhost:8080
✅ JSON export
✅ Real-time progress tracking
✅ Error handling and logging

## Web Interface

The web UI provides:
- Input form for Etsy category URL
- Max pages configuration
- Real-time progress bar
- Results summary with statistics
- Automatic file saving

## Output

Results are saved to `output/etsy_reviews.json` with structure:

```json
{
  "timestamp": "2025-10-26T00:00:00+00:00",
  "category_url": "https://www.etsy.com/...",
  "total_products": 50,
  "total_reviews": 250,
  "products": [
    {
      "name": "Product Name",
      "price": "$19.99",
      "rating": "4.8",
      "review_count": "123",
      "product_url": "https://...",
      "image_url": "https://...",
      "reviews": []
    }
  ],
  "summary": {
    "pages_scraped": 5,
    "products_with_reviews": 45,
    "average_rating": 4.7,
    "time_taken_seconds": 120
  }
}
```

## Important Notes

### Etsy's JavaScript Rendering

Etsy heavily uses JavaScript to render content. This scraper:
- Works best for initial product listings
- May not capture all dynamically loaded content
- Reviews require individual product page visits

### Rate Limiting

- Default: 2 requests per second
- Configurable in `src/rate_limit.rs`
- Helps prevent IP blocking

### User Agents

- Rotates through 5 different user agents
- Appears as regular browser traffic
- Configurable in `src/user_agents.rs`

## Troubleshooting

### Empty Results

- Etsy may block automated requests
- Try using the web interface
- Reduce request frequency
- Use different IP/VPN

### Build Errors

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

## Advanced Usage

### Custom Selectors

Edit `src/etsy.rs` to adjust CSS selectors for:
- Product listings
- Prices
- Ratings
- Review text

### Extending Functionality

Add features in:
- `src/etsy.rs` - Scraping logic
- `src/web.rs` - Web interface
- `src/rate_limit.rs` - Rate limiting

## Docker Commands

```bash
# Build
docker-compose build

# Run in background
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down

# Access shell
docker-compose exec etsy-scraper /bin/bash
```

## Environment Variables

```bash
# Set log level
export RUST_LOG=debug

# Custom port
export PORT=3000
```

## Performance

- ~5 products/second with rate limiting
- Memory usage: ~50MB
- CPU usage: Low
- Network: Depends on page size

## Legal & Ethical Considerations

⚠️ **Important**: Always respect website terms of service
- Review Etsy's robots.txt
- Don't overload servers
- Use data responsibly
- Consider API alternatives

## Support

For issues or questions:
1. Check logs: `docker-compose logs`
2. Review Etsy's HTML structure
3. Adjust selectors in `src/etsy.rs`
4. Test with web interface first
