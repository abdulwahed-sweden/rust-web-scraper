use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Automatic selector detection with intelligent heuristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSelectors {
    pub title: Vec<String>,
    pub content: Vec<String>,
    pub links: Vec<String>,
    pub images: Vec<String>,
    pub metadata: Vec<String>,
}

impl Default for AutoSelectors {
    fn default() -> Self {
        Self {
            title: vec![
                "h1".to_string(),
                "h2".to_string(),
                "title".to_string(),
                "meta[property='og:title']".to_string(),
                ".title".to_string(),
                "#title".to_string(),
            ],
            content: vec![
                "article".to_string(),
                "main".to_string(),
                "p".to_string(),
                ".content".to_string(),
                ".article-body".to_string(),
                ".post-content".to_string(),
                "[role='main']".to_string(),
            ],
            links: vec![
                "a[href]".to_string(),
                "nav a".to_string(),
                ".nav-link".to_string(),
            ],
            images: vec![
                "img[src]".to_string(),
                "picture img".to_string(),
                "[data-src]".to_string(),
            ],
            metadata: vec![
                "meta[name='description']".to_string(),
                "meta[property='og:description']".to_string(),
                "meta[name='keywords']".to_string(),
                "meta[name='author']".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedContent {
    pub title: Option<String>,
    pub content: Vec<String>,
    pub links: Vec<LinkData>,
    pub images: Vec<ImageData>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkData {
    pub text: String,
    pub href: String,
    pub is_external: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
}

pub struct SelectorDetector {
    selectors: AutoSelectors,
}

impl SelectorDetector {
    pub fn new() -> Self {
        Self {
            selectors: AutoSelectors::default(),
        }
    }

    pub fn with_custom_selectors(selectors: AutoSelectors) -> Self {
        Self { selectors }
    }

    /// Detect and extract content from HTML using intelligent heuristics
    pub fn detect(&self, html: &str, base_url: &str) -> DetectedContent {
        let document = Html::parse_document(html);

        DetectedContent {
            title: self.detect_title(&document),
            content: self.detect_content(&document),
            links: self.detect_links(&document, base_url),
            images: self.detect_images(&document, base_url),
            metadata: self.detect_metadata(&document),
        }
    }

    fn detect_title(&self, document: &Html) -> Option<String> {
        for selector_str in &self.selectors.title {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let text = if selector_str.starts_with("meta") {
                        // For meta tags, get the content attribute
                        element.value().attr("content")
                            .map(|s| s.to_string())
                    } else {
                        // For regular elements, get text content
                        let text: String = element.text().collect();
                        if text.trim().is_empty() {
                            None
                        } else {
                            Some(text.trim().to_string())
                        }
                    };

                    if let Some(t) = text {
                        if !t.is_empty() {
                            return Some(t);
                        }
                    }
                }
            }
        }
        None
    }

    fn detect_content(&self, document: &Html) -> Vec<String> {
        let mut content = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for selector_str in &self.selectors.content {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let text: String = element.text().collect();
                    let trimmed = text.trim();

                    // Skip empty, duplicate, or too short content
                    if !trimmed.is_empty() && trimmed.len() > 10 && seen.insert(trimmed.to_string()) {
                        content.push(trimmed.to_string());
                    }
                }
            }
        }

        content
    }

    fn detect_links(&self, document: &Html, base_url: &str) -> Vec<LinkData> {
        let mut links = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for selector_str in &self.selectors.links {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if let Some(href) = element.value().attr("href") {
                        let text: String = element.text().collect();
                        let text = text.trim();

                        // Resolve relative URLs
                        let absolute_url = if href.starts_with("http://") || href.starts_with("https://") {
                            href.to_string()
                        } else if let Ok(base) = url::Url::parse(base_url) {
                            base.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string())
                        } else {
                            href.to_string()
                        };

                        // Check if external link
                        let is_external = if let (Ok(base), Ok(link)) = (url::Url::parse(base_url), url::Url::parse(&absolute_url)) {
                            base.host() != link.host()
                        } else {
                            false
                        };

                        // Avoid duplicates
                        if seen.insert(absolute_url.clone()) {
                            links.push(LinkData {
                                text: if text.is_empty() { href.to_string() } else { text.to_string() },
                                href: absolute_url,
                                is_external,
                            });
                        }
                    }
                }
            }
        }

        links
    }

    fn detect_images(&self, document: &Html, base_url: &str) -> Vec<ImageData> {
        let mut images = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for selector_str in &self.selectors.images {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    // Try both src and data-src attributes
                    let src = element.value().attr("src")
                        .or_else(|| element.value().attr("data-src"));

                    if let Some(src_value) = src {
                        // Resolve relative URLs
                        let absolute_url = if src_value.starts_with("http://") || src_value.starts_with("https://") {
                            src_value.to_string()
                        } else if src_value.starts_with("//") {
                            format!("https:{}", src_value)
                        } else if let Ok(base) = url::Url::parse(base_url) {
                            base.join(src_value).map(|u| u.to_string()).unwrap_or_else(|_| src_value.to_string())
                        } else {
                            src_value.to_string()
                        };

                        // Avoid duplicates
                        if seen.insert(absolute_url.clone()) {
                            images.push(ImageData {
                                src: absolute_url,
                                alt: element.value().attr("alt").map(|s| s.to_string()),
                                title: element.value().attr("title").map(|s| s.to_string()),
                            });
                        }
                    }
                }
            }
        }

        images
    }

    fn detect_metadata(&self, document: &Html) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        for selector_str in &self.selectors.metadata {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if let Some(content) = element.value().attr("content") {
                        // Extract the metadata name/property
                        let key = element.value().attr("name")
                            .or_else(|| element.value().attr("property"))
                            .unwrap_or("unknown")
                            .to_string();

                        metadata.insert(key, content.to_string());
                    }
                }
            }
        }

        metadata
    }
}

impl Default for SelectorDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_selector_detection() {
        let html = r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <title>Test Page</title>
                    <meta name="description" content="A test page">
                </head>
                <body>
                    <h1>Main Title</h1>
                    <article>
                        <p>This is the main content of the page.</p>
                        <p>Another paragraph with more details.</p>
                    </article>
                    <a href="https://example.com">External Link</a>
                    <img src="/image.jpg" alt="Test Image">
                </body>
            </html>
        "#;

        let detector = SelectorDetector::new();
        let result = detector.detect(html, "https://example.com");

        assert!(result.title.is_some());
        assert!(!result.content.is_empty());
        assert!(!result.links.is_empty());
        assert!(!result.images.is_empty());
    }
}
