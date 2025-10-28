# Etsy Product Review Extraction - Implementation Guide

## Overview

The Rust Etsy scraper now includes **product review extraction** functionality. This feature attempts to fetch customer reviews for each product by accessing Etsy's review API endpoints.

## Features Implemented

### 1. Review Fetching Module (`src/etsy_reviews.rs`)

- **Listing ID Extraction**: Automatically extracts product listing IDs from Etsy URLs
- **API Integration**: Attempts to fetch up to 20 reviews per product via Etsy's review API
- **Error Handling**: Gracefully handles 403, 429, and 404 errors with retry logic
- **Rate Limiting**: Respects rate limits with exponential backoff
- **User Agent Rotation**: Uses random user agents to avoid detection

### 2. CLI Support

New command-line flags added to `scraper-cli`:

```bash
cargo run --bin scraper-cli -- \
  --category-url "https://www.etsy.com/se-en/c/bath-and-beauty/soaps/bath-salts-and-scrubs" \
  --max-pages 5 \
  --fetch-reviews \
  --save \
  --verbose
```

**Flags:**
- `--category-url <URL>`: Etsy category URL to scrape (enables Etsy mode)
- `--fetch-reviews`: Enable review extraction (default: disabled for faster scraping)
- `--verbose`: Show detailed progress including review fetching attempts

### 3. Web Interface Support

The web UI at http://localhost:8080 now includes:
- ✅ Checkbox: "Fetch Product Reviews (slower, up to 20 per product)"
- ✅ API parameter: `fetch_reviews: boolean`
- ✅ Real-time progress tracking

## Usage Examples

### CLI Example

```bash
# Scrape with review extraction
cargo run --bin scraper-cli -- \
  --category-url "https://www.etsy.com/se-en/c/category/subcategory" \
  --fetch-reviews \
  --save \
  --verbose

# Scrape without reviews (faster)
cargo run --bin scraper-cli -- \
  --category-url "https://www.etsy.com/se-en/c/category/subcategory" \
  --save
```

### Web API Example

```bash
curl -X POST http://localhost:8080/api/scrape \
  -H "Content-Type: application/json" \
  -d '{
    "category_url": "https://www.etsy.com/se-en/c/bath-and-beauty/soaps",
    "max_pages": 3,
    "fetch_reviews": true
  }'
```

### Web Interface

1. Navigate to http://localhost:8080
2. Enter Etsy category URL
3. Check ✓ "Fetch Product Reviews"
4. Click "Start Scraping"

## Output Structure

```json
{
  "timestamp": "2025-10-26T01:35:01+02:00",
  "category_url": "https://www.etsy.com/...",
  "total_products": 48,
  "total_reviews": 0,
  "products": [
    {
      "name": "Natural Bath Salt Soak",
      "price": "32.81 SEK",
      "rating": "4.9",
      "review_count": "1357",
      "product_url": "https://www.etsy.com/listing/1234567890/...",
      "image_url": "https://i.etsystatic.com/...",
      "reviews": [
        {
          "text": "Smells wonderful, perfect for gifting!",
          "reviewer_name": "Sarah",
          "rating": "5"
        }
      ]
    }
  ]
}
```

## Current Limitations

### ⚠️ Etsy API Access Issues

**Status**: Review extraction currently returns empty arrays due to API access restrictions.

**Observed Behavior**:
- ✅ Listing IDs are correctly extracted from product URLs
- ✅ Review API requests are properly formatted and sent
- ❌ All API requests return `404 Not Found`

**Possible Causes**:

1. **API Endpoint Changes**: Etsy may have changed their internal API structure
2. **Authentication Required**: The API might require OAuth tokens or API keys
3. **Rate Limiting**: Etsy may block unauthenticated API requests
4. **Regional Restrictions**: API access may vary by region
5. **Anti-Scraping Measures**: Etsy actively prevents automated access

**Tested Endpoint**:
```
https://www.etsy.com/api/v3/ajax/bespoke/member/feedback?listing_id={ID}&limit=20
```

### Alternative Solutions

#### Option 1: HTML Scraping from Product Pages

**Pros**: More reliable, works with current HTML structure
**Cons**: Much slower (requires visiting each product page)

**Implementation approach**:
```rust
async fn fetch_reviews_from_html(product_url: &str) -> Result<Vec<Review>> {
    // 1. Visit product page
    // 2. Parse review HTML elements
    // 3. Extract review text, author, rating
}
```

#### Option 2: Headless Browser (Selenium/Playwright)

**Pros**: Can handle JavaScript-rendered content
**Cons**: Much slower, requires browser automation

**Recommended crate**: `fantoccini` or `headless_chrome`

#### Option 3: Etsy Official API

**Pros**: Official, supported, rate-limited properly
**Cons**: Requires API key, limited access, costs money

**Registration**: https://www.etsy.com/developers

#### Option 4: Third-Party Services

**Options**:
- ScraperAPI
- Bright Data
- Oxylabs

**Pros**: Handles anti-bot measures, provides infrastructure
**Cons**: Subscription costs, rate limits

## Code Structure

### Files Modified/Created

1. **`src/etsy_reviews.rs`** (NEW)
   - `extract_listing_id()`: Regex-based ID extraction
   - `fetch_reviews()`: API client with retry logic
   - `EtsyReviewResponse`: JSON response structures

2. **`src/etsy.rs`** (MODIFIED)
   - Added `fetch_reviews: bool` field to `EtsyScraper`
   - New method: `with_options(verbose, fetch_reviews)`
   - Updated `extract_product_info()` to call review fetcher

3. **`src/main.rs`** (MODIFIED)
   - Added `--category-url` flag
   - Added `--fetch-reviews` flag
   - Etsy scraping mode in main()

4. **`src/web.rs`** (MODIFIED)
   - Added `fetch_reviews` to `ScrapeRequest`
   - Updated HTML form with checkbox
   - Updated JavaScript to pass flag to API

5. **`src/lib.rs`** (MODIFIED)
   - Exported `etsy_reviews` module

## Testing

### Unit Tests

```bash
# Run tests for listing ID extraction
cargo test test_extract_listing_id
```

### Integration Tests

```bash
# Test without review fetching (fast)
cargo run --bin scraper-cli -- \
  --category-url "https://www.etsy.com/se-en/c/bath-and-beauty" \
  --max-pages 1 \
  --save

# Test with review fetching (slow, currently returns empty reviews)
cargo run --bin scraper-cli -- \
  --category-url "https://www.etsy.com/se-en/c/bath-and-beauty" \
  --max-pages 1 \
  --fetch-reviews \
  --verbose
```

### Expected Output

```
🛍️  Etsy Scraper Mode
Category: https://www.etsy.com/se-en/c/bath-and-beauty
Fetch Reviews: ✓ Enabled
Max Pages: 1

[Page 1/1] Fetching: https://www.etsy.com/...
    Processing product 1...
      Fetching reviews from API for listing 1234567890
        ⚠ HTTP error: 404 Not Found
    Processing product 2...
      Fetching reviews from API for listing 9876543210
        ⚠ HTTP error: 404 Not Found
  ✓ Found 48 products on this page

============================================================
📊 Scraping Results
============================================================
✓ Products found: 48
✓ Reviews collected: 0
✓ Pages scraped: 1
✓ Time taken: 34s

💾 Saved to: output/etsy_reviews.json
```

## Performance Considerations

### With Review Fetching Enabled

- **Rate**: ~2 requests/second (rate limited)
- **Time per product**: ~0.5 seconds
- **50 products**: ~25-30 seconds
- **Memory**: ~60MB

### Without Review Fetching

- **Rate**: ~2 requests/second
- **Time per page**: ~2-3 seconds
- **50 products**: ~3 seconds
- **Memory**: ~50MB

## Future Improvements

### Short Term (Easy)

1. **Implement HTML scraping fallback**: Visit product pages directly
2. **Add caching**: Cache reviews to avoid redundant requests
3. **Progress indicators**: Better UI feedback during review fetching
4. **Error categorization**: Distinguish between 403, 404, 429 errors

### Medium Term

1. **Headless browser support**: Use Playwright/Selenium for JavaScript rendering
2. **Proxy rotation**: Avoid IP blocks with proxy pools
3. **Cookie management**: Maintain session cookies across requests
4. **Captcha handling**: Detect and handle captcha challenges

### Long Term

1. **Etsy API integration**: Use official API with authentication
2. **Database storage**: Store reviews in SQLite/PostgreSQL
3. **Sentiment analysis**: Analyze review sentiment with AI
4. **Review monitoring**: Track review changes over time

## Troubleshooting

### Problem: All reviews return empty

**Solution**: This is expected. Etsy's API is not publicly accessible. Consider:
- Using Etsy's official API with authentication
- Implementing HTML scraping from product pages
- Using a headless browser for JavaScript rendering

### Problem: Too many 429 errors

**Solution**: Increase rate limiting delay:
```rust
// In src/rate_limit.rs
impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(1.0) // Reduce to 1 request/second
    }
}
```

### Problem: Slow scraping with reviews enabled

**Solution**: Disable review fetching for initial scraping:
```bash
cargo run -- --category-url "..." --save  # No --fetch-reviews flag
```

## Legal and Ethical Considerations

⚠️ **Important Disclaimers**:

1. **Terms of Service**: Review Etsy's ToS before scraping
2. **robots.txt**: Respect Etsy's robots.txt directives
3. **Rate Limiting**: Don't overload Etsy's servers
4. **Data Usage**: Use scraped data ethically and legally
5. **Privacy**: Reviews may contain personal information
6. **Commercial Use**: Check licensing before commercial use

## Support

For issues or questions:
1. Check verbose logs: `--verbose` flag
2. Review error messages in console
3. Check `output/etsy_reviews.json` for partial data
4. Consider using Etsy's official API for production use

## References

- Etsy Developer API: https://www.etsy.com/developers
- Rust reqwest crate: https://docs.rs/reqwest
- Scraper crate: https://docs.rs/scraper
- Regex crate: https://docs.rs/regex
