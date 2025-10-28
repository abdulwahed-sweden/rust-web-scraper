use anyhow::{Context, Result};
use chrono::Local;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

use crate::rate_limit::RateLimiter;
use crate::user_agents::get_random_user_agent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtsyProduct {
    pub name: String,
    pub price: String,
    pub rating: Option<String>,
    pub review_count: Option<String>,
    pub product_url: String,
    pub image_url: Option<String>,
    pub reviews: Vec<Review>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub text: String,
    pub reviewer_name: Option<String>,
    pub rating: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtsyScrapingResult {
    pub timestamp: String,
    pub category_url: String,
    pub total_products: usize,
    pub total_reviews: usize,
    pub products: Vec<EtsyProduct>,
    pub summary: ScrapingSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingSummary {
    pub pages_scraped: usize,
    pub products_with_reviews: usize,
    pub average_rating: Option<f64>,
    pub time_taken_seconds: u64,
}

pub struct EtsyScraper {
    client: reqwest::Client,
    rate_limiter: RateLimiter,
    verbose: bool,
    fetch_reviews: bool,
}

impl EtsyScraper {
    pub fn new(verbose: bool) -> Result<Self> {
        Self::with_options(verbose, false)
    }

    pub fn with_options(verbose: bool, fetch_reviews: bool) -> Result<Self> {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            rate_limiter: RateLimiter::default(),
            verbose,
            fetch_reviews,
        })
    }

    pub async fn scrape_category(
        &self,
        category_url: &str,
        max_pages: usize,
    ) -> Result<EtsyScrapingResult> {
        let start_time = std::time::Instant::now();
        let mut all_products = Vec::new();
        let mut visited_urls = HashSet::new();
        let mut current_url = category_url.to_string();
        let mut page_count = 0;

        log::info!("Starting Etsy scraping from: {}", category_url);

        loop {
            if visited_urls.contains(&current_url) {
                if self.verbose {
                    println!("  Already visited: {}", current_url);
                }
                break;
            }

            if max_pages > 0 && page_count >= max_pages {
                if self.verbose {
                    println!("  Reached maximum page limit: {}", max_pages);
                }
                break;
            }

            visited_urls.insert(current_url.clone());
            page_count += 1;

            if self.verbose {
                println!("\n[Page {}/{}] Fetching: {}",
                    page_count,
                    if max_pages > 0 { max_pages.to_string() } else { "∞".to_string() },
                    current_url
                );
            }

            // Rate limiting
            self.rate_limiter.wait().await;

            // Fetch page
            match self.fetch_page(&current_url).await {
                Ok(html) => {
                    // Extract products from this page
                    let products = self.extract_products(&html, &current_url).await?;

                    if self.verbose {
                        println!("  ✓ Found {} products on this page", products.len());
                    }

                    all_products.extend(products);

                    // Try to find next page
                    match self.extract_next_page_url(&html, &current_url)? {
                        Some(next_url) => {
                            if self.verbose {
                                println!("  → Next page: {}", next_url);
                            }
                            current_url = next_url;
                        }
                        None => {
                            if self.verbose {
                                println!("  No more pages found");
                            }
                            break;
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to fetch page {}: {}", current_url, e);
                    break;
                }
            }
        }

        let elapsed = start_time.elapsed();
        let total_reviews: usize = all_products.iter().map(|p| p.reviews.len()).sum();

        let summary = ScrapingSummary {
            pages_scraped: page_count,
            products_with_reviews: all_products.iter().filter(|p| !p.reviews.is_empty()).count(),
            average_rating: self.calculate_average_rating(&all_products),
            time_taken_seconds: elapsed.as_secs(),
        };

        Ok(EtsyScrapingResult {
            timestamp: Local::now().to_rfc3339(),
            category_url: category_url.to_string(),
            total_products: all_products.len(),
            total_reviews,
            products: all_products,
            summary,
        })
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

    async fn extract_products(&self, html: &str, base_url: &str) -> Result<Vec<EtsyProduct>> {
        let document = Html::parse_document(html);
        let mut products = Vec::new();

        // Etsy product selectors (these may need adjustment based on actual HTML structure)
        // These are common patterns - adjust based on actual Etsy HTML
        let product_selectors = vec![
            "div.wt-grid__item-xs-6",  // Common Etsy grid item
            "div[data-appears-component-name*='listing']",
            "li.wt-list-unstyled",
            "article",
        ];

        let mut product_selector_str = product_selectors[0];
        for selector_str in &product_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if document.select(&selector).next().is_some() {
                    product_selector_str = selector_str;
                    break;
                }
            }
        }

        let product_selector = Selector::parse(product_selector_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse product selector: {:?}", e))?;

        for (index, element) in document.select(&product_selector).enumerate().take(50) {
            if self.verbose && index < 5 {
                println!("    Processing product {}...", index + 1);
            }

            let product = self.extract_product_info(element.html().as_str(), base_url).await;

            if let Ok(prod) = product {
                products.push(prod);
            }
        }

        Ok(products)
    }

    async fn extract_product_info(&self, html: &str, base_url: &str) -> Result<EtsyProduct> {
        let document = Html::parse_document(html);

        // Extract product name
        let name = self.extract_text(&document, &["h3", "h2", "a[title]", ".v2-listing-card__title"])
            .unwrap_or_else(|| "Unknown Product".to_string());

        // Extract price
        let price = self.extract_text(&document, &[".currency-value", ".wt-text-title-01", "span.currency-symbol"])
            .unwrap_or_else(|| "N/A".to_string());

        // Extract rating
        let rating = self.extract_text(&document, &["[data-rating]", ".stars", ".wt-display-inline-block"]);

        // Extract review count
        let review_count = self.extract_text(&document, &["[aria-label*='reviews']", ".wt-text-caption"]);

        // Extract product URL
        let product_url = self.extract_attr(&document, &["a"], "href", base_url)
            .unwrap_or_else(|| base_url.to_string());

        // Extract image URL
        let image_url = self.extract_attr(&document, &["img"], "src", base_url);

        // Fetch reviews if enabled
        let reviews = if self.fetch_reviews {
            // Extract listing ID from product URL
            if let Some(listing_id) = crate::etsy_reviews::extract_listing_id(&product_url) {
                match crate::etsy_reviews::fetch_reviews(
                    &self.client,
                    &listing_id,
                    &self.rate_limiter,
                    self.verbose,
                )
                .await
                {
                    Ok(reviews) => reviews,
                    Err(e) => {
                        if self.verbose {
                            println!("        ⚠ Failed to fetch reviews: {}", e);
                        }
                        Vec::new()
                    }
                }
            } else {
                if self.verbose {
                    println!("        ⚠ Could not extract listing ID from URL: {}", product_url);
                }
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(EtsyProduct {
            name: name.trim().to_string(),
            price: price.trim().to_string(),
            rating,
            review_count,
            product_url,
            image_url,
            reviews,
        })
    }

    fn extract_text(&self, document: &Html, selectors: &[&str]) -> Option<String> {
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let text: String = element.text().collect();
                    if !text.trim().is_empty() {
                        return Some(text.trim().to_string());
                    }
                }
            }
        }
        None
    }

    fn extract_attr(&self, document: &Html, selectors: &[&str], attr: &str, base_url: &str) -> Option<String> {
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(value) = element.value().attr(attr) {
                        // Convert to absolute URL if needed
                        if let Ok(base) = Url::parse(base_url) {
                            if let Ok(absolute_url) = base.join(value) {
                                return Some(absolute_url.to_string());
                            }
                        }
                        return Some(value.to_string());
                    }
                }
            }
        }
        None
    }

    fn extract_next_page_url(&self, html: &str, current_url: &str) -> Result<Option<String>> {
        let document = Html::parse_document(html);

        let next_selectors = vec![
            "a.wt-action-group__item-container[aria-label*='Next']",
            "a[rel='next']",
            "a.pagination-next",
            "li.pagination-next a",
        ];

        for selector_str in &next_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(href) = element.value().attr("href") {
                        let base_url = Url::parse(current_url)?;
                        let next_url = base_url.join(href)?;
                        return Ok(Some(next_url.to_string()));
                    }
                }
            }
        }

        Ok(None)
    }

    fn calculate_average_rating(&self, products: &[EtsyProduct]) -> Option<f64> {
        let ratings: Vec<f64> = products
            .iter()
            .filter_map(|p| p.rating.as_ref())
            .filter_map(|r| r.parse::<f64>().ok())
            .collect();

        if ratings.is_empty() {
            None
        } else {
            Some(ratings.iter().sum::<f64>() / ratings.len() as f64)
        }
    }
}
