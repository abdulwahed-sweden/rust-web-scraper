use crate::auto_selectors::AutoSelectors;
use crate::scraper::{ScrapingConfig, ScrapingResult, WebScraper};
use crate::utils::normalize_url;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use url::Url;

/// Configuration for deep scraping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepScrapeConfig {
    /// Starting URLs
    pub start_urls: Vec<String>,

    /// Maximum depth to crawl (0 = only start URLs)
    pub max_depth: usize,

    /// Maximum total pages to scrape
    pub max_pages: usize,

    /// Only follow links within same domain
    pub stay_in_domain: bool,

    /// Only follow links within same subdomain
    pub stay_in_subdomain: bool,

    /// Patterns to include in URLs (regex)
    pub include_patterns: Vec<String>,

    /// Patterns to exclude from URLs (regex)
    pub exclude_patterns: Vec<String>,

    /// Rate limit (requests per second per domain)
    pub rate_limit: f64,

    /// Custom selectors for extraction
    pub custom_selectors: Option<AutoSelectors>,

    /// Filter out navigation/footer links
    pub filter_navigation: bool,

    /// Minimum content length to consider a page valuable
    pub min_content_length: usize,
}

impl Default for DeepScrapeConfig {
    fn default() -> Self {
        Self {
            start_urls: Vec::new(),
            max_depth: 2,
            max_pages: 50,
            stay_in_domain: true,
            stay_in_subdomain: false,
            include_patterns: Vec::new(),
            exclude_patterns: vec![
                r"\.pdf$".to_string(),
                r"\.zip$".to_string(),
                r"\.jpg$".to_string(),
                r"\.png$".to_string(),
                r"\.gif$".to_string(),
                r"\#.*$".to_string(), // Fragment URLs
            ],
            rate_limit: 2.0,
            custom_selectors: None,
            filter_navigation: true,
            min_content_length: 200,
        }
    }
}

/// Represents a URL in the crawl queue
#[derive(Debug, Clone)]
struct CrawlItem {
    url: String,
    depth: usize,
    parent_url: Option<String>,
}

/// Link scoring for intelligent filtering
#[derive(Debug, Clone)]
pub struct LinkScore {
    pub url: String,
    pub score: f64,
    pub is_navigation: bool,
    pub is_external: bool,
}

/// Result of a deep scraping session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepScrapeResult {
    pub session_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub config: DeepScrapeConfig,
    pub results: Vec<ScrapingResult>,
    pub crawl_tree: Vec<CrawlNode>,
    pub total_pages_crawled: usize,
    pub total_links_discovered: usize,
    pub total_links_filtered: usize,
    pub domains_visited: Vec<String>,
    pub errors: Vec<String>,
    pub status: CrawlStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlNode {
    pub url: String,
    pub depth: usize,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub scraped: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CrawlStatus {
    Running,
    Completed,
    PartiallyCompleted,
    Failed,
}

/// Deep scraper engine
pub struct DeepScraper {
    config: DeepScrapeConfig,
    visited: Arc<Mutex<HashSet<String>>>,
    queue: Arc<Mutex<VecDeque<CrawlItem>>>,
    results: Arc<Mutex<Vec<ScrapingResult>>>,
    crawl_tree: Arc<Mutex<Vec<CrawlNode>>>,
    errors: Arc<Mutex<Vec<String>>>,
}

impl DeepScraper {
    pub fn new(config: DeepScrapeConfig) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));

        // Initialize queue with start URLs
        {
            let mut q = queue.lock().unwrap();
            for url in &config.start_urls {
                q.push_back(CrawlItem {
                    url: url.clone(),
                    depth: 0,
                    parent_url: None,
                });
            }
        }

        Self {
            config,
            visited: Arc::new(Mutex::new(HashSet::new())),
            queue,
            results: Arc::new(Mutex::new(Vec::new())),
            crawl_tree: Arc::new(Mutex::new(Vec::new())),
            errors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Execute the deep scraping
    pub async fn scrape(&mut self) -> DeepScrapeResult {
        let session_id = uuid::Uuid::new_v4().to_string();
        let start_time = chrono::Utc::now().to_rfc3339();

        log::info!("🔍 Starting deep scrape: {} URLs, max depth: {}",
            self.config.start_urls.len(), self.config.max_depth);

        let mut pages_crawled = 0;
        let mut links_discovered = 0;
        let mut links_filtered = 0;

        while pages_crawled < self.config.max_pages {
            // Get next URL from queue
            let item = {
                let mut queue = self.queue.lock().unwrap();
                queue.pop_front()
            };

            let item = match item {
                Some(i) => i,
                None => break, // Queue empty, done
            };

            // Skip if already visited
            {
                let mut visited = self.visited.lock().unwrap();
                if visited.contains(&item.url) {
                    continue;
                }
                visited.insert(item.url.clone());
            }

            log::info!("📄 Scraping [depth {}]: {}", item.depth, item.url);

            // Scrape the page
            match self.scrape_page(&item).await {
                Ok((result, links)) => {
                    links_discovered += links.len();

                    // Filter and queue links
                    if item.depth < self.config.max_depth {
                        let filtered_links = self.filter_links(&item.url, links);
                        links_filtered += links_discovered - filtered_links.len();

                        self.enqueue_links(&item.url, &filtered_links, item.depth + 1);
                    }

                    // Store result
                    let mut results = self.results.lock().unwrap();
                    results.push(result);
                    pages_crawled += 1;

                    // Update crawl tree
                    self.update_crawl_tree(&item, None);
                }
                Err(e) => {
                    log::error!("❌ Failed to scrape {}: {}", item.url, e);
                    let mut errors = self.errors.lock().unwrap();
                    errors.push(format!("{}: {}", item.url, e));

                    // Mark as error in crawl tree
                    self.update_crawl_tree(&item, Some(e.to_string()));
                }
            }

            // Respect rate limit
            let delay = std::time::Duration::from_secs_f64(1.0 / self.config.rate_limit);
            tokio::time::sleep(delay).await;
        }

        let end_time = chrono::Utc::now().to_rfc3339();
        let status = self.determine_status(pages_crawled);

        log::info!("✅ Deep scrape completed: {} pages, {} links discovered",
            pages_crawled, links_discovered);

        DeepScrapeResult {
            session_id,
            start_time,
            end_time: Some(end_time),
            config: self.config.clone(),
            results: self.results.lock().unwrap().clone(),
            crawl_tree: self.crawl_tree.lock().unwrap().clone(),
            total_pages_crawled: pages_crawled,
            total_links_discovered: links_discovered,
            total_links_filtered: links_filtered,
            domains_visited: self.get_domains_visited(),
            errors: self.errors.lock().unwrap().clone(),
            status,
        }
    }

    /// Scrape a single page
    async fn scrape_page(&self, item: &CrawlItem) -> Result<(ScrapingResult, Vec<String>), Box<dyn std::error::Error>> {
        // Create scraper config
        let scraper_config = ScrapingConfig {
            urls: vec![item.url.clone()],
            enable_pagination: false,
            max_pages: 1,
            rate_limit: self.config.rate_limit,
            custom_selectors: self.config.custom_selectors.clone(),
        };

        // Perform scrape
        let scraper = WebScraper::new(&scraper_config, false)?;
        let session = scraper.scrape(scraper_config).await?;

        if session.results.is_empty() {
            return Err("No results returned".into());
        }

        let result = session.results[0].clone();

        // Extract all links
        let links: Vec<String> = result.content.links.iter()
            .map(|link| link.href.clone())
            .collect();

        Ok((result, links))
    }

    /// Filter links based on config rules
    fn filter_links(&self, base_url: &str, links: Vec<String>) -> Vec<String> {
        let base_url_parsed = match Url::parse(base_url) {
            Ok(u) => u,
            Err(_) => return Vec::new(),
        };

        links.into_iter()
            .filter_map(|link| {
                // Resolve relative URLs
                let absolute_url = match base_url_parsed.join(&link) {
                    Ok(u) => u.to_string(),
                    Err(_) => return None,
                };

                // Normalize URL
                let normalized = normalize_url(&absolute_url);

                // Apply filters
                if !self.should_crawl(&normalized, &base_url_parsed) {
                    return None;
                }

                Some(normalized)
            })
            .collect()
    }

    /// Determine if a URL should be crawled
    fn should_crawl(&self, url: &str, base_url: &Url) -> bool {
        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        // Check domain restrictions
        if self.config.stay_in_domain {
            if let (Some(base_domain), Some(url_domain)) = (base_url.domain(), parsed.domain()) {
                if base_domain != url_domain {
                    return false;
                }
            }
        }

        // Check subdomain restrictions
        if self.config.stay_in_subdomain {
            if let (Some(base_host), Some(url_host)) = (base_url.host_str(), parsed.host_str()) {
                if base_host != url_host {
                    return false;
                }
            }
        }

        // Check exclude patterns
        for pattern in &self.config.exclude_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(url) {
                    return false;
                }
            }
        }

        // Check include patterns (if specified)
        if !self.config.include_patterns.is_empty() {
            let mut matched = false;
            for pattern in &self.config.include_patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(url) {
                        matched = true;
                        break;
                    }
                }
            }
            if !matched {
                return false;
            }
        }

        true
    }

    /// Add links to the crawl queue
    fn enqueue_links(&self, parent_url: &str, links: &[String], depth: usize) {
        let mut queue = self.queue.lock().unwrap();
        for link in links {
            queue.push_back(CrawlItem {
                url: link.clone(),
                depth,
                parent_url: Some(parent_url.to_string()),
            });
        }
    }

    /// Update crawl tree with node info
    fn update_crawl_tree(&self, item: &CrawlItem, error: Option<String>) {
        let mut tree = self.crawl_tree.lock().unwrap();
        tree.push(CrawlNode {
            url: item.url.clone(),
            depth: item.depth,
            parent: item.parent_url.clone(),
            children: Vec::new(),
            scraped: error.is_none(),
            error,
        });
    }

    /// Get list of unique domains visited
    fn get_domains_visited(&self) -> Vec<String> {
        let visited = self.visited.lock().unwrap();
        let mut domains: HashSet<String> = HashSet::new();

        for url in visited.iter() {
            if let Ok(parsed) = Url::parse(url) {
                if let Some(domain) = parsed.domain() {
                    domains.insert(domain.to_string());
                }
            }
        }

        domains.into_iter().collect()
    }

    /// Determine final crawl status
    fn determine_status(&self, pages_crawled: usize) -> CrawlStatus {
        let errors = self.errors.lock().unwrap();

        if pages_crawled == 0 {
            CrawlStatus::Failed
        } else if !errors.is_empty() && pages_crawled < self.config.max_pages {
            CrawlStatus::PartiallyCompleted
        } else {
            CrawlStatus::Completed
        }
    }
}
