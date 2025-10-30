use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::learning_profile::{ProfileDatabase, SiteProfile};
use crate::scraper::{ScrapingConfig, ScrapingSession, WebScraper};
use crate::structure_analyzer::{StructureAnalysis, StructureAnalyzer};
use crate::utils::get_random_user_agent;

#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<Mutex<Vec<ScrapingSession>>>,
    pub profiles: Arc<Mutex<ProfileDatabase>>,
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

// Profile Management API

pub async fn get_profiles(state: web::Data<AppState>) -> Result<HttpResponse> {
    let profiles_guard = state.profiles.lock().unwrap();
    match profiles_guard.get_all() {
        Ok(profiles) => Ok(HttpResponse::Ok().json(profiles)),
        Err(e) => {
            log::error!("Failed to get profiles: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve profiles: {}", e)
            })))
        }
    }
}

pub async fn get_profile(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    let profiles = state.profiles.lock().unwrap();

    match profiles.get_by_id(&id) {
        Ok(Some(profile)) => Ok(HttpResponse::Ok().json(profile)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Profile not found"
        }))),
        Err(e) => {
            log::error!("Failed to get profile: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve profile: {}", e)
            })))
        }
    }
}

pub async fn get_profile_by_domain(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let domain = path.into_inner();
    let profiles = state.profiles.lock().unwrap();

    match profiles.get_by_domain(&domain) {
        Ok(Some(profile)) => Ok(HttpResponse::Ok().json(profile)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "No profile found for domain"
        }))),
        Err(e) => {
            log::error!("Failed to get profile by domain: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve profile: {}", e)
            })))
        }
    }
}

pub async fn delete_profile(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    let profiles = state.profiles.lock().unwrap();

    match profiles.delete(&id) {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Profile deleted successfully"
        }))),
        Err(e) => {
            log::error!("Failed to delete profile: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete profile: {}", e)
            })))
        }
    }
}

pub async fn get_profile_stats(state: web::Data<AppState>) -> Result<HttpResponse> {
    let profiles = state.profiles.lock().unwrap();

    match profiles.get_stats() {
        Ok(stats) => Ok(HttpResponse::Ok().json(stats)),
        Err(e) => {
            log::error!("Failed to get stats: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve stats: {}", e)
            })))
        }
    }
}

pub async fn clear_profiles(state: web::Data<AppState>) -> Result<HttpResponse> {
    let profiles = state.profiles.lock().unwrap();

    match profiles.clear_all() {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "All profiles cleared successfully"
        }))),
        Err(e) => {
            log::error!("Failed to clear profiles: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to clear profiles: {}", e)
            })))
        }
    }
}

pub async fn analyze_handler(
    state: web::Data<AppState>,
    req: web::Json<AnalyzeRequest>
) -> Result<HttpResponse> {
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

    // Auto-save profile if confidence is high enough
    if analysis.recommendations.best_main_content.is_some() {
        let confidence_threshold = 0.5;
        let top_score = analysis.sections.first().map(|s| s.score).unwrap_or(0.0);

        if top_score >= confidence_threshold {
            let profiles = state.profiles.lock().unwrap();
            match profiles.save_from_analysis(&analysis) {
                Ok(profile) => {
                    log::info!("Auto-saved profile for {} (confidence: {:.2})",
                        profile.domain, profile.confidence);
                }
                Err(e) => {
                    log::warn!("Failed to auto-save profile: {}", e);
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(AnalyzeResponse {
        success: true,
        message: format!(
            "Successfully analyzed structure: {} sections found",
            analysis.sections.len()
        ),
        analysis: Some(analysis),
    }))
}

// Deep Scraping Handlers

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepScrapeRequest {
    pub start_urls: Vec<String>,
    #[serde(default = "default_depth")]
    pub max_depth: usize,
    #[serde(default = "default_max_pages")]
    pub max_pages: usize,
    #[serde(default = "default_stay_in_domain")]
    pub stay_in_domain: bool,
    #[serde(default)]
    pub stay_in_subdomain: bool,
    #[serde(default)]
    pub include_patterns: Vec<String>,
    #[serde(default = "default_exclude_patterns")]
    pub exclude_patterns: Vec<String>,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: f64,
    #[serde(default)]
    pub custom_selectors: Option<crate::auto_selectors::AutoSelectors>,
    #[serde(default = "default_filter_navigation")]
    pub filter_navigation: bool,
    #[serde(default = "default_min_content_length")]
    pub min_content_length: usize,
}

fn default_depth() -> usize { 2 }
fn default_max_pages() -> usize { 50 }
fn default_stay_in_domain() -> bool { true }
fn default_filter_navigation() -> bool { true }
fn default_min_content_length() -> usize { 200 }
fn default_exclude_patterns() -> Vec<String> {
    vec![
        r"\.pdf$".to_string(),
        r"\.zip$".to_string(),
        r"\.jpg$".to_string(),
        r"\.png$".to_string(),
        r"\.gif$".to_string(),
        r"\#.*$".to_string(),
    ]
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepScrapeResponse {
    pub success: bool,
    pub message: String,
    pub result: Option<crate::deep_scraper::DeepScrapeResult>,
}

pub async fn deep_scrape_handler(
    req: web::Json<DeepScrapeRequest>,
) -> Result<HttpResponse> {
    log::info!("Received deep scrape request: {} start URLs, max depth: {}",
        req.start_urls.len(), req.max_depth);

    // Create config
    let config = crate::deep_scraper::DeepScrapeConfig {
        start_urls: req.start_urls.clone(),
        max_depth: req.max_depth,
        max_pages: req.max_pages,
        stay_in_domain: req.stay_in_domain,
        stay_in_subdomain: req.stay_in_subdomain,
        include_patterns: req.include_patterns.clone(),
        exclude_patterns: req.exclude_patterns.clone(),
        rate_limit: req.rate_limit,
        custom_selectors: req.custom_selectors.clone(),
        filter_navigation: req.filter_navigation,
        min_content_length: req.min_content_length,
    };

    // Create deep scraper
    let mut scraper = crate::deep_scraper::DeepScraper::new(config);

    // Execute deep scrape
    let result = scraper.scrape().await;

    let success = result.status == crate::deep_scraper::CrawlStatus::Completed ||
                 result.status == crate::deep_scraper::CrawlStatus::PartiallyCompleted;

    Ok(HttpResponse::Ok().json(DeepScrapeResponse {
        success,
        message: format!(
            "Deep scrape {}: {} pages crawled, {} links discovered",
            if success { "completed" } else { "failed" },
            result.total_pages_crawled,
            result.total_links_discovered
        ),
        result: Some(result),
    }))
}
