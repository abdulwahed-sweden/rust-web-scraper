# 🏗️ Architecture Documentation

## Overview

Rust Web Scraper is built with a clean, modular architecture that separates concerns and makes the codebase maintainable and extensible.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      User Interfaces                        │
├───────────────────┬──────────────────┬──────────────────────┤
│   Web Browser     │   Tauri Desktop  │   API Clients        │
│   (static/*)      │   (wrapper)      │   (REST API)         │
└───────────────────┴──────────────────┴──────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Actix-Web Server                         │
│                      (src/main.rs)                          │
├─────────────────────────────────────────────────────────────┤
│  • HTTP Server                                              │
│  • Static File Serving                                      │
│  • CORS Configuration                                       │
│  • Logging & Compression                                    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                     API Layer                               │
│                    (src/api.rs)                             │
├─────────────────────────────────────────────────────────────┤
│  • Route Handlers                                           │
│  • Request/Response Models                                  │
│  • Session Management                                       │
│  • Error Handling                                           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  Scraping Engine                            │
│                 (src/scraper.rs)                            │
├─────────────────────────────────────────────────────────────┤
│  • WebScraper - Main scraping logic                         │
│  • Configuration management                                 │
│  • Pagination handling                                      │
│  • Result aggregation                                       │
└─────────────────────────────────────────────────────────────┘
          │                           │
          ▼                           ▼
┌─────────────────────┐    ┌──────────────────────────────┐
│  Auto Selectors     │    │     Utilities                │
│ (auto_selectors.rs) │    │     (utils.rs)               │
├─────────────────────┤    ├──────────────────────────────┤
│ • SelectorDetector  │    │ • RateLimiter                │
│ • Content extraction│    │ • User Agent rotation        │
│ • Smart heuristics  │    │ • Helper functions           │
└─────────────────────┘    └──────────────────────────────┘
```

## Module Breakdown

### 1. Main Server (`src/main.rs`)

**Purpose**: Entry point and HTTP server configuration

**Responsibilities**:
- Initialize Actix-Web HTTP server
- Configure middleware (CORS, logging, compression)
- Define API routes
- Serve static files
- Manage application state

**Key Components**:
```rust
AppState {
    sessions: Arc<Mutex<Vec<ScrapingSession>>>
}
```

### 2. API Layer (`src/api.rs`)

**Purpose**: HTTP request handling and API endpoints

**Endpoints**:
- `GET /api/health` - Health check
- `POST /api/scrape` - Start scraping job
- `GET /api/sessions` - Get all sessions
- `GET /api/sessions/{id}` - Get specific session
- `DELETE /api/sessions` - Clear all sessions

**Models**:
- `ScrapeRequest` - Input parameters
- `ScrapeResponse` - API response wrapper
- `AppState` - Shared application state

### 3. Scraping Engine (`src/scraper.rs`)

**Purpose**: Core scraping logic and orchestration

**Key Types**:
```rust
pub struct WebScraper {
    client: reqwest::Client,
    rate_limiter: RateLimiter,
    detector: SelectorDetector,
    verbose: bool,
}

pub struct ScrapingConfig {
    urls: Vec<String>,
    enable_pagination: bool,
    max_pages: usize,
    rate_limit: f64,
    custom_selectors: Option<AutoSelectors>,
}

pub struct ScrapingSession {
    start_time: String,
    config: ScrapingConfig,
    results: Vec<ScrapingResult>,
    total_pages_scraped: usize,
    total_links_found: usize,
    total_images_found: usize,
    errors: Vec<String>,
}
```

**Flow**:
1. Create `WebScraper` with configuration
2. For each URL:
   - Fetch page with rate limiting
   - Detect/apply selectors
   - Extract content
   - If pagination enabled, find next page
3. Aggregate results into `ScrapingSession`

### 4. Auto Selectors (`src/auto_selectors.rs`)

**Purpose**: Intelligent content detection without manual configuration

**Key Types**:
```rust
pub struct SelectorDetector {
    selectors: AutoSelectors,
}

pub struct AutoSelectors {
    title: Vec<String>,
    content: Vec<String>,
    links: Vec<String>,
    images: Vec<String>,
    metadata: Vec<String>,
}

pub struct DetectedContent {
    title: Option<String>,
    content: Vec<String>,
    links: Vec<LinkData>,
    images: Vec<ImageData>,
    metadata: HashMap<String, String>,
}
```

**Detection Strategy**:
1. **Title**: Try common patterns (h1, title, og:title)
2. **Content**: Look for article, main, paragraphs
3. **Links**: Extract all anchor tags with URLs
4. **Images**: Find img tags and data-src attributes
5. **Metadata**: Parse meta tags for SEO data

**Smart Features**:
- Duplicate detection
- URL resolution (relative → absolute)
- External link detection
- Empty content filtering

### 5. Utilities (`src/utils.rs`)

**Purpose**: Shared utilities and helpers

**Components**:
- `RateLimiter`: Polite scraping with delays
- `get_random_user_agent()`: Rotate user agents
- Constants and helper functions

## Data Flow

### Scraping Request Flow

```
1. User submits form in Web UI
   └─> POST /api/scrape with JSON body

2. API handler validates request
   └─> Creates ScrapingConfig

3. WebScraper initialized
   └─> Creates HTTP client
   └─> Sets up rate limiter
   └─> Configures selector detector

4. For each URL:
   └─> Rate limiter waits
   └─> Fetch page (reqwest)
   └─> Parse HTML (scraper crate)
   └─> Detect/extract content
   └─> Find next page if pagination enabled
   └─> Store ScrapingResult

5. Aggregate results
   └─> Create ScrapingSession
   └─> Store in AppState
   └─> Return to client

6. Client displays results
   └─> Show statistics
   └─> Render extracted content
   └─> Offer JSON download
```

## Technology Stack

### Backend
- **Actix-Web** - Fast, ergonomic web framework
- **Tokio** - Async runtime
- **Reqwest** - HTTP client
- **Scraper** - HTML parsing with CSS selectors
- **Serde** - Serialization/deserialization
- **Anyhow** - Error handling

### Frontend
- **Vanilla JavaScript** - No framework overhead
- **Modern CSS** - Grid, Flexbox, CSS Variables
- **Responsive Design** - Works on all devices

### DevOps
- **Docker** - Containerization
- **Docker Compose** - Multi-container orchestration
- **Cargo** - Build system and package manager

## Design Patterns

### 1. Builder Pattern
Used in `WebScraper::new()` and configuration

### 2. Strategy Pattern
`SelectorDetector` can use custom or auto selectors

### 3. Repository Pattern
`AppState` manages session storage

### 4. Middleware Chain
Actix-Web middleware for logging, CORS, compression

### 5. Async/Await
Throughout for non-blocking I/O

## Error Handling

- Uses `anyhow::Result` for flexible error propagation
- Graceful degradation (continues on individual URL failures)
- Detailed error messages logged
- User-friendly error responses in API

## Performance Considerations

### Async Operations
- All network I/O is async
- Non-blocking HTTP server
- Concurrent URL processing (when not paginating)

### Rate Limiting
- Prevents overwhelming target servers
- Configurable requests per second
- Automatic delays between requests

### Memory Efficiency
- Streaming HTML parsing
- Bounded session storage
- Efficient string handling

### Optimization Opportunities
- Connection pooling (reqwest handles this)
- Response compression
- Caching (future enhancement)

## Security Considerations

### CORS
- Configured to allow any origin (adjust for production)

### Input Validation
- URL validation
- Selector syntax validation
- Rate limit bounds

### Rate Limiting
- Prevents abuse of scraping service
- Protects target servers

### No Authentication (Current)
- Add auth for production use
- Consider API keys
- Implement rate limiting per user

## Scalability

### Horizontal Scaling
- Stateless design (sessions stored in memory, can move to Redis)
- Docker-friendly
- Can run multiple instances behind load balancer

### Vertical Scaling
- Efficient async I/O
- Low memory footprint
- Fast HTML parsing

### Database Integration
- Currently in-memory storage
- Easy to add PostgreSQL/MongoDB for persistence
- Sessions are serializable (JSON)

## Future Enhancements

### Backend
- [ ] Persistent storage (PostgreSQL/Redis)
- [ ] WebSocket for real-time updates
- [ ] Job queue (for long-running scrapes)
- [ ] Authentication & authorization
- [ ] User accounts and saved configurations
- [ ] JavaScript rendering (headless browser)
- [ ] Proxy support

### Frontend
- [ ] Real-time progress updates
- [ ] Export to CSV/Excel
- [ ] Saved scraping templates
- [ ] Schedule recurring scrapes
- [ ] Data visualization charts

### Architecture
- [ ] Microservices (separate scraping workers)
- [ ] Message queue (RabbitMQ/Kafka)
- [ ] Caching layer (Redis)
- [ ] Monitoring & alerting (Prometheus/Grafana)

## Testing Strategy

### Unit Tests
- Test individual functions
- Mock HTTP responses
- Validate selector detection

### Integration Tests
- Test API endpoints
- Verify end-to-end flows
- Test with real websites (carefully)

### Performance Tests
- Load testing with Apache Bench
- Memory profiling
- Concurrent request handling

## Deployment

### Local Development
```bash
cargo run
```

### Docker
```bash
docker build -t rust-web-scraper .
docker run -p 8080:8080 rust-web-scraper
```

### Docker Compose
```bash
docker-compose up
```

### Production Considerations
- Use reverse proxy (Nginx)
- Enable HTTPS (Let's Encrypt)
- Set up monitoring
- Configure log rotation
- Implement backup strategy

---

**Last Updated**: 2025-10-30
**Version**: 0.3.0
