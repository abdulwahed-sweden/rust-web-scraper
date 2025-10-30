use actix_cors::Cors;
use actix_files as fs;
use actix_web::{middleware, web, App, HttpServer};
use std::sync::{Arc, Mutex};

mod api;
mod auto_selectors;
mod deep_scraper;
mod learning_profile;
mod scraper;
mod structure_analyzer;
mod utils;

use api::AppState;
use learning_profile::ProfileDatabase;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Initialize profile database
    let db_path = std::env::var("PROFILE_DB_PATH")
        .unwrap_or_else(|_| "./profiles.db".to_string());

    let profile_db = ProfileDatabase::new(&db_path)
        .expect("Failed to initialize profile database");

    log::info!("📊 Profile database initialized at: {}", db_path);

    let state = web::Data::new(AppState {
        sessions: Arc::new(Mutex::new(Vec::new())),
        profiles: Arc::new(Mutex::new(profile_db)),
    });

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    log::info!("🚀 Starting Rust Web Scraper");
    log::info!("🌐 Server running at http://{}:{}", host, port);
    log::info!("📖 API docs available at http://{}:{}/api/health", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(cors)
            // API routes
            .route("/api/health", web::get().to(api::health_check))
            .route("/api/scrape", web::post().to(api::scrape_handler))
            .route("/api/deep-scrape", web::post().to(api::deep_scrape_handler))
            .route("/api/analyze", web::post().to(api::analyze_handler))
            .route("/api/sessions", web::get().to(api::get_sessions))
            .route("/api/sessions", web::delete().to(api::clear_sessions))
            .route("/api/sessions/{id}", web::get().to(api::get_session))
            // Profile management routes
            .route("/api/profiles", web::get().to(api::get_profiles))
            .route("/api/profiles/stats", web::get().to(api::get_profile_stats))
            .route("/api/profiles", web::delete().to(api::clear_profiles))
            .route("/api/profiles/{id}", web::get().to(api::get_profile))
            .route("/api/profiles/{id}", web::delete().to(api::delete_profile))
            .route("/api/profiles/domain/{domain}", web::get().to(api::get_profile_by_domain))
            // Serve static files
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // Serve index.html at root
            .route("/", web::get().to(|| async {
                actix_files::NamedFile::open_async("./static/index.html").await
            }))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
