//! Settings persistence - load and save settings to disk.
//!
//! Settings are stored in the platform-specific application data folder:
//! - macOS: ~/Library/Application Support/com.cdisc-transpiler.CDISC Transpiler/
//! - Windows: %APPDATA%/cdisc-transpiler/config/
//! - Linux: ~/.config/cdisc-transpiler/

use super::Settings;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

const APP_QUALIFIER: &str = "com";
const APP_ORG: &str = "cdisc-transpiler";
const APP_NAME: &str = "CDISC Transpiler";
const CONFIG_FILENAME: &str = "settings.toml";

/// Get the path to the settings file.
///
/// Returns `None` if the platform-specific directory cannot be determined.
pub fn settings_path() -> Option<PathBuf> {
    ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
        .map(|dirs| dirs.config_dir().join(CONFIG_FILENAME))
}

/// Load settings from disk.
///
/// Returns default settings if:
/// - The settings file doesn't exist
/// - The settings file cannot be parsed
/// - The platform-specific directory cannot be determined
pub fn load_settings() -> Settings {
    let Some(path) = settings_path() else {
        tracing::warn!("Could not determine settings path, using defaults");
        return Settings::default();
    };

    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(settings) => {
                tracing::info!("Loaded settings from {:?}", path);
                settings
            }
            Err(e) => {
                tracing::warn!("Failed to parse settings file: {}, using defaults", e);
                Settings::default()
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::info!("No settings file found at {:?}, using defaults", path);
            Settings::default()
        }
        Err(e) => {
            tracing::warn!("Failed to read settings file: {}, using defaults", e);
            Settings::default()
        }
    }
}

/// Save settings to disk.
///
/// Creates the parent directory if it doesn't exist.
pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let Some(path) = settings_path() else {
        return Err("Could not determine settings path".to_string());
    };

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    // Serialize to TOML
    let content = toml::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    // Write to file
    fs::write(&path, content).map_err(|e| format!("Failed to write settings file: {}", e))?;

    tracing::info!("Saved settings to {:?}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_path_exists() {
        // Should return Some on most platforms
        let path = settings_path();
        assert!(path.is_some());
    }

    #[test]
    fn test_default_settings_serializable() {
        let settings = Settings::default();
        let toml = toml::to_string_pretty(&settings);
        assert!(toml.is_ok());
    }

    #[test]
    fn test_settings_round_trip() {
        let settings = Settings::default();
        let toml_str = toml::to_string_pretty(&settings).unwrap();
        let parsed: Settings = toml::from_str(&toml_str).unwrap();

        assert_eq!(settings.general.dark_mode, parsed.general.dark_mode);
        assert_eq!(settings.general.ct_version, parsed.general.ct_version);
        assert_eq!(settings.validation.mode, parsed.validation.mode);
    }
}
