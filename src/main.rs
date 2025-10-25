use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use futures::future::join_all;
use reqwest;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    urls: Vec<String>,
    selectors: SelectorConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SelectorConfig {
    title: String,
    content: String,
    links: Option<String>,
    images: Option<String>,
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

    // Determine URLs and config
    let (urls, selector_config) = if let Some(config_path) = args.config {
        let config = load_config(&config_path)?;
        (config.urls, config.selectors)
    } else if let Some(urls) = args.urls {
        (urls, SelectorConfig::default())
    } else {
        // Default example
        println!("No URLs or config provided. Using default example.");
        println!("Usage examples:");
        println!("  cargo run -- -u https://example.com -u https://rust-lang.org");
        println!("  cargo run -- --config config.toml --save");
        println!("\nRunning default example...\n");
        (vec!["https://example.com".to_string()], SelectorConfig::default())
    };

    println!("Scraping {} URL(s)...", urls.len());

    // Scrape all URLs concurrently
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

    Ok(())
}
