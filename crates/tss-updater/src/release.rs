//! Update information types.

/// Information about an available update.
///
/// This is a simplified structure containing just what the UI needs to display.
/// All the heavy lifting (asset selection, download, installation) is handled
/// by the `self_update` crate.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// The version string (e.g., "v1.2.3" or "1.2.3-beta.1").
    pub version: String,

    /// The changelog/release notes in markdown format.
    pub changelog: String,
}

impl UpdateInfo {
    /// Create new update info.
    #[must_use]
    pub fn new(version: String, changelog: String) -> Self {
        Self { version, changelog }
    }

    /// Get the version without the "v" prefix if present.
    #[must_use]
    pub fn version_display(&self) -> &str {
        self.version.strip_prefix('v').unwrap_or(&self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_display() {
        let info = UpdateInfo::new("v1.2.3".to_string(), "Changes".to_string());
        assert_eq!(info.version_display(), "1.2.3");

        let info = UpdateInfo::new("1.2.3".to_string(), "Changes".to_string());
        assert_eq!(info.version_display(), "1.2.3");
    }
}
