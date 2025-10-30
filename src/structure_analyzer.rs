use scraper::{Html, Selector, ElementRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Structural analysis of HTML pages with intelligent scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureAnalysis {
    pub url: String,
    pub timestamp: String,
    pub sections: Vec<Section>,
    pub recommendations: Recommendations,
    pub debug_info: Option<DebugInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub selector: String,
    pub section_type: SectionType,
    pub score: f64,
    pub confidence: f64,
    pub stats: SectionStats,
    pub preview: String,
    pub xpath: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionType {
    MainContent,
    Article,
    Sidebar,
    Navigation,
    Header,
    Footer,
    Comments,
    RelatedLinks,
    Advertisements,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionStats {
    pub text_length: usize,
    pub word_count: usize,
    pub link_count: usize,
    pub image_count: usize,
    pub paragraph_count: usize,
    pub heading_count: usize,
    pub density_score: f64,
    pub link_density: f64,
    pub element_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendations {
    pub best_main_content: Option<String>,
    pub best_title: Option<String>,
    pub best_comments: Option<String>,
    pub suggested_mode: ExtractionMode,
    pub confidence_level: ConfidenceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMode {
    Article,
    Product,
    Forum,
    ListPage,
    Documentation,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    VeryHigh,
    High,
    Medium,
    Low,
    VeryLow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    pub total_elements: usize,
    pub analyzed_sections: usize,
    pub processing_time_ms: u64,
    pub scoring_details: Vec<ScoringDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringDetail {
    pub selector: String,
    pub raw_score: f64,
    pub adjustments: HashMap<String, f64>,
    pub final_score: f64,
}

pub struct StructureAnalyzer {
    min_content_length: usize,
    min_word_count: usize,
    detect_comments: bool,
    detect_metadata: bool,
    debug_mode: bool,
}

impl Default for StructureAnalyzer {
    fn default() -> Self {
        Self {
            min_content_length: 200,
            min_word_count: 50,
            detect_comments: true,
            detect_metadata: true,
            debug_mode: false,
        }
    }
}

impl StructureAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_options(
        min_content_length: usize,
        detect_comments: bool,
        debug_mode: bool,
    ) -> Self {
        Self {
            min_content_length,
            min_word_count: min_content_length / 5,
            detect_comments,
            detect_metadata: true,
            debug_mode,
        }
    }

    /// Analyze HTML structure and return scored sections
    pub fn analyze(&self, html: &str, url: &str) -> StructureAnalysis {
        let start_time = std::time::Instant::now();
        let document = Html::parse_document(html);

        // Find all potential content sections
        let sections = self.find_sections(&document);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&sections);

        // Build debug info if enabled
        let debug_info = if self.debug_mode {
            Some(DebugInfo {
                total_elements: self.count_elements(&document),
                analyzed_sections: sections.len(),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                scoring_details: vec![],
            })
        } else {
            None
        };

        StructureAnalysis {
            url: url.to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
            sections,
            recommendations,
            debug_info,
        }
    }

    fn find_sections(&self, document: &Html) -> Vec<Section> {
        let mut sections = Vec::new();

        // Common structural selectors to analyze
        let structural_selectors = vec![
            // Main content
            ("article", SectionType::Article),
            ("main", SectionType::MainContent),
            ("[role='main']", SectionType::MainContent),
            (".content", SectionType::MainContent),
            (".main-content", SectionType::MainContent),
            (".post-content", SectionType::Article),
            (".article-body", SectionType::Article),
            // Sidebar
            ("aside", SectionType::Sidebar),
            (".sidebar", SectionType::Sidebar),
            (".widget", SectionType::Sidebar),
            // Navigation
            ("nav", SectionType::Navigation),
            (".navigation", SectionType::Navigation),
            (".menu", SectionType::Navigation),
            // Header/Footer
            ("header", SectionType::Header),
            ("footer", SectionType::Footer),
            // Comments
            (".comments", SectionType::Comments),
            ("#comments", SectionType::Comments),
            (".comment-list", SectionType::Comments),
        ];

        for (selector_str, section_type) in structural_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if let Some(section) = self.analyze_element(element, selector_str, section_type.clone()) {
                        // Only include sections with meaningful content
                        if section.stats.text_length >= self.min_content_length
                            || matches!(section.section_type, SectionType::Header | SectionType::Footer | SectionType::Navigation) {
                            sections.push(section);
                        }
                    }
                }
            }
        }

        // If no main content found, analyze divs
        if !sections.iter().any(|s| matches!(s.section_type, SectionType::MainContent | SectionType::Article)) {
            sections.extend(self.analyze_divs(document));
        }

        // Sort by score (highest first)
        sections.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Remove duplicates (nested elements)
        self.deduplicate_sections(sections)
    }

    fn analyze_element(&self, element: ElementRef, selector: &str, mut section_type: SectionType) -> Option<Section> {
        let text: String = element.text().collect();
        let text = text.trim();

        if text.is_empty() {
            return None;
        }

        let stats = self.calculate_stats(element);

        // Adjust section type based on stats
        if stats.text_length > 500 && stats.density_score > 0.7 {
            if matches!(section_type, SectionType::MainContent) {
                section_type = SectionType::Article;
            }
        }

        // Calculate score
        let score = self.calculate_score(&stats, &section_type);
        let confidence = self.calculate_confidence(&stats, &section_type);

        // Generate preview (first 200 chars, respecting UTF-8 boundaries)
        let preview = if text.chars().count() > 200 {
            let truncated: String = text.chars().take(200).collect();
            format!("{}...", truncated)
        } else {
            text.to_string()
        };

        Some(Section {
            selector: selector.to_string(),
            section_type,
            score,
            confidence,
            stats,
            preview,
            xpath: None, // Could be computed if needed
        })
    }

    fn analyze_divs(&self, document: &Html) -> Vec<Section> {
        let mut sections = Vec::new();

        if let Ok(selector) = Selector::parse("div") {
            for element in document.select(&selector) {
                let stats = self.calculate_stats(element);

                // Only consider divs with substantial content
                if stats.text_length >= self.min_content_length * 2
                    && stats.density_score > 0.6
                    && stats.paragraph_count > 2 {

                    let score = self.calculate_score(&stats, &SectionType::MainContent);

                    if score > 0.5 {
                        let text: String = element.text().collect();
                        let preview = if text.len() > 200 {
                            format!("{}...", &text[..200].trim())
                        } else {
                            text.trim().to_string()
                        };

                        // Try to generate a unique selector
                        let selector_str = self.generate_selector(element);

                        sections.push(Section {
                            selector: selector_str,
                            section_type: SectionType::MainContent,
                            score,
                            confidence: self.calculate_confidence(&stats, &SectionType::MainContent),
                            stats,
                            preview,
                            xpath: None,
                        });
                    }
                }
            }
        }

        sections
    }

    fn calculate_stats(&self, element: ElementRef) -> SectionStats {
        let text: String = element.text().collect();
        let text_length = text.trim().len();
        let word_count = text.split_whitespace().count();

        // Count elements
        let link_count = self.count_descendants(element, "a");
        let image_count = self.count_descendants(element, "img");
        let paragraph_count = self.count_descendants(element, "p");
        let heading_count = self.count_descendants(element, "h1")
            + self.count_descendants(element, "h2")
            + self.count_descendants(element, "h3");

        let element_count = self.count_all_descendants(element);

        // Calculate density score (text per element ratio)
        let density_score = if element_count > 0 {
            (text_length as f64 / element_count as f64).min(1.0)
        } else {
            0.0
        };

        // Calculate link density (inverse - fewer links = better for main content)
        let link_density = if text_length > 0 {
            (link_count as f64 * 50.0) / text_length as f64
        } else {
            1.0
        };

        SectionStats {
            text_length,
            word_count,
            link_count,
            image_count,
            paragraph_count,
            heading_count,
            density_score,
            link_density,
            element_count,
        }
    }

    fn calculate_score(&self, stats: &SectionStats, section_type: &SectionType) -> f64 {
        let mut score = 0.0;

        match section_type {
            SectionType::Article | SectionType::MainContent => {
                // Favor high text density
                score += stats.density_score * 0.3;

                // Favor low link density
                score += (1.0 - stats.link_density.min(1.0)) * 0.3;

                // Favor multiple paragraphs
                score += (stats.paragraph_count.min(10) as f64 / 10.0) * 0.2;

                // Favor longer content
                score += (stats.text_length.min(5000) as f64 / 5000.0) * 0.2;
            }
            SectionType::Sidebar => {
                // Favor high link count
                score += (stats.link_count.min(20) as f64 / 20.0) * 0.5;

                // Penalize very long text
                score += (1.0 - (stats.text_length.min(2000) as f64 / 2000.0)) * 0.3;
            }
            SectionType::Navigation | SectionType::Header | SectionType::Footer => {
                // Favor high link density
                score += stats.link_density.min(1.0) * 0.5;

                // Favor short text
                score += (1.0 - (stats.text_length.min(500) as f64 / 500.0)) * 0.3;
            }
            SectionType::Comments => {
                // Favor multiple text blocks
                score += (stats.element_count.min(50) as f64 / 50.0) * 0.4;

                // Moderate text length
                score += ((stats.text_length as f64 - 500.0).abs() / 2000.0).min(1.0) * 0.3;
            }
            _ => {
                score = 0.5;
            }
        }

        score.clamp(0.0, 1.0)
    }

    fn calculate_confidence(&self, stats: &SectionStats, section_type: &SectionType) -> f64 {
        let mut confidence = 0.5;

        // Higher word count = higher confidence
        confidence += (stats.word_count.min(500) as f64 / 500.0) * 0.2;

        // More paragraphs = higher confidence for articles
        if matches!(section_type, SectionType::Article | SectionType::MainContent) {
            confidence += (stats.paragraph_count.min(10) as f64 / 10.0) * 0.2;
        }

        // Balanced link density = higher confidence
        if stats.link_density > 0.1 && stats.link_density < 0.3 {
            confidence += 0.1;
        }

        confidence.clamp(0.0, 1.0)
    }

    fn generate_recommendations(&self, sections: &[Section]) -> Recommendations {
        let best_main_content = sections
            .iter()
            .find(|s| matches!(s.section_type, SectionType::Article | SectionType::MainContent))
            .map(|s| s.selector.clone());

        let best_title = Some("h1, h2, title".to_string());

        let best_comments = sections
            .iter()
            .find(|s| matches!(s.section_type, SectionType::Comments))
            .map(|s| s.selector.clone());

        // Determine extraction mode based on content
        let suggested_mode = if sections.iter().any(|s| s.selector.contains("product")) {
            ExtractionMode::Product
        } else if sections.iter().any(|s| matches!(s.section_type, SectionType::Article)) {
            ExtractionMode::Article
        } else if sections.iter().any(|s| matches!(s.section_type, SectionType::Comments)) {
            ExtractionMode::Forum
        } else {
            ExtractionMode::Generic
        };

        // Determine confidence level
        let confidence_level = if let Some(main) = sections.first() {
            if main.score > 0.8 {
                ConfidenceLevel::VeryHigh
            } else if main.score > 0.6 {
                ConfidenceLevel::High
            } else if main.score > 0.4 {
                ConfidenceLevel::Medium
            } else {
                ConfidenceLevel::Low
            }
        } else {
            ConfidenceLevel::VeryLow
        };

        Recommendations {
            best_main_content,
            best_title,
            best_comments,
            suggested_mode,
            confidence_level,
        }
    }

    fn deduplicate_sections(&self, mut sections: Vec<Section>) -> Vec<Section> {
        let mut result = Vec::new();
        let mut seen_previews = std::collections::HashSet::new();

        for section in sections.drain(..) {
            // Use first 100 chars as fingerprint
            let fingerprint = if section.preview.len() > 100 {
                &section.preview[..100]
            } else {
                &section.preview
            };

            if seen_previews.insert(fingerprint.to_string()) {
                result.push(section);
            }
        }

        result
    }

    fn count_descendants(&self, element: ElementRef, tag: &str) -> usize {
        if let Ok(selector) = Selector::parse(tag) {
            element.select(&selector).count()
        } else {
            0
        }
    }

    fn count_all_descendants(&self, element: ElementRef) -> usize {
        element.descendants().count()
    }

    fn count_elements(&self, document: &Html) -> usize {
        document.root_element().descendants().count()
    }

    fn generate_selector(&self, element: ElementRef) -> String {
        // Try to generate a meaningful selector
        if let Some(id) = element.value().id() {
            return format!("#{}", id);
        }

        if let Some(classes) = element.value().classes().next() {
            return format!(".{}", classes);
        }

        element.value().name().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structure_analyzer() {
        let html = r#"
            <html>
                <body>
                    <header>
                        <nav><a href="/">Home</a></nav>
                    </header>
                    <article>
                        <h1>Main Article</h1>
                        <p>This is a long article with substantial content that should be detected as the main content area.</p>
                        <p>It has multiple paragraphs to increase the confidence score.</p>
                        <p>More content here to meet the minimum length requirements.</p>
                    </article>
                    <aside>
                        <h3>Related Links</h3>
                        <a href="/link1">Link 1</a>
                        <a href="/link2">Link 2</a>
                    </aside>
                </body>
            </html>
        "#;

        let analyzer = StructureAnalyzer::new();
        let analysis = analyzer.analyze(html, "https://example.com");

        assert!(!analysis.sections.is_empty());
        assert!(analysis.recommendations.best_main_content.is_some());
    }
}
