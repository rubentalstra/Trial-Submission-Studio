//! Configuration types for the update helper.

use serde::Deserialize;
use std::path::PathBuf;

/// Configuration passed from the main application via config file.
#[derive(Debug, Deserialize)]
pub struct HelperConfig {
    /// Path to the new .app bundle (in temp directory).
    pub new_app_path: PathBuf,
    /// Path to the current .app bundle to replace.
    pub current_app_path: PathBuf,
    /// PID of the parent process to wait for.
    pub parent_pid: u32,
    /// Version being installed.
    #[serde(default)]
    pub version: String,
    /// Previous version (for rollback info).
    #[serde(default)]
    pub previous_version: String,
}

impl HelperConfig {
    /// Reads the configuration from a JSON file.
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file '{}': {}", path, e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// Validates that the required paths exist.
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        if !self.new_app_path.exists() {
            return Err(format!("New app not found: {:?}", self.new_app_path));
        }
        if !self.current_app_path.exists() {
            return Err(format!(
                "Current app not found: {:?}",
                self.current_app_path
            ));
        }
        Ok(())
    }
}
