use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

use crate::auto_selectors::{AutoSelectors, DetectedContent, SelectorDetector};
use crate::utils::{get_random_user_agent, RateLimiter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingConfig {
    pub urls: Vec<String>,
    #[serde(default)]
    pub enable_pagination: bool,
    #[serde(default)]
    pub max_pages: usize,
    #[serde(default)]
    pub rate_limit: f64,
    #[serde(default)]
    pub custom_selectors: Option<AutoSelectors>,
}

impl Default for ScrapingConfig {
    fn default() -> Self {
        Self {
            urls: Vec::new(),
            enable_pagination: false,
            max_pages: 0,
            rate_limit: 2.0,
            custom_selectors: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingResult {
    pub url: String,
    pub timestamp: String,
    pub status: String,
    pub content: DetectedContent,
    pub page_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingSession {
    pub start_time: String,
    pub config: ScrapingConfig,
    pub results: Vec<ScrapingResult>,
    pub total_pages_scraped: usize,
    pub total_links_found: usize,
    pub total_images_found: usize,
    pub errors: Vec<String>,
}

pub struct WebScraper {
    client: reqwest::Client,
    rate_limiter: RateLimiter,
    detector: SelectorDetector,
    verbose: bool,
}

impl WebScraper {
    pub fn new(config: &ScrapingConfig, verbose: bool) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .build()?;

        let detector = if let Some(ref custom) = config.custom_selectors {
            SelectorDetector::with_custom_selectors(custom.clone())
        } else {
            SelectorDetector::new()
        };

        Ok(Self {
            client,
            rate_limiter: RateLimiter::new(config.rate_limit),
            detector,
            verbose,
        })
    }

    pub async fn scrape(&self, config: ScrapingConfig) -> Result<ScrapingSession> {
        let start_time = Local::now().to_rfc3339();
        let mut all_results = Vec::new();
        let mut errors = Vec::new();

        for url in &config.urls {
            if config.enable_pagination {
                match self.scrape_with_pagination(url, config.max_pages).await {
                    Ok(mut results) => all_results.append(&mut results),
                    Err(e) => errors.push(format!("Failed to scrape {}: {}", url, e)),
                }
            } else {
                match self.scrape_single_page(url, 1).await {
                    Ok(result) => all_results.push(result),
                    Err(e) => errors.push(format!("Failed to scrape {}: {}", url, e)),
                }
            }
        }

        let total_links_found = all_results.iter().map(|r| r.content.links.len()).sum();
        let total_images_found = all_results.iter().map(|r| r.content.images.len()).sum();

        Ok(ScrapingSession {
            start_time,
            config,
            total_pages_scraped: all_results.len(),
            total_links_found,
            total_images_found,
            results: all_results,
            errors,
        })
    }

    async fn scrape_single_page(&self, url: &str, page_number: usize) -> Result<ScrapingResult> {
        if self.verbose {
            log::info!("Scraping: {}", url);
        }

        self.rate_limiter.wait().await;

        let html = self.fetch_page(url).await?;
        let content = self.detector.detect(&html, url);

        Ok(ScrapingResult {
            url: url.to_string(),
            timestamp: Local::now().to_rfc3339(),
            status: "success".to_string(),
            content,
            page_number,
        })
    }

    async fn scrape_with_pagination(&self, start_url: &str, max_pages: usize) -> Result<Vec<ScrapingResult>> {
        let mut results = Vec::new();
        let mut visited_urls = HashSet::new();
        let mut current_url = start_url.to_string();
        let mut page_count = 0;

        let effective_max_pages = if max_pages > 0 { max_pages } else { usize::MAX };

        loop {
            if visited_urls.contains(&current_url) || page_count >= effective_max_pages {
                break;
            }

            visited_urls.insert(current_url.clone());
            page_count += 1;

            if self.verbose {
                log::info!("Page {}/{}: {}", page_count,
                    if max_pages > 0 { max_pages.to_string() } else { "∞".to_string() },
                    current_url
                );
            }

            match self.scrape_single_page(&current_url, page_count).await {
                Ok(result) => {
                    // Try to find next page link
                    let next_url = self.find_next_page(&result.content, &current_url);
                    results.push(result);

                    if let Some(next) = next_url {
                        current_url = next;
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    log::error!("Failed to scrape {}: {}", current_url, e);
                    break;
                }
            }
        }

        Ok(results)
    }

    async fn fetch_page(&self, url: &str) -> Result<String> {
        let user_agent = get_random_user_agent();

        let response = self.client
            .get(url)
            .header("User-Agent", user_agent)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .send()
            .await
            .context("Failed to fetch page")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        response.text().await.context("Failed to read response body")
    }

    fn find_next_page(&self, content: &DetectedContent, current_url: &str) -> Option<String> {
        // Look for common pagination patterns
        let pagination_keywords = ["next", "next page", "→", "»", "›"];

        for link in &content.links {
            let text_lower = link.text.to_lowercase();

            // Check if link text contains pagination keywords
            if pagination_keywords.iter().any(|&kw| text_lower.contains(kw)) {
                // Ensure it's not an external link and not the current page
                if !link.is_external && link.href != current_url {
                    return Some(link.href.clone());
                }
            }

            // Check for rel="next" pattern in href
            if link.href.contains("page=") || link.href.contains("p=") {
                if let (Ok(current), Ok(next)) = (Url::parse(current_url), Url::parse(&link.href)) {
                    if current.host() == next.host() && current.path() == next.path() {
                        return Some(link.href.clone());
                    }
                }
            }
        }

        None
    }
}

use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scraping_config_default() {
        let config = ScrapingConfig::default();
        assert_eq!(config.rate_limit, 2.0);
        assert!(!config.enable_pagination);
    }

    #[test]
    fn test_scraping_config_from_json() {
        let json = r#"{
            "urls": ["https://example.com"],
            "enable_pagination": true,
            "max_pages": 5
        }"#;

        let config: ScrapingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.urls.len(), 1);
        assert!(config.enable_pagination);
        assert_eq!(config.max_pages, 5);
    }
}
