//! Update status file for post-update feedback.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Status of an update operation, written by the helper and read by the app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    /// Whether the update was successful.
    pub success: bool,
    /// Version that was installed.
    pub version: String,
    /// Previous version (for rollback info).
    pub previous_version: String,
    /// Timestamp of update.
    pub timestamp: DateTime<Utc>,
    /// Error message if failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Path to log file for debugging.
    pub log_file: PathBuf,
}

impl UpdateStatus {
    /// Creates a new successful status.
    pub fn success(version: String, previous_version: String, log_file: PathBuf) -> Self {
        Self {
            success: true,
            version,
            previous_version,
            timestamp: Utc::now(),
            error: None,
            log_file,
        }
    }

    /// Creates a new failed status.
    pub fn failure(
        version: String,
        previous_version: String,
        error: String,
        log_file: PathBuf,
    ) -> Self {
        Self {
            success: false,
            version,
            previous_version,
            timestamp: Utc::now(),
            error: Some(error),
            log_file,
        }
    }

    /// Gets the path to the status file.
    pub fn status_file_path() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("Trial Submission Studio").join("update_status.json"))
    }

    /// Writes the status to the status file.
    pub fn write(&self) -> Result<(), String> {
        let path = Self::status_file_path()
            .ok_or_else(|| "Could not determine status file path".to_string())?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create status directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize status: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write status file: {}", e))?;

        eprintln!("[helper] Status written to: {:?}", path);
        Ok(())
    }

    /// Reads the status from the status file.
    /// Used by the main app to read post-update status.
    #[allow(dead_code)]
    pub fn read() -> Option<Self> {
        let path = Self::status_file_path()?;

        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Deletes the status file.
    /// Used by the main app after reading and displaying the status.
    #[allow(dead_code)]
    pub fn delete() -> Result<(), String> {
        if let Some(path) = Self::status_file_path() {
            if path.exists() {
                fs::remove_file(&path)
                    .map_err(|e| format!("Failed to delete status file: {}", e))?;
            }
        }
        Ok(())
    }
}

/// Gets the path to the dirs crate's data directory.
mod dirs {
    use std::path::PathBuf;

    /// Returns the user's data directory.
    pub fn data_dir() -> Option<PathBuf> {
        // On macOS: ~/Library/Application Support
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join("Library/Application Support"))
    }
}
