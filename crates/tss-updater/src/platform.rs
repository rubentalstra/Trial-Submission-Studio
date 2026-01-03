//! Platform detection and asset matching.
//!
//! Detects the current platform and architecture to find the correct
//! release asset for updates.

use crate::error::{Result, UpdateError};
use crate::release::{Asset, Release};

/// Supported operating systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    /// macOS / Darwin.
    MacOs,
    /// Microsoft Windows.
    Windows,
    /// Linux.
    Linux,
}

impl Os {
    /// Detect the current operating system.
    #[must_use]
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }

    /// Get the asset name pattern for this OS.
    #[must_use]
    pub const fn asset_pattern(&self) -> &'static str {
        match self {
            Self::MacOs => "macos",
            Self::Windows => "windows",
            Self::Linux => "linux",
        }
    }

    /// Get the expected archive extension for auto-updates.
    #[must_use]
    pub const fn archive_extension(&self) -> &'static str {
        match self {
            Self::MacOs | Self::Windows => "zip",
            Self::Linux => "tar.gz",
        }
    }

    /// Get a human-readable name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::MacOs => "macOS",
            Self::Windows => "Windows",
            Self::Linux => "Linux",
        }
    }
}

/// Supported CPU architectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    /// x86_64 / AMD64 / Intel 64-bit.
    X86_64,
    /// ARM64 / AArch64 / Apple Silicon.
    Aarch64,
}

impl Arch {
    /// Detect the current architecture.
    #[must_use]
    pub fn current() -> Self {
        if cfg!(target_arch = "aarch64") {
            Self::Aarch64
        } else {
            Self::X86_64
        }
    }

    /// Get the asset name pattern for this architecture.
    #[must_use]
    pub const fn asset_pattern(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "arm64", // Common naming convention
        }
    }

    /// Alternative patterns that might be used.
    #[must_use]
    pub const fn alternative_patterns(&self) -> &'static [&'static str] {
        match self {
            Self::X86_64 => &["x86_64", "x64", "amd64"],
            Self::Aarch64 => &["arm64", "aarch64", "apple-silicon"],
        }
    }

    /// Get a human-readable name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "ARM64",
        }
    }
}

/// Current platform information.
#[derive(Debug, Clone, Copy)]
pub struct Platform {
    /// The operating system.
    pub os: Os,
    /// The CPU architecture.
    pub arch: Arch,
}

impl Platform {
    /// Detect the current platform.
    #[must_use]
    pub fn current() -> Self {
        Self {
            os: Os::current(),
            arch: Arch::current(),
        }
    }

    /// Get a display string for this platform.
    #[must_use]
    pub fn display_name(&self) -> String {
        format!("{} {}", self.os.display_name(), self.arch.display_name())
    }

    /// Find the matching asset in a release for this platform.
    ///
    /// This looks for ZIP/TAR.GZ assets (not DMG) since we use those for auto-updates.
    pub fn find_asset<'a>(&self, release: &'a Release) -> Result<&'a Asset> {
        let os_pattern = self.os.asset_pattern();
        let extension = self.os.archive_extension();

        // Try each architecture pattern
        for arch_pattern in self.arch.alternative_patterns() {
            for asset in &release.assets {
                // Skip checksum files
                if asset.is_checksum() {
                    continue;
                }

                // Skip DMG files (those are for manual first-time installs)
                if asset.is_dmg() {
                    continue;
                }

                let name_lower = asset.name.to_lowercase();

                // Check if asset matches OS, arch, and extension
                let matches_os = name_lower.contains(os_pattern);
                let matches_arch = name_lower.contains(arch_pattern);
                let matches_ext = name_lower.ends_with(&format!(".{extension}"));

                if matches_os && matches_arch && matches_ext {
                    return Ok(asset);
                }
            }
        }

        Err(UpdateError::NoCompatibleRelease {
            platform: self.os.display_name().to_string(),
            arch: self.arch.display_name().to_string(),
        })
    }

    /// Get the target triple for this platform.
    #[must_use]
    pub fn target_triple(&self) -> &'static str {
        match (self.os, self.arch) {
            (Os::MacOs, Arch::Aarch64) => "aarch64-apple-darwin",
            (Os::MacOs, Arch::X86_64) => "x86_64-apple-darwin",
            (Os::Windows, Arch::X86_64) => "x86_64-pc-windows-msvc",
            (Os::Windows, Arch::Aarch64) => "aarch64-pc-windows-msvc",
            (Os::Linux, Arch::X86_64) => "x86_64-unknown-linux-gnu",
            (Os::Linux, Arch::Aarch64) => "aarch64-unknown-linux-gnu",
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_asset(name: &str) -> Asset {
        Asset {
            name: name.to_string(),
            content_type: "application/octet-stream".to_string(),
            size: 1024,
            download_count: 0,
            browser_download_url: format!("https://example.com/{name}"),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_release(assets: Vec<Asset>) -> Release {
        Release {
            tag_name: "v1.0.0".to_string(),
            name: Some("Release 1.0.0".to_string()),
            body: Some("Release notes".to_string()),
            draft: false,
            prerelease: false,
            created_at: Utc::now(),
            published_at: Some(Utc::now()),
            assets,
            html_url: "https://github.com/test/test/releases/v1.0.0".to_string(),
        }
    }

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        // Just ensure it doesn't panic
        assert!(!platform.display_name().is_empty());
        assert!(!platform.target_triple().is_empty());
    }

    #[test]
    fn test_find_macos_arm64_asset() {
        let release = make_release(vec![
            make_asset("trial-submission-studio-v1.0.0-macos-arm64.zip"),
            make_asset("trial-submission-studio-v1.0.0-macos-arm64.zip.sha256"),
            make_asset("trial-submission-studio-v1.0.0-macos-x86_64.zip"),
            make_asset("trial-submission-studio-v1.0.0-windows-x86_64.zip"),
            make_asset("trial-submission-studio-v1.0.0-linux-x86_64.tar.gz"),
        ]);

        let platform = Platform {
            os: Os::MacOs,
            arch: Arch::Aarch64,
        };

        let asset = platform.find_asset(&release).unwrap();
        assert_eq!(asset.name, "trial-submission-studio-v1.0.0-macos-arm64.zip");
    }

    #[test]
    fn test_find_windows_asset() {
        let release = make_release(vec![
            make_asset("trial-submission-studio-v1.0.0-macos-arm64.zip"),
            make_asset("trial-submission-studio-v1.0.0-windows-x86_64.zip"),
            make_asset("trial-submission-studio-v1.0.0-linux-x86_64.tar.gz"),
        ]);

        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };

        let asset = platform.find_asset(&release).unwrap();
        assert_eq!(
            asset.name,
            "trial-submission-studio-v1.0.0-windows-x86_64.zip"
        );
    }

    #[test]
    fn test_find_linux_asset() {
        let release = make_release(vec![
            make_asset("trial-submission-studio-v1.0.0-macos-arm64.zip"),
            make_asset("trial-submission-studio-v1.0.0-windows-x86_64.zip"),
            make_asset("trial-submission-studio-v1.0.0-linux-x86_64.tar.gz"),
        ]);

        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };

        let asset = platform.find_asset(&release).unwrap();
        assert_eq!(
            asset.name,
            "trial-submission-studio-v1.0.0-linux-x86_64.tar.gz"
        );
    }

    #[test]
    fn test_no_compatible_asset() {
        let release = make_release(vec![make_asset(
            "trial-submission-studio-v1.0.0-freebsd-x86_64.tar.gz",
        )]);

        let platform = Platform {
            os: Os::MacOs,
            arch: Arch::Aarch64,
        };

        let result = platform.find_asset(&release);
        assert!(result.is_err());
    }

    #[test]
    fn test_skips_dmg_for_updates() {
        let release = make_release(vec![
            make_asset("Trial-Submission-Studio-v1.0.0-macos-arm64.dmg"),
            make_asset("trial-submission-studio-v1.0.0-macos-arm64.zip"),
        ]);

        let platform = Platform {
            os: Os::MacOs,
            arch: Arch::Aarch64,
        };

        let asset = platform.find_asset(&release).unwrap();
        // Should pick the zip, not the dmg
        assert!(asset.name.ends_with(".zip"));
    }
}
