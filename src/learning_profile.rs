use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use crate::structure_analyzer::StructureAnalysis;

/// A learned profile for a domain or pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteProfile {
    pub id: String,
    pub domain: String,
    pub pattern: Option<String>,
    pub main_content_selector: Option<String>,
    pub title_selector: Option<String>,
    pub comments_selector: Option<String>,
    pub extraction_mode: String,
    pub confidence: f64,
    pub use_count: i32,
    pub success_rate: f64,
    pub created_at: String,
    pub last_used: String,
    pub notes: Option<String>,
}

/// Profile database manager
pub struct ProfileDatabase {
    conn: Connection,
}

impl ProfileDatabase {
    /// Create a new database connection and initialize schema
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open database connection")?;

        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to create in-memory database")?;

        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY,
                domain TEXT NOT NULL,
                pattern TEXT,
                main_content_selector TEXT,
                title_selector TEXT,
                comments_selector TEXT,
                extraction_mode TEXT NOT NULL,
                confidence REAL NOT NULL,
                use_count INTEGER DEFAULT 0,
                success_rate REAL DEFAULT 1.0,
                created_at TEXT NOT NULL,
                last_used TEXT NOT NULL,
                notes TEXT
            )",
            [],
        )?;

        // Create indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_domain ON profiles(domain)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_confidence ON profiles(confidence DESC)",
            [],
        )?;

        Ok(())
    }

    /// Save a new profile from structure analysis
    pub fn save_from_analysis(&self, analysis: &StructureAnalysis) -> Result<SiteProfile> {
        let domain = Self::extract_domain(&analysis.url)?;

        let profile = SiteProfile {
            id: Uuid::new_v4().to_string(),
            domain: domain.clone(),
            pattern: None,
            main_content_selector: analysis.recommendations.best_main_content.clone(),
            title_selector: analysis.recommendations.best_title.clone(),
            comments_selector: analysis.recommendations.best_comments.clone(),
            extraction_mode: format!("{:?}", analysis.recommendations.suggested_mode),
            confidence: self.calculate_confidence_from_analysis(analysis),
            use_count: 0,
            success_rate: 1.0,
            created_at: chrono::Local::now().to_rfc3339(),
            last_used: chrono::Local::now().to_rfc3339(),
            notes: None,
        };

        self.insert_profile(&profile)?;
        log::info!("Saved profile for domain: {}", domain);

        Ok(profile)
    }

    /// Save or update a profile
    pub fn insert_profile(&self, profile: &SiteProfile) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO profiles
            (id, domain, pattern, main_content_selector, title_selector,
             comments_selector, extraction_mode, confidence, use_count,
             success_rate, created_at, last_used, notes)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                profile.id,
                profile.domain,
                profile.pattern,
                profile.main_content_selector,
                profile.title_selector,
                profile.comments_selector,
                profile.extraction_mode,
                profile.confidence,
                profile.use_count,
                profile.success_rate,
                profile.created_at,
                profile.last_used,
                profile.notes,
            ],
        )?;

        Ok(())
    }

    /// Get profile by domain (most recent and confident)
    pub fn get_by_domain(&self, domain: &str) -> Result<Option<SiteProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, domain, pattern, main_content_selector, title_selector,
                    comments_selector, extraction_mode, confidence, use_count,
                    success_rate, created_at, last_used, notes
             FROM profiles
             WHERE domain = ?1
             ORDER BY confidence DESC, last_used DESC
             LIMIT 1"
        )?;

        let profile = stmt.query_row([domain], |row| {
            Ok(SiteProfile {
                id: row.get(0)?,
                domain: row.get(1)?,
                pattern: row.get(2)?,
                main_content_selector: row.get(3)?,
                title_selector: row.get(4)?,
                comments_selector: row.get(5)?,
                extraction_mode: row.get(6)?,
                confidence: row.get(7)?,
                use_count: row.get(8)?,
                success_rate: row.get(9)?,
                created_at: row.get(10)?,
                last_used: row.get(11)?,
                notes: row.get(12)?,
            })
        }).optional()?;

        Ok(profile)
    }

    /// Get all profiles, ordered by confidence
    pub fn get_all(&self) -> Result<Vec<SiteProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, domain, pattern, main_content_selector, title_selector,
                    comments_selector, extraction_mode, confidence, use_count,
                    success_rate, created_at, last_used, notes
             FROM profiles
             ORDER BY confidence DESC, last_used DESC"
        )?;

        let profiles = stmt.query_map([], |row| {
            Ok(SiteProfile {
                id: row.get(0)?,
                domain: row.get(1)?,
                pattern: row.get(2)?,
                main_content_selector: row.get(3)?,
                title_selector: row.get(4)?,
                comments_selector: row.get(5)?,
                extraction_mode: row.get(6)?,
                confidence: row.get(7)?,
                use_count: row.get(8)?,
                success_rate: row.get(9)?,
                created_at: row.get(10)?,
                last_used: row.get(11)?,
                notes: row.get(12)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(profiles)
    }

    /// Get profiles for a specific extraction mode
    pub fn get_by_mode(&self, mode: &str) -> Result<Vec<SiteProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, domain, pattern, main_content_selector, title_selector,
                    comments_selector, extraction_mode, confidence, use_count,
                    success_rate, created_at, last_used, notes
             FROM profiles
             WHERE extraction_mode = ?1
             ORDER BY confidence DESC"
        )?;

        let profiles = stmt.query_map([mode], |row| {
            Ok(SiteProfile {
                id: row.get(0)?,
                domain: row.get(1)?,
                pattern: row.get(2)?,
                main_content_selector: row.get(3)?,
                title_selector: row.get(4)?,
                comments_selector: row.get(5)?,
                extraction_mode: row.get(6)?,
                confidence: row.get(7)?,
                use_count: row.get(8)?,
                success_rate: row.get(9)?,
                created_at: row.get(10)?,
                last_used: row.get(11)?,
                notes: row.get(12)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(profiles)
    }

    /// Update profile usage statistics
    pub fn update_usage(&self, profile_id: &str, success: bool) -> Result<()> {
        let profile = self.get_by_id(profile_id)?;

        if let Some(mut p) = profile {
            p.use_count += 1;

            // Update success rate with exponential moving average
            let alpha = 0.3; // Weight for new observation
            let new_success = if success { 1.0 } else { 0.0 };
            p.success_rate = alpha * new_success + (1.0 - alpha) * p.success_rate;

            p.last_used = chrono::Local::now().to_rfc3339();

            self.insert_profile(&p)?;
            log::info!("Updated usage for profile: {} (success: {})", profile_id, success);
        }

        Ok(())
    }

    /// Get profile by ID
    pub fn get_by_id(&self, id: &str) -> Result<Option<SiteProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, domain, pattern, main_content_selector, title_selector,
                    comments_selector, extraction_mode, confidence, use_count,
                    success_rate, created_at, last_used, notes
             FROM profiles
             WHERE id = ?1"
        )?;

        let profile = stmt.query_row([id], |row| {
            Ok(SiteProfile {
                id: row.get(0)?,
                domain: row.get(1)?,
                pattern: row.get(2)?,
                main_content_selector: row.get(3)?,
                title_selector: row.get(4)?,
                comments_selector: row.get(5)?,
                extraction_mode: row.get(6)?,
                confidence: row.get(7)?,
                use_count: row.get(8)?,
                success_rate: row.get(9)?,
                created_at: row.get(10)?,
                last_used: row.get(11)?,
                notes: row.get(12)?,
            })
        }).optional()?;

        Ok(profile)
    }

    /// Delete profile by ID
    pub fn delete(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM profiles WHERE id = ?1", [id])?;
        log::info!("Deleted profile: {}", id);
        Ok(())
    }

    /// Clear all profiles
    pub fn clear_all(&self) -> Result<()> {
        self.conn.execute("DELETE FROM profiles", [])?;
        log::info!("Cleared all profiles");
        Ok(())
    }

    /// Get statistics
    pub fn get_stats(&self) -> Result<ProfileStats> {
        let total_profiles: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM profiles",
            [],
            |row| row.get(0)
        )?;

        let total_uses: i32 = self.conn.query_row(
            "SELECT COALESCE(SUM(use_count), 0) FROM profiles",
            [],
            |row| row.get(0)
        )?;

        let avg_confidence: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(confidence), 0.0) FROM profiles",
            [],
            |row| row.get(0)
        )?;

        let avg_success_rate: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(success_rate), 0.0) FROM profiles",
            [],
            |row| row.get(0)
        )?;

        Ok(ProfileStats {
            total_profiles,
            total_uses,
            avg_confidence,
            avg_success_rate,
        })
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> Result<String> {
        let parsed = url::Url::parse(url)
            .context("Failed to parse URL")?;

        parsed.host_str()
            .map(|h| h.to_string())
            .context("No host in URL")
    }

    /// Calculate confidence from analysis
    fn calculate_confidence_from_analysis(&self, analysis: &StructureAnalysis) -> f64 {
        if analysis.sections.is_empty() {
            return 0.0;
        }

        // Get top section score
        let top_score = analysis.sections.first()
            .map(|s| s.score)
            .unwrap_or(0.0);

        // Factor in recommendations
        let has_main_content = analysis.recommendations.best_main_content.is_some();
        let has_title = analysis.recommendations.best_title.is_some();

        let mut confidence = top_score * 0.7;

        if has_main_content {
            confidence += 0.2;
        }

        if has_title {
            confidence += 0.1;
        }

        confidence.min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStats {
    pub total_profiles: i32,
    pub total_uses: i32,
    pub avg_confidence: f64,
    pub avg_success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = ProfileDatabase::new_in_memory().unwrap();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_profiles, 0);
    }

    #[test]
    fn test_profile_crud() {
        let db = ProfileDatabase::new_in_memory().unwrap();

        let profile = SiteProfile {
            id: Uuid::new_v4().to_string(),
            domain: "example.com".to_string(),
            pattern: None,
            main_content_selector: Some("article".to_string()),
            title_selector: Some("h1".to_string()),
            comments_selector: None,
            extraction_mode: "Article".to_string(),
            confidence: 0.9,
            use_count: 0,
            success_rate: 1.0,
            created_at: chrono::Local::now().to_rfc3339(),
            last_used: chrono::Local::now().to_rfc3339(),
            notes: None,
        };

        db.insert_profile(&profile).unwrap();

        let retrieved = db.get_by_domain("example.com").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().domain, "example.com");

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_profiles, 1);
    }
}
