//! Configuration types for the update system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::version::Version;

/// How often to check for updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateCheckFrequency {
    /// Never automatically check for updates (user must manually check).
    /// This is the default to comply with SignPath Foundation privacy requirements.
    #[default]
    Disabled,
    /// Check for updates when the application starts.
    OnStartup,
    /// Check for updates once per day.
    Daily,
}

impl UpdateCheckFrequency {
    /// Get all available frequency options.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Disabled, Self::OnStartup, Self::Daily]
    }

    /// Get a human-readable label for this frequency.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Disabled => "Disabled",
            Self::OnStartup => "On Startup",
            Self::Daily => "Daily",
        }
    }

    /// Get a description of this frequency.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Disabled => "Never check for updates automatically",
            Self::OnStartup => "Check for updates each time the application starts",
            Self::Daily => "Check for updates once per day",
        }
    }
}

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
}

/// User settings for the update system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSettings {
    /// How often to check for updates.
    pub check_frequency: UpdateCheckFrequency,

    /// Which release channel to follow.
    pub channel: UpdateChannel,

    /// Version to skip (user clicked "Skip This Version").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skipped_version: Option<String>,

    /// Last time we checked for updates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_check_time: Option<DateTime<Utc>>,

    /// Last time we notified the user about an available update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_notification_time: Option<DateTime<Utc>>,
}

impl UpdateSettings {
    /// Check if we should perform an automatic update check.
    ///
    /// This considers the check frequency setting and the last check time.
    #[must_use]
    pub fn should_check_now(&self) -> bool {
        match self.check_frequency {
            UpdateCheckFrequency::Disabled => false,
            UpdateCheckFrequency::OnStartup => true,
            UpdateCheckFrequency::Daily => {
                match self.last_check_time {
                    None => true, // Never checked before
                    Some(last_check) => {
                        let now = Utc::now();
                        let elapsed = now.signed_duration_since(last_check);
                        // Check if at least 24 hours have passed
                        elapsed.num_hours() >= 24
                    }
                }
            }
        }
    }

    /// Check if a version should be skipped based on user preference.
    #[must_use]
    pub fn should_skip_version(&self, version: &Version) -> bool {
        match &self.skipped_version {
            Some(skipped) => skipped == &version.to_string(),
            None => false,
        }
    }

    /// Record that we just checked for updates.
    pub fn record_check(&mut self) {
        self.last_check_time = Some(Utc::now());
    }

    /// Record that we notified the user about an update.
    pub fn record_notification(&mut self) {
        self.last_notification_time = Some(Utc::now());
    }

    /// Clear the skipped version.
    pub fn clear_skipped_version(&mut self) {
        self.skipped_version = None;
    }

    /// Set a version to skip.
    pub fn skip_version(&mut self, version: &Version) {
        self.skipped_version = Some(version.to_string());
    }
}

/// Minimum time between manual update checks to prevent spam (in seconds).
pub const MANUAL_CHECK_COOLDOWN_SECS: i64 = 300; // 5 minutes

impl UpdateSettings {
    /// Check if enough time has passed since the last check to allow a manual check.
    ///
    /// This prevents spamming the GitHub API with rapid manual checks.
    #[must_use]
    pub fn can_check_manually(&self) -> bool {
        match self.last_check_time {
            None => true,
            Some(last_check) => {
                let now = Utc::now();
                let elapsed = now.signed_duration_since(last_check);
                elapsed.num_seconds() >= MANUAL_CHECK_COOLDOWN_SECS
            }
        }
    }

    /// Get the number of seconds until manual check is allowed again.
    #[must_use]
    pub fn seconds_until_manual_check_allowed(&self) -> Option<i64> {
        match self.last_check_time {
            None => None,
            Some(last_check) => {
                let now = Utc::now();
                let elapsed = now.signed_duration_since(last_check);
                let remaining = MANUAL_CHECK_COOLDOWN_SECS - elapsed.num_seconds();
                if remaining > 0 { Some(remaining) } else { None }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = UpdateSettings::default();
        // Default is Disabled to comply with SignPath privacy requirements
        assert_eq!(settings.check_frequency, UpdateCheckFrequency::Disabled);
        assert_eq!(settings.channel, UpdateChannel::Stable);
        assert!(settings.skipped_version.is_none());
    }

    #[test]
    fn test_should_check_disabled() {
        let settings = UpdateSettings {
            check_frequency: UpdateCheckFrequency::Disabled,
            ..Default::default()
        };
        assert!(!settings.should_check_now());
    }

    #[test]
    fn test_should_check_on_startup() {
        let settings = UpdateSettings {
            check_frequency: UpdateCheckFrequency::OnStartup,
            ..Default::default()
        };
        assert!(settings.should_check_now());
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
        let version = Version::from_str("1.2.3").unwrap();

        assert!(!settings.should_skip_version(&version));

        settings.skip_version(&version);
        assert!(settings.should_skip_version(&version));

        settings.clear_skipped_version();
        assert!(!settings.should_skip_version(&version));
    }
}
