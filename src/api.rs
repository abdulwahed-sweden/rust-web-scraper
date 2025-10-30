use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::scraper::{ScrapingConfig, ScrapingSession, WebScraper};

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
