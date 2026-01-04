//! Auto-update system for Trial Submission Studio.
//!
//! This crate provides functionality for checking for updates and installing them
//! using the `self_update` crate for all the heavy lifting.
//!
//! # Overview
//!
//! The update system uses GitHub Releases as its source. It supports:
//!
//! - Semantic versioning with pre-release tags (alpha, beta, rc)
//! - Configurable update channels (stable, beta)
//! - Automatic platform detection (macOS, Windows, Linux on x64 and ARM64)
//! - Seamless in-place updates via self_update
//!
//! # Supported Platforms
//!
//! - macOS: x86_64 (Intel) and aarch64 (Apple Silicon)
//! - Windows: x86_64 and aarch64
//! - Linux: x86_64 and aarch64
//!
//! # Example
//!
//! ```no_run
//! use tss_updater::{UpdateService, UpdateSettings};
//!
//! let settings = UpdateSettings::default();
//!
//! // Check for updates
//! if let Ok(Some(update)) = UpdateService::check_for_update(&settings) {
//!     println!("Update available: {}", update.version);
//!     println!("Changelog: {}", update.changelog);
//!
//!     // Download and install (app will restart automatically)
//!     UpdateService::download_and_install().unwrap();
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod error;
pub mod release;
pub mod version;

// Re-export main types for convenience
pub use config::{UpdateChannel, UpdateCheckFrequency, UpdateSettings};
pub use error::{Result, UpdateError};
pub use release::UpdateInfo;
pub use version::{PreRelease, Version};

use self_update::backends::github::Update;
use self_update::cargo_crate_version;

/// The current version of this crate (from Cargo.toml).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub repository owner.
pub const REPO_OWNER: &str = "rubentalstra";

/// GitHub repository name.
pub const REPO_NAME: &str = "Trial-Submission-Studio";

/// Binary name.
pub const BIN_NAME: &str = "trial-submission-studio";

/// Get the runtime target triple for asset matching.
///
/// This constructs the target triple at runtime using `std::env::consts`,
/// ensuring we always download the correct architecture even if the binary
/// was cross-compiled.
fn get_runtime_target() -> String {
    let arch = std::env::consts::ARCH; // "aarch64" or "x86_64"
    let os_suffix = match std::env::consts::OS {
        "macos" => "apple-darwin",
        "windows" => "pc-windows-msvc",
        "linux" => "unknown-linux-gnu",
        _ => "unknown",
    };
    format!("{}-{}", arch, os_suffix)
}

/// High-level update service that uses self_update for all operations.
///
/// The update service automatically detects the current platform's target triple
/// at runtime using `std::env::consts`, ensuring the correct architecture is
/// always downloaded regardless of how the binary was built.
pub struct UpdateService;

impl UpdateService {
    /// Check for available updates.
    ///
    /// Returns `Some(UpdateInfo)` if an update is available, `None` otherwise.
    /// Uses self_update's GitHub backend to fetch releases.
    pub fn check_for_update(settings: &UpdateSettings) -> Result<Option<UpdateInfo>> {
        let target = get_runtime_target();
        tracing::debug!("Using target for update check: {}", target);

        let update = Update::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .bin_name(BIN_NAME)
            .target(&target)
            .current_version(cargo_crate_version!())
            .build()
            .map_err(|e| UpdateError::SelfUpdate(e.to_string()))?;

        let latest = match update.get_latest_release() {
            Ok(release) => release,
            Err(e) => {
                tracing::debug!("Failed to get latest release: {}", e);
                return Err(UpdateError::Network(e.to_string()));
            }
        };

        // Parse version from tag (e.g., "v1.2.3" or "v1.2.3-beta.1")
        let latest_version = Version::from_tag(&latest.version)?;
        let current = Version::current();

        tracing::debug!(
            "Current version: {}, Latest version: {}",
            current,
            latest_version
        );

        // Check if update is newer
        if latest_version <= current {
            tracing::debug!("Already up to date");
            return Ok(None);
        }

        // Filter by channel (stable users shouldn't see beta releases)
        if !settings.channel.includes(&latest_version) {
            tracing::debug!(
                "Skipping {} - not allowed by channel {:?}",
                latest_version,
                settings.channel
            );
            return Ok(None);
        }

        // Check if user has skipped this version
        if settings.should_skip_version(&latest_version) {
            tracing::debug!("User has skipped version {}", latest_version);
            return Ok(None);
        }

        Ok(Some(UpdateInfo {
            version: latest.version,
            changelog: latest.body.unwrap_or_default(),
        }))
    }

    /// Download, verify, install, and restart the application.
    ///
    /// This function uses self_update to handle the entire update process:
    /// - Downloads the correct platform-specific asset (detected at runtime)
    /// - Verifies the download
    /// - Extracts and replaces the binary
    /// - The application should be restarted after this returns successfully
    ///
    /// Note: This function blocks during download. Progress is shown via self_update's
    /// built-in progress indicator.
    pub fn download_and_install() -> Result<()> {
        let target = get_runtime_target();
        tracing::info!(
            "Starting update download and installation for target: {}",
            target
        );

        let status = Update::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .bin_name(BIN_NAME)
            .target(&target)
            .show_download_progress(true)
            .current_version(cargo_crate_version!())
            .build()
            .map_err(|e| UpdateError::SelfUpdate(e.to_string()))?
            .update()
            .map_err(|e| UpdateError::SelfUpdate(e.to_string()))?;

        tracing::info!("Update installed successfully: {:?}", status);

        Ok(())
    }

    /// Restart the application after an update.
    ///
    /// This spawns the current executable as a new process and exits.
    /// Call this after `download_and_install` returns successfully.
    pub fn restart() -> Result<()> {
        let exe = std::env::current_exe().map_err(|e| UpdateError::Io(e.to_string()))?;

        tracing::info!("Restarting application: {:?}", exe);

        std::process::Command::new(&exe)
            .spawn()
            .map_err(|e| UpdateError::Io(e.to_string()))?;

        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_repo_constants() {
        assert_eq!(REPO_OWNER, "rubentalstra");
        assert_eq!(REPO_NAME, "Trial-Submission-Studio");
        assert_eq!(BIN_NAME, "trial-submission-studio");
    }

    #[test]
    fn test_runtime_target_detection() {
        let target = get_runtime_target();

        // Target should contain architecture
        assert!(
            target.contains("aarch64") || target.contains("x86_64"),
            "Target should contain architecture: {}",
            target
        );

        // Target should contain OS-specific suffix
        #[cfg(target_os = "macos")]
        assert!(
            target.ends_with("apple-darwin"),
            "macOS target should end with apple-darwin: {}",
            target
        );

        #[cfg(target_os = "windows")]
        assert!(
            target.ends_with("pc-windows-msvc"),
            "Windows target should end with pc-windows-msvc: {}",
            target
        );

        #[cfg(target_os = "linux")]
        assert!(
            target.ends_with("unknown-linux-gnu"),
            "Linux target should end with unknown-linux-gnu: {}",
            target
        );
    }
}
