use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::scraper::{ScrapingConfig, ScrapingSession, WebScraper};
use crate::structure_analyzer::{StructureAnalysis, StructureAnalyzer};
use crate::utils::get_random_user_agent;

#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<Mutex<Vec<ScrapingSession>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapeRequest {
    pub urls: Vec<String>,
    #[serde(default)]
    pub enable_pagination: bool,
    #[serde(default)]
    pub max_pages: usize,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: f64,
    #[serde(default)]
    pub custom_selectors: Option<crate::auto_selectors::AutoSelectors>,
}

fn default_rate_limit() -> f64 {
    2.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapeResponse {
    pub success: bool,
    pub message: String,
    pub session: Option<ScrapingSession>,
}

pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "service": "rust-web-scraper"
    })))
}

pub async fn scrape_handler(
    state: web::Data<AppState>,
    req: web::Json<ScrapeRequest>,
) -> Result<HttpResponse> {
    log::info!("Received scrape request for {} URL(s)", req.urls.len());

    let config = ScrapingConfig {
        urls: req.urls.clone(),
        enable_pagination: req.enable_pagination,
        max_pages: req.max_pages,
        rate_limit: req.rate_limit,
        custom_selectors: req.custom_selectors.clone(),
    };

    let scraper = match WebScraper::new(&config, true) {
        Ok(s) => s,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ScrapeResponse {
                success: false,
                message: format!("Failed to create scraper: {}", e),
                session: None,
            }));
        }
    };

    match scraper.scrape(config).await {
        Ok(session) => {
            // Store session in state
            state.sessions.lock().unwrap().push(session.clone());

            log::info!(
                "Scraping complete: {} pages scraped, {} links found",
                session.total_pages_scraped,
                session.total_links_found
            );

            Ok(HttpResponse::Ok().json(ScrapeResponse {
                success: true,
                message: format!(
                    "Successfully scraped {} pages with {} links and {} images",
                    session.total_pages_scraped,
                    session.total_links_found,
                    session.total_images_found
                ),
                session: Some(session),
            }))
        }
        Err(e) => {
            log::error!("Scraping failed: {}", e);
            Ok(HttpResponse::Ok().json(ScrapeResponse {
                success: false,
                message: format!("Scraping failed: {}", e),
                session: None,
            }))
        }
    }
}

pub async fn get_sessions(state: web::Data<AppState>) -> Result<HttpResponse> {
    let sessions = state.sessions.lock().unwrap();
    Ok(HttpResponse::Ok().json(&*sessions))
}

pub async fn get_session(
    state: web::Data<AppState>,
    path: web::Path<usize>,
) -> Result<HttpResponse> {
    let index = path.into_inner();
    let sessions = state.sessions.lock().unwrap();

    if index < sessions.len() {
        Ok(HttpResponse::Ok().json(&sessions[index]))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Session not found"
        })))
    }
}

pub async fn clear_sessions(state: web::Data<AppState>) -> Result<HttpResponse> {
    state.sessions.lock().unwrap().clear();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "All sessions cleared"
    })))
}

// Structure Analysis API

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    pub url: String,
    #[serde(default)]
    pub min_content_length: Option<usize>,
    #[serde(default)]
    pub detect_comments: bool,
    #[serde(default)]
    pub debug_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeResponse {
    pub success: bool,
    pub message: String,
    pub analysis: Option<StructureAnalysis>,
}

pub async fn analyze_handler(req: web::Json<AnalyzeRequest>) -> Result<HttpResponse> {
    log::info!("Received structure analysis request for: {}", req.url);

    // Fetch the page
    let user_agent = get_random_user_agent();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            log::error!("Failed to create HTTP client: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    let response = client
        .get(&req.url)
        .header("User-Agent", user_agent)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to fetch URL: {}", e);
            actix_web::error::ErrorBadRequest(format!("Failed to fetch URL: {}", e))
        })?;

    if !response.status().is_success() {
        return Ok(HttpResponse::Ok().json(AnalyzeResponse {
            success: false,
            message: format!("HTTP error: {}", response.status()),
            analysis: None,
        }));
    }

    let html = response.text().await.map_err(|e| {
        log::error!("Failed to read response body: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    // Analyze structure
    let analyzer = if let Some(min_len) = req.min_content_length {
        StructureAnalyzer::with_options(min_len, req.detect_comments, req.debug_mode)
    } else {
        StructureAnalyzer::new()
    };

    let analysis = analyzer.analyze(&html, &req.url);

    log::info!(
        "Analysis complete: {} sections found, confidence: {:?}",
        analysis.sections.len(),
        analysis.recommendations.confidence_level
    );

    Ok(HttpResponse::Ok().json(AnalyzeResponse {
        success: true,
        message: format!(
            "Successfully analyzed structure: {} sections found",
            analysis.sections.len()
        ),
        analysis: Some(analysis),
    }))
}
