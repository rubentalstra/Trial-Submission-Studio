//! Configuration types for the update system.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::version::Version;

/// Update channel selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannel {
    /// Only receive stable releases.
    #[default]
    Stable,
    /// Receive beta and stable releases.
    Beta,
}

impl UpdateChannel {
    /// Check if a version should be shown for this channel.
    #[must_use]
    pub fn includes(&self, version: &Version) -> bool {
        match self {
            Self::Stable => version.is_stable(),
            Self::Beta => true, // Beta channel includes all releases
        }
    }

    /// Get a human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Beta => "Beta",
        }
    }

    /// Get a description of this channel.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Stable => "Receive only stable releases",
            Self::Beta => "Receive beta and stable releases",
        }
    }
}

impl fmt::Display for UpdateChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// User settings for the update system.
///
/// By default, update checking is disabled to comply with privacy requirements.
/// Users must explicitly enable it in the application settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSettings {
    /// Whether update checking is enabled.
    /// Default is `false` to comply with SignPath Foundation privacy requirements.
    #[serde(default)]
    pub enabled: bool,

    /// Which release channel to follow.
    #[serde(default)]
    pub channel: UpdateChannel,

    /// Version to skip (user clicked "Skip This Version").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skipped_version: Option<String>,

    /// Last time we checked for updates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_check: Option<DateTime<Utc>>,
}

/// Minimum time between manual update checks to prevent spam (in seconds).
pub const MANUAL_CHECK_COOLDOWN_SECS: i64 = 300; // 5 minutes

impl UpdateSettings {
    /// Creates new settings with update checking enabled.
    #[must_use]
    pub fn with_enabled(enabled: bool) -> Self {
        Self {
            enabled,
            ..Default::default()
        }
    }

    /// Check if we should perform an automatic update check on startup.
    ///
    /// Returns `true` only if:
    /// - Update checking is enabled
    /// - Enough time has passed since the last check
    #[must_use]
    pub fn should_check_on_startup(&self) -> bool {
        if !self.enabled {
            return false;
        }

        // Check if at least 24 hours have passed since last check
        match self.last_check {
            None => true,
            Some(last) => {
                let now = Utc::now();
                let elapsed = now.signed_duration_since(last);
                elapsed.num_hours() >= 24
            }
        }
    }

    /// Check if enough time has passed to allow a manual check.
    ///
    /// This prevents spamming the GitHub API with rapid manual checks.
    #[must_use]
    pub fn can_check_manually(&self) -> bool {
        match self.last_check {
            None => true,
            Some(last) => {
                let now = Utc::now();
                let elapsed = now.signed_duration_since(last);
                elapsed.num_seconds() >= MANUAL_CHECK_COOLDOWN_SECS
            }
        }
    }

    /// Get the number of seconds until manual check is allowed again.
    #[must_use]
    pub fn seconds_until_manual_check_allowed(&self) -> Option<i64> {
        match self.last_check {
            None => None,
            Some(last) => {
                let now = Utc::now();
                let elapsed = now.signed_duration_since(last);
                let remaining = MANUAL_CHECK_COOLDOWN_SECS - elapsed.num_seconds();
                if remaining > 0 { Some(remaining) } else { None }
            }
        }
    }

    /// Check if a version should be skipped based on user preference.
    #[must_use]
    pub fn should_skip_version(&self, version: &str) -> bool {
        match &self.skipped_version {
            Some(skipped) => {
                skipped == version || skipped == version.strip_prefix('v').unwrap_or(version)
            }
            None => false,
        }
    }

    /// Record that we just checked for updates.
    pub fn record_check(&mut self) {
        self.last_check = Some(Utc::now());
    }

    /// Set a version to skip.
    pub fn skip_version(&mut self, version: impl Into<String>) {
        self.skipped_version = Some(version.into());
    }

    /// Clear the skipped version.
    pub fn clear_skipped_version(&mut self) {
        self.skipped_version = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = UpdateSettings::default();
        // Default is disabled to comply with privacy requirements
        assert!(!settings.enabled);
        assert_eq!(settings.channel, UpdateChannel::Stable);
        assert!(settings.skipped_version.is_none());
        assert!(settings.last_check.is_none());
    }

    #[test]
    fn test_should_check_on_startup_disabled() {
        let settings = UpdateSettings::default();
        assert!(!settings.should_check_on_startup());
    }

    #[test]
    fn test_should_check_on_startup_enabled() {
        let settings = UpdateSettings::with_enabled(true);
        assert!(settings.should_check_on_startup());
    }

    #[test]
    fn test_channel_includes() {
        let stable = Version::new(1, 0, 0);
        let beta = Version::from_str("1.0.0-beta.1").unwrap();

        assert!(UpdateChannel::Stable.includes(&stable));
        assert!(!UpdateChannel::Stable.includes(&beta));

        assert!(UpdateChannel::Beta.includes(&stable));
        assert!(UpdateChannel::Beta.includes(&beta));
    }

    use std::str::FromStr;

    #[test]
    fn test_skip_version() {
        let mut settings = UpdateSettings::default();

        assert!(!settings.should_skip_version("1.2.3"));

        settings.skip_version("1.2.3");
        assert!(settings.should_skip_version("1.2.3"));
        assert!(settings.should_skip_version("v1.2.3")); // Also matches with v prefix

        settings.clear_skipped_version();
        assert!(!settings.should_skip_version("1.2.3"));
    }

    #[test]
    fn test_record_check() {
        let mut settings = UpdateSettings::default();
        assert!(settings.last_check.is_none());

        settings.record_check();
        assert!(settings.last_check.is_some());
    }
}
