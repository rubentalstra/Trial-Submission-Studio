//! Mapping Repository for persisting and reusing mapping configurations.
//!
//! This module provides a file-system based repository for storing and retrieving
//! column mapping configurations. Mappings can be reused across runs to ensure
//! consistent column-to-variable mappings.
//!
//! # Storage Format
//!
//! Mappings are stored as JSON files with the naming convention:
//! `{study_id}_{domain_code}.json`
//!
//! The repository supports:
//! - Saving individual domain mappings
//! - Loading mappings by study/domain
//! - Loading all mappings for a study
//! - Listing available mappings

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use sdtm_model::MappingConfig;

/// Repository for storing and retrieving mapping configurations.
///
/// The repository uses a directory-based storage where each mapping
/// is stored as a JSON file with study and domain identifiers.
#[derive(Debug, Clone)]
pub struct MappingRepository {
    /// Base directory for storing mapping files.
    base_dir: PathBuf,
}

/// Metadata about a stored mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingMetadata {
    /// Study ID this mapping belongs to.
    pub study_id: String,
    /// Domain code for this mapping.
    pub domain_code: String,
    /// File path where the mapping is stored.
    pub file_path: PathBuf,
    /// Number of column mappings in this config.
    pub mapping_count: usize,
    /// Number of unmapped columns.
    pub unmapped_count: usize,
}

/// Extended mapping config with additional repository metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMappingConfig {
    /// The core mapping configuration.
    #[serde(flatten)]
    pub config: MappingConfig,
    /// Optional timestamp of when this mapping was saved (ISO 8601).
    pub saved_at: Option<String>,
    /// Optional description or notes about this mapping.
    pub description: Option<String>,
    /// Version of the mapping format.
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl StoredMappingConfig {
    /// Create a new stored config from a mapping config.
    pub fn new(config: MappingConfig) -> Self {
        Self {
            config,
            saved_at: Some(chrono_timestamp()),
            description: None,
            version: default_version(),
        }
    }

    /// Add a description to this stored config.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Get current timestamp in ISO 8601 format.
fn chrono_timestamp() -> String {
    // Simple timestamp without external dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple ISO 8601-like format
    format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        1970 + secs / 31536000,
        (secs % 31536000) / 2592000 + 1,
        (secs % 2592000) / 86400 + 1,
        (secs % 86400) / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

impl MappingRepository {
    /// Create a new mapping repository at the given directory.
    ///
    /// The directory will be created if it doesn't exist.
    pub fn new(base_dir: impl Into<PathBuf>) -> Result<Self> {
        let base_dir = base_dir.into();
        fs::create_dir_all(&base_dir).with_context(|| {
            format!(
                "Failed to create mapping repository: {}",
                base_dir.display()
            )
        })?;
        Ok(Self { base_dir })
    }

    /// Get the base directory of this repository.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Save a mapping configuration to the repository.
    ///
    /// The mapping is stored with a filename based on study_id and domain_code.
    pub fn save(&self, config: &MappingConfig) -> Result<PathBuf> {
        let stored = StoredMappingConfig::new(config.clone());
        self.save_stored(&stored)
    }

    /// Save a stored mapping configuration (with metadata) to the repository.
    pub fn save_stored(&self, stored: &StoredMappingConfig) -> Result<PathBuf> {
        let filename = self.mapping_filename(&stored.config.study_id, &stored.config.domain_code);
        let path = self.base_dir.join(&filename);
        let json = serde_json::to_string_pretty(stored)
            .with_context(|| format!("Failed to serialize mapping for {}", filename))?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write mapping to {}", path.display()))?;
        Ok(path)
    }

    /// Load a mapping configuration for a specific study and domain.
    ///
    /// Returns `None` if no mapping exists.
    pub fn load(&self, study_id: &str, domain_code: &str) -> Result<Option<MappingConfig>> {
        let stored = self.load_stored(study_id, domain_code)?;
        Ok(stored.map(|s| s.config))
    }

    /// Load a stored mapping configuration (with metadata).
    pub fn load_stored(
        &self,
        study_id: &str,
        domain_code: &str,
    ) -> Result<Option<StoredMappingConfig>> {
        let filename = self.mapping_filename(study_id, domain_code);
        let path = self.base_dir.join(&filename);
        if !path.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read mapping from {}", path.display()))?;
        let stored: StoredMappingConfig = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse mapping from {}", path.display()))?;
        Ok(Some(stored))
    }

    /// Load all mappings for a specific study.
    pub fn load_study_mappings(&self, study_id: &str) -> Result<BTreeMap<String, MappingConfig>> {
        let mut mappings = BTreeMap::new();
        let prefix = format!("{}_", normalize_id(study_id));

        for entry in fs::read_dir(&self.base_dir)
            .with_context(|| format!("Failed to read repository: {}", self.base_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            if !filename.starts_with(&prefix) || !filename.ends_with(".json") {
                continue;
            }

            let contents = fs::read_to_string(&path)?;
            if let Ok(stored) = serde_json::from_str::<StoredMappingConfig>(&contents) {
                mappings.insert(stored.config.domain_code.to_uppercase(), stored.config);
            }
        }

        Ok(mappings)
    }

    /// List all available mappings in the repository.
    pub fn list(&self) -> Result<Vec<MappingMetadata>> {
        let mut metadata = Vec::new();

        for entry in fs::read_dir(&self.base_dir)
            .with_context(|| format!("Failed to read repository: {}", self.base_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            if !filename.ends_with(".json") {
                continue;
            }

            let contents = fs::read_to_string(&path)?;
            if let Ok(stored) = serde_json::from_str::<StoredMappingConfig>(&contents) {
                metadata.push(MappingMetadata {
                    study_id: stored.config.study_id.clone(),
                    domain_code: stored.config.domain_code.clone(),
                    file_path: path,
                    mapping_count: stored.config.mappings.len(),
                    unmapped_count: stored.config.unmapped_columns.len(),
                });
            }
        }

        metadata.sort_by(|a, b| {
            a.study_id
                .cmp(&b.study_id)
                .then_with(|| a.domain_code.cmp(&b.domain_code))
        });
        Ok(metadata)
    }

    /// Delete a mapping from the repository.
    pub fn delete(&self, study_id: &str, domain_code: &str) -> Result<bool> {
        let filename = self.mapping_filename(study_id, domain_code);
        let path = self.base_dir.join(&filename);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete mapping: {}", path.display()))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if a mapping exists.
    pub fn exists(&self, study_id: &str, domain_code: &str) -> bool {
        let filename = self.mapping_filename(study_id, domain_code);
        self.base_dir.join(&filename).exists()
    }

    /// Generate the filename for a mapping.
    fn mapping_filename(&self, study_id: &str, domain_code: &str) -> String {
        format!(
            "{}_{}.json",
            normalize_id(study_id),
            normalize_id(domain_code)
        )
    }
}

/// Normalize an ID for use in filenames.
fn normalize_id(id: &str) -> String {
    id.trim()
        .to_uppercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// Builder for loading or creating mapping configs with repository support.
pub struct MappingConfigLoader {
    repository: Option<MappingRepository>,
    study_id: String,
}

impl MappingConfigLoader {
    /// Create a new loader for a study.
    pub fn new(study_id: impl Into<String>) -> Self {
        Self {
            repository: None,
            study_id: study_id.into(),
        }
    }

    /// Set the repository to use for loading/saving mappings.
    pub fn with_repository(mut self, repository: MappingRepository) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Load a mapping for a domain, falling back to default if not found.
    pub fn load_or_default(
        &self,
        domain_code: &str,
        default_fn: impl FnOnce() -> MappingConfig,
    ) -> Result<MappingConfig> {
        if let Some(ref repo) = self.repository
            && let Some(config) = repo.load(&self.study_id, domain_code)?
        {
            return Ok(config);
        }
        Ok(default_fn())
    }

    /// Save a mapping to the repository if one is configured.
    pub fn save(&self, config: &MappingConfig) -> Result<Option<PathBuf>> {
        if let Some(ref repo) = self.repository {
            return Ok(Some(repo.save(config)?));
        }
        Ok(None)
    }
}
