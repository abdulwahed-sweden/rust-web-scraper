use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use futures::future::join_all;
use reqwest;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use url::Url;

#[derive(Parser, Debug)]
#[command(name = "rust-web-scraper")]
#[command(about = "A powerful web scraper with concurrent support", long_about = None)]
struct Args {
    /// URLs to scrape (can provide multiple)
    #[arg(short, long, num_args = 1..)]
    urls: Option<Vec<String>>,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Output directory for scraped data
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Save output to files
    #[arg(short, long)]
    save: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Enable pagination (follow "Next" links automatically)
    #[arg(short, long)]
    paginate: bool,

    /// Maximum number of pages to scrape per URL (0 = unlimited)
    #[arg(short = 'm', long, default_value = "0")]
    max_pages: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    urls: Vec<String>,
    selectors: SelectorConfig,
    pagination: Option<PaginationConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SelectorConfig {
    title: String,
    content: String,
    links: Option<String>,
    images: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PaginationConfig {
    enabled: bool,
    next_selector: String,
    max_pages: Option<usize>,
}

impl Default for SelectorConfig {
    fn default() -> Self {
        Self {
            title: "h1".to_string(),
            content: "p".to_string(),
            links: Some("a".to_string()),
            images: Some("img".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ScrapedData {
    url: String,
    timestamp: String,
    title: String,
    content: Vec<String>,
    links: Vec<LinkData>,
    images: Vec<ImageData>,
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct LinkData {
    text: String,
    href: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageData {
    alt: String,
    src: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    content_count: usize,
    links_count: usize,
    images_count: usize,
}

async fn scrape_url(url: &str, config: &SelectorConfig, verbose: bool) -> Result<ScrapedData> {
    if verbose {
        println!("Fetching content from: {}", url);
    }

    // Fetch the HTML content
    let response = reqwest::get(url)
        .await
        .context(format!("Failed to fetch URL: {}", url))?;

    let body = response
        .text()
        .await
        .context("Failed to read response body")?;

    // Parse the HTML
    let document = Html::parse_document(&body);

    // Extract title
    let title_selector = Selector::parse(&config.title)
        .map_err(|e| anyhow::anyhow!("Invalid title selector: {:?}", e))?;

    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| "No title found".to_string())
        .trim()
        .to_string();

    // Extract content
    let content_selector = Selector::parse(&config.content)
        .map_err(|e| anyhow::anyhow!("Invalid content selector: {:?}", e))?;

    let content: Vec<String> = document
        .select(&content_selector)
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Extract links
    let links = if let Some(link_selector_str) = &config.links {
        let link_selector = Selector::parse(link_selector_str)
            .map_err(|e| anyhow::anyhow!("Invalid link selector: {:?}", e))?;

        document
            .select(&link_selector)
            .filter_map(|el| {
                let href = el.value().attr("href")?;
                let text = el.text().collect::<String>().trim().to_string();
                Some(LinkData {
                    text,
                    href: href.to_string(),
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    // Extract images
    let images = if let Some(img_selector_str) = &config.images {
        let img_selector = Selector::parse(img_selector_str)
            .map_err(|e| anyhow::anyhow!("Invalid image selector: {:?}", e))?;

        document
            .select(&img_selector)
            .filter_map(|el| {
                let src = el.value().attr("src")?;
                let alt = el.value().attr("alt").unwrap_or("").to_string();
                Some(ImageData {
                    alt,
                    src: src.to_string(),
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    let metadata = Metadata {
        content_count: content.len(),
        links_count: links.len(),
        images_count: images.len(),
    };

    Ok(ScrapedData {
        url: url.to_string(),
        timestamp: Local::now().to_rfc3339(),
        title,
        content,
        links,
        images,
        metadata,
    })
}

fn extract_next_page_url(
    document: &Html,
    current_url: &str,
    next_selector: &str,
) -> Result<Option<String>> {
    let selector = Selector::parse(next_selector)
        .map_err(|e| anyhow::anyhow!("Invalid next page selector: {:?}", e))?;

    if let Some(next_link) = document.select(&selector).next() {
        if let Some(href) = next_link.value().attr("href") {
            // Convert relative URL to absolute
            let base_url = Url::parse(current_url)
                .context("Failed to parse current URL")?;
            let next_url = base_url
                .join(href)
                .context("Failed to join next page URL")?;
            return Ok(Some(next_url.to_string()));
        }
    }

    Ok(None)
}

async fn scrape_with_pagination(
    start_url: &str,
    config: &SelectorConfig,
    pagination_config: Option<&PaginationConfig>,
    max_pages: usize,
    verbose: bool,
    args: &Args,
) -> Result<Vec<ScrapedData>> {
    let mut all_data = Vec::new();
    let mut visited_urls = HashSet::new();
    let mut current_url = start_url.to_string();
    let mut page_count = 0;

    let effective_max_pages = if max_pages > 0 {
        max_pages
    } else if let Some(pagination) = pagination_config {
        pagination.max_pages.unwrap_or(usize::MAX)
    } else {
        usize::MAX
    };

    loop {
        // Check if we've already visited this URL
        if visited_urls.contains(&current_url) {
            if verbose {
                println!("  Skipping already visited URL: {}", current_url);
            }
            break;
        }

        // Check if we've reached the max page limit
        if page_count >= effective_max_pages {
            if verbose {
                println!("  Reached maximum page limit ({} pages)", effective_max_pages);
            }
            break;
        }

        visited_urls.insert(current_url.clone());
        page_count += 1;

        if verbose {
            println!("  Page {}/{}: {}",
                page_count,
                if effective_max_pages == usize::MAX {
                    "∞".to_string()
                } else {
                    effective_max_pages.to_string()
                },
                current_url
            );
        }

        // Scrape the current page
        match scrape_url(&current_url, config, false).await {
            Ok(data) => {
                // Save immediately if requested
                if args.save {
                    match save_to_file(&data, &args.output) {
                        Ok(filepath) => {
                            if verbose {
                                println!("    ✓ Saved to: {:?}", filepath);
                            }
                        }
                        Err(e) => {
                            eprintln!("    ✗ Failed to save: {}", e);
                        }
                    }
                }
                all_data.push(data);
            }
            Err(e) => {
                eprintln!("  ✗ Error scraping {}: {}", current_url, e);
                break;
            }
        }

        // Check if pagination is enabled
        let should_paginate = if let Some(pagination) = pagination_config {
            pagination.enabled
        } else {
            false
        };

        if !should_paginate {
            break;
        }

        // Try to find the next page URL
        let response = match reqwest::get(&current_url).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("  ✗ Failed to fetch page for pagination: {}", e);
                break;
            }
        };

        let body = match response.text().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("  ✗ Failed to read response body: {}", e);
                break;
            }
        };

        let document = Html::parse_document(&body);

        let next_selector = if let Some(pagination) = pagination_config {
            &pagination.next_selector
        } else {
            break;
        };

        match extract_next_page_url(&document, &current_url, next_selector) {
            Ok(Some(next_url)) => {
                if verbose {
                    println!("    → Next page found: {}", next_url);
                }
                current_url = next_url;
            }
            Ok(None) => {
                if verbose {
                    println!("    No more pages found");
                }
                break;
            }
            Err(e) => {
                eprintln!("    ✗ Error extracting next page URL: {}", e);
                break;
            }
        }
    }

    Ok(all_data)
}

fn load_config(path: &PathBuf) -> Result<Config> {
    let config_str = fs::read_to_string(path)
        .context(format!("Failed to read config file: {:?}", path))?;

    let config: Config = toml::from_str(&config_str)
        .context("Failed to parse config file")?;

    Ok(config)
}

fn save_to_file(data: &ScrapedData, output_dir: &PathBuf) -> Result<PathBuf> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)
        .context(format!("Failed to create output directory: {:?}", output_dir))?;

    // Generate filename from URL and timestamp
    let filename = format!(
        "scraped_{}_{}.json",
        data.url
            .replace("https://", "")
            .replace("http://", "")
            .replace("/", "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>(),
        Local::now().format("%Y%m%d_%H%M%S")
    );

    let filepath = output_dir.join(filename);
    let json = serde_json::to_string_pretty(data)
        .context("Failed to serialize data to JSON")?;

    fs::write(&filepath, json)
        .context(format!("Failed to write to file: {:?}", filepath))?;

    Ok(filepath)
}

fn display_results(data: &ScrapedData) {
    println!("\n{}", "=".repeat(60));
    println!("URL: {}", data.url);
    println!("Timestamp: {}", data.timestamp);
    println!("{}", "=".repeat(60));
    println!("\nTitle: {}", data.title);

    println!("\nContent ({} items):", data.content.len());
    for (i, paragraph) in data.content.iter().enumerate().take(5) {
        println!("  {}. {}", i + 1, paragraph);
    }
    if data.content.len() > 5 {
        println!("  ... and {} more", data.content.len() - 5);
    }

    println!("\nLinks ({} items):", data.links.len());
    for (i, link) in data.links.iter().enumerate().take(5) {
        println!("  {}. {} -> {}", i + 1, link.text, link.href);
    }
    if data.links.len() > 5 {
        println!("  ... and {} more", data.links.len() - 5);
    }

    println!("\nImages ({} items):", data.images.len());
    for (i, img) in data.images.iter().enumerate().take(5) {
        println!("  {}. [{}] {}", i + 1, img.alt, img.src);
    }
    if data.images.len() > 5 {
        println!("  ... and {} more", data.images.len() - 5);
    }

    println!("\nMetadata:");
    println!("  Content items: {}", data.metadata.content_count);
    println!("  Links: {}", data.metadata.links_count);
    println!("  Images: {}", data.metadata.images_count);
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Determine URLs, config, and pagination settings
    let (urls, selector_config, pagination_config) = if let Some(ref config_path) = args.config {
        let config = load_config(config_path)?;
        (config.urls, config.selectors, config.pagination)
    } else if let Some(ref urls) = args.urls {
        (urls.clone(), SelectorConfig::default(), None)
    } else {
        // Default example
        println!("No URLs or config provided. Using default example.");
        println!("Usage examples:");
        println!("  cargo run -- -u https://example.com -u https://rust-lang.org");
        println!("  cargo run -- --config config.toml --save");
        println!("  cargo run -- -u https://books.toscrape.com --paginate -m 5");
        println!("\nRunning default example...\n");
        (vec!["https://example.com".to_string()], SelectorConfig::default(), None)
    };

    // Check if pagination is enabled via CLI or config
    let use_pagination = args.paginate || pagination_config.as_ref().map_or(false, |p| p.enabled);

    if use_pagination {
        println!("Scraping {} URL(s) with pagination enabled...", urls.len());

        // Process each URL with pagination
        let mut total_success = 0;
        let mut total_error = 0;

        for url in &urls {
            println!("\n{}", "=".repeat(60));
            println!("Starting pagination from: {}", url);
            println!("{}", "=".repeat(60));

            match scrape_with_pagination(
                url,
                &selector_config,
                pagination_config.as_ref(),
                args.max_pages,
                args.verbose,
                &args,
            ).await {
                Ok(pages) => {
                    // Display results for each page
                    for (i, data) in pages.iter().enumerate() {
                        if !args.save {  // Only display if not saving (saves already displayed)
                            println!("\n--- Page {} of {} ---", i + 1, pages.len());
                            display_results(data);
                        }
                    }

                    total_success += pages.len();
                    println!("\n✓ Successfully scraped {} pages from {}", pages.len(), url);
                }
                Err(e) => {
                    eprintln!("\n✗ Error during pagination for {}: {}", url, e);
                    total_error += 1;
                }
            }
        }

        println!("\n{}", "=".repeat(60));
        println!("Summary: {} pages scraped, {} URLs failed", total_success, total_error);
        println!("{}", "=".repeat(60));
    } else {
        println!("Scraping {} URL(s)...", urls.len());

        // Scrape all URLs concurrently (original behavior)
        let scrape_tasks: Vec<_> = urls
            .iter()
            .map(|url| scrape_url(url, &selector_config, args.verbose))
            .collect();

        let results = join_all(scrape_tasks).await;

        // Process results
        let mut success_count = 0;
        let mut error_count = 0;

        for result in results {
            match result {
                Ok(data) => {
                    display_results(&data);

                    if args.save {
                        match save_to_file(&data, &args.output) {
                            Ok(filepath) => {
                                println!("\n✓ Data saved to: {:?}", filepath);
                            }
                            Err(e) => {
                                eprintln!("\n✗ Failed to save data: {}", e);
                            }
                        }
                    }

                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("\n✗ Error scraping URL: {}", e);
                    error_count += 1;
                }
            }
        }

        println!("\n{}", "=".repeat(60));
        println!("Summary: {} successful, {} failed", success_count, error_count);
        println!("{}", "=".repeat(60));
    }

    Ok(())
}
