# Changelog

All notable changes to the Rust Web Scraper project.

## [0.3.0] - 2025-10-26

### Added - Etsy Product Review Extraction Support

#### New Features

1. **Review Fetching Module** (`src/etsy_reviews.rs`)
   - Extract listing IDs from Etsy product URLs using regex
   - Fetch up to 20 reviews per product via API
   - Graceful error handling with retry logic (3 attempts)
   - Exponential backoff for rate-limited requests
   - Support for 403, 404, and 429 HTTP errors

2. **CLI Enhancements**
   - New flag: `--category-url <URL>` - Enable Etsy-specific scraping mode
   - New flag: `--fetch-reviews` - Enable review extraction (default: off)
   - Enhanced output with review statistics
   - Verbose mode shows review fetching progress

3. **Web Interface Updates**
   - New checkbox: "Fetch Product Reviews (slower, up to 20 per product)"
   - Updated API to accept `fetch_reviews` parameter
   - Real-time progress tracking for review fetching
   - Enhanced result display with review counts

4. **EtsyScraper API Changes**
   - New method: `with_options(verbose, fetch_reviews)`
   - Backward compatible: `new(verbose)` still works (reviews disabled)
   - Added `fetch_reviews: bool` field to scraper struct
   - Integrated review fetching into product extraction pipeline

#### Data Structure Changes

```rust
// New Review struct in etsy.rs
pub struct Review {
    pub text: String,
    pub reviewer_name: Option<String>,
    pub rating: Option<String>,
}

// Enhanced EtsyProduct
pub struct EtsyProduct {
    // ... existing fields
    pub reviews: Vec<Review>, // NEW
}

// API Response structures
pub struct EtsyReviewResponse {
    pub reviews: Vec<ApiReview>,
}
```

#### Files Modified

- `src/etsy_reviews.rs` - **NEW** (173 lines)
- `src/etsy.rs` - Modified to integrate review fetching
- `src/main.rs` - Added Etsy CLI mode with new flags
- `src/web.rs` - Added review checkbox and API support
- `src/lib.rs` - Exported new `etsy_reviews` module
- `Cargo.toml` - No new dependencies required

#### Documentation Added

- `REVIEW_EXTRACTION.md` - Comprehensive guide (350+ lines)
- Updated `USAGE_EXAMPLES.md` with review extraction examples
- This `CHANGELOG.md` file

### Known Limitations

**Etsy API Access**: Review extraction currently returns empty arrays due to:
- Etsy's API endpoint returning 404 errors
- Possible authentication requirements
- Anti-scraping measures

**Workarounds documented in REVIEW_EXTRACTION.md**:
1. HTML scraping from product pages
2. Headless browser automation
3. Official Etsy API with authentication
4. Third-party scraping services

### Performance Impact

| Mode | Products | Time | Reviews |
|------|----------|------|---------|
| Without `--fetch-reviews` | 50 | ~3s | 0 |
| With `--fetch-reviews` | 50 | ~34s | 0* |

*Currently returns empty due to API restrictions

### Testing

```bash
# Test CLI with reviews
cargo run --bin scraper-cli -- \
  --category-url "https://www.etsy.com/se-en/c/bath-and-beauty" \
  --fetch-reviews \
  --max-pages 1 \
  --verbose

# Test web interface
cargo run --bin scraper-web
# Visit http://localhost:8080 and check the review checkbox
```

### Breaking Changes

None. All changes are backward compatible.

### Migration Guide

**For existing users**: No changes required. Review fetching is opt-in via `--fetch-reviews` flag.

**To enable review fetching**:

```rust
// Old way (still works, reviews disabled)
let scraper = EtsyScraper::new(true)?;

// New way (reviews enabled)
let scraper = EtsyScraper::with_options(true, true)?;
//                                      ^verbose  ^fetch_reviews
```

## [0.2.0] - 2025-10-26 (Previous Release)

### Added
- Etsy-specific scraping with `EtsyScraper` struct
- Web interface with Actix-web at http://localhost:8080
- Rate limiting (2 requests/second)
- Random user agent rotation
- Docker support with `docker-compose.yml`
- Pagination support for Etsy categories
- JSON export to `output/etsy_reviews.json`

### Files Added
- `src/etsy.rs` - Etsy scraping logic
- `src/web.rs` - Web server and UI
- `src/rate_limit.rs` - Rate limiting
- `src/user_agents.rs` - UA rotation
- `Dockerfile` - Container build
- `docker-compose.yml` - Orchestration
- `ETSY_SETUP.md` - Setup guide
- `USAGE_EXAMPLES.md` - Usage examples

## [0.1.0] - Initial Release

### Added
- Basic web scraping with CSS selectors
- Concurrent scraping with Tokio
- CLI arguments with Clap
- Configuration file support (TOML)
- Pagination support
- File output with JSON serialization
- Error handling with anyhow

---

## Future Roadmap

### v0.4.0 (Planned)
- [ ] HTML-based review scraping (fallback)
- [ ] Headless browser support (Playwright)
- [ ] Review caching to database
- [ ] Sentiment analysis integration

### v0.5.0 (Planned)
- [ ] Official Etsy API integration
- [ ] OAuth authentication support
- [ ] Multi-threaded product processing
- [ ] GraphQL API endpoint

### v1.0.0 (Future)
- [ ] Production-ready review extraction
- [ ] Complete API coverage
- [ ] Performance optimizations
- [ ] Comprehensive test suite
