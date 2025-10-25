use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use rust_web_scraper::etsy::{EtsyScraper, EtsyScrapingResult};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
struct ScrapeRequest {
    category_url: String,
    max_pages: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScrapeResponse {
    success: bool,
    message: String,
    data: Option<EtsyScrapingResult>,
}

#[derive(Clone)]
struct AppState {
    results: Arc<Mutex<Option<EtsyScrapingResult>>>,
}

async fn index() -> Result<HttpResponse> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Etsy Scraper - Web Interface</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 900px;
            margin: 0 auto;
            background: white;
            border-radius: 20px;
            padding: 40px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
        }
        h1 {
            color: #667eea;
            margin-bottom: 10px;
            font-size: 2.5em;
        }
        .subtitle {
            color: #666;
            margin-bottom: 30px;
            font-size: 1.1em;
        }
        .input-group {
            margin-bottom: 20px;
        }
        label {
            display: block;
            margin-bottom: 8px;
            color: #333;
            font-weight: 600;
        }
        input[type="text"], input[type="number"] {
            width: 100%;
            padding: 12px 15px;
            border: 2px solid #ddd;
            border-radius: 8px;
            font-size: 16px;
            transition: border-color 0.3s;
        }
        input:focus {
            outline: none;
            border-color: #667eea;
        }
        .btn {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 15px 30px;
            border: none;
            border-radius: 8px;
            font-size: 18px;
            font-weight: 600;
            cursor: pointer;
            width: 100%;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        .btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 10px 20px rgba(102, 126, 234, 0.4);
        }
        .btn:disabled {
            opacity: 0.6;
            cursor: not-allowed;
        }
        .status {
            margin-top: 20px;
            padding: 15px;
            border-radius: 8px;
            display: none;
        }
        .status.show {
            display: block;
        }
        .status.success {
            background: #d4edda;
            color: #155724;
            border: 1px solid #c3e6cb;
        }
        .status.error {
            background: #f8d7da;
            color: #721c24;
            border: 1px solid #f5c6cb;
        }
        .status.info {
            background: #d1ecf1;
            color: #0c5460;
            border: 1px solid #bee5eb;
        }
        .progress {
            margin-top: 20px;
            display: none;
        }
        .progress.show {
            display: block;
        }
        .progress-bar {
            width: 100%;
            height: 30px;
            background: #e0e0e0;
            border-radius: 15px;
            overflow: hidden;
        }
        .progress-bar-fill {
            height: 100%;
            background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
            width: 0%;
            transition: width 0.3s;
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            font-weight: 600;
        }
        .results {
            margin-top: 30px;
            display: none;
        }
        .results.show {
            display: block;
        }
        .stat-card {
            background: #f8f9fa;
            padding: 20px;
            border-radius: 10px;
            margin-bottom: 15px;
            border-left: 4px solid #667eea;
        }
        .stat-label {
            color: #666;
            font-size: 0.9em;
            margin-bottom: 5px;
        }
        .stat-value {
            color: #333;
            font-size: 1.8em;
            font-weight: 700;
        }
        .emoji {
            font-size: 1.5em;
            margin-right: 10px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1><span class="emoji">🛍️</span>Etsy Web Scraper</h1>
        <p class="subtitle">Extract product data and reviews from Etsy categories</p>

        <form id="scrapeForm">
            <div class="input-group">
                <label for="categoryUrl">
                    <span class="emoji">🔗</span>Etsy Category URL
                </label>
                <input
                    type="text"
                    id="categoryUrl"
                    name="categoryUrl"
                    placeholder="https://www.etsy.com/se-en/c/bath-and-beauty/soaps/bath-salts-and-scrubs"
                    value="https://www.etsy.com/se-en/c/bath-and-beauty/soaps/bath-salts-and-scrubs"
                    required
                >
            </div>

            <div class="input-group">
                <label for="maxPages">
                    <span class="emoji">📄</span>Maximum Pages (0 = unlimited)
                </label>
                <input
                    type="number"
                    id="maxPages"
                    name="maxPages"
                    value="5"
                    min="0"
                    max="100"
                >
            </div>

            <button type="submit" class="btn" id="scrapeBtn">
                <span class="emoji">🚀</span>Start Scraping
            </button>
        </form>

        <div class="status" id="status"></div>

        <div class="progress" id="progress">
            <div class="progress-bar">
                <div class="progress-bar-fill" id="progressBar">Scraping...</div>
            </div>
        </div>

        <div class="results" id="results"></div>
    </div>

    <script>
        const form = document.getElementById('scrapeForm');
        const statusDiv = document.getElementById('status');
        const progressDiv = document.getElementById('progress');
        const progressBar = document.getElementById('progressBar');
        const resultsDiv = document.getElementById('results');
        const scrapeBtn = document.getElementById('scrapeBtn');

        form.addEventListener('submit', async (e) => {
            e.preventDefault();

            const categoryUrl = document.getElementById('categoryUrl').value;
            const maxPages = parseInt(document.getElementById('maxPages').value);

            // Reset UI
            statusDiv.className = 'status';
            resultsDiv.className = 'results';
            progressDiv.className = 'progress';

            // Show progress
            progressDiv.className = 'progress show';
            progressBar.style.width = '30%';
            scrapeBtn.disabled = true;

            try {
                const response = await fetch('/api/scrape', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        category_url: categoryUrl,
                        max_pages: maxPages
                    })
                });

                progressBar.style.width = '100%';

                const data = await response.json();

                if (data.success) {
                    showStatus('success', data.message);
                    displayResults(data.data);
                } else {
                    showStatus('error', data.message);
                }
            } catch (error) {
                showStatus('error', `Error: ${error.message}`);
            } finally {
                progressDiv.className = 'progress';
                scrapeBtn.disabled = false;
            }
        });

        function showStatus(type, message) {
            statusDiv.className = `status ${type} show`;
            statusDiv.textContent = message;
        }

        function displayResults(data) {
            if (!data) return;

            const html = `
                <h2 style="color: #667eea; margin-bottom: 20px;">
                    <span class="emoji">📊</span>Scraping Results
                </h2>

                <div class="stat-card">
                    <div class="stat-label">Total Products Found</div>
                    <div class="stat-value">${data.total_products}</div>
                </div>

                <div class="stat-card">
                    <div class="stat-label">Total Reviews Collected</div>
                    <div class="stat-value">${data.total_reviews}</div>
                </div>

                <div class="stat-card">
                    <div class="stat-label">Pages Scraped</div>
                    <div class="stat-value">${data.summary.pages_scraped}</div>
                </div>

                <div class="stat-card">
                    <div class="stat-label">Time Taken</div>
                    <div class="stat-value">${data.summary.time_taken_seconds}s</div>
                </div>

                ${data.summary.average_rating ? `
                <div class="stat-card">
                    <div class="stat-label">Average Rating</div>
                    <div class="stat-value">${data.summary.average_rating.toFixed(2)} ⭐</div>
                </div>
                ` : ''}

                <div style="margin-top: 20px; text-align: center;">
                    <p style="color: #666;">
                        Data saved to: <code>output/etsy_reviews.json</code>
                    </p>
                </div>
            `;

            resultsDiv.innerHTML = html;
            resultsDiv.className = 'results show';
        }
    </script>
</body>
</html>
"#;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

async fn scrape_api(
    state: web::Data<AppState>,
    req: web::Json<ScrapeRequest>,
) -> Result<HttpResponse> {
    log::info!("Received scrape request for: {}", req.category_url);

    // Create scraper
    let scraper = match EtsyScraper::new(true) {
        Ok(s) => s,
        Err(e) => {
            return Ok(HttpResponse::Ok().json(ScrapeResponse {
                success: false,
                message: format!("Failed to create scraper: {}", e),
                data: None,
            }));
        }
    };

    // Scrape
    match scraper.scrape_category(&req.category_url, req.max_pages).await {
        Ok(result) => {
            // Save to file
            let json = serde_json::to_string_pretty(&result)?;
            std::fs::create_dir_all("output")?;
            std::fs::write("output/etsy_reviews.json", json)?;

            // Store in state
            *state.results.lock().unwrap() = Some(result.clone());

            log::info!(
                "Scraping complete: {} products, {} reviews",
                result.total_products,
                result.total_reviews
            );

            Ok(HttpResponse::Ok().json(ScrapeResponse {
                success: true,
                message: format!(
                    "Successfully scraped {} products with {} reviews!",
                    result.total_products, result.total_reviews
                ),
                data: Some(result),
            }))
        }
        Err(e) => {
            log::error!("Scraping failed: {}", e);
            Ok(HttpResponse::Ok().json(ScrapeResponse {
                success: false,
                message: format!("Scraping failed: {}", e),
                data: None,
            }))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let state = web::Data::new(AppState {
        results: Arc::new(Mutex::new(None)),
    });

    log::info!("Starting Etsy Scraper Web Server at http://localhost:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .route("/", web::get().to(index))
            .route("/api/scrape", web::post().to(scrape_api))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
