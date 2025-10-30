// Rust Web Scraper Library
//
// A powerful, intelligent web scraping library with automatic content detection,
// rate limiting, and support for pagination.

pub mod auto_selectors;
pub mod scraper;
pub mod structure_analyzer;
pub mod utils;

// Re-export main types for convenience
pub use auto_selectors::{AutoSelectors, DetectedContent, ImageData, LinkData, SelectorDetector};
pub use scraper::{ScrapingConfig, ScrapingResult, ScrapingSession, WebScraper};
pub use structure_analyzer::{
    StructureAnalysis, StructureAnalyzer, Section, SectionType,
    Recommendations, ExtractionMode, ConfidenceLevel
};
pub use utils::{get_random_user_agent, RateLimiter, USER_AGENTS};
