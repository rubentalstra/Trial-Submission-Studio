//! Configuration types for the update system.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::version::{PreRelease, Version};

/// Update channel selection.
///
/// Channels are ordered by stability: Stable > ReleaseCandidate > Beta > Alpha
/// Each channel includes all releases from more stable channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannel {
    /// Receive only stable releases (most conservative).
    #[default]
    Stable,

    /// Receive release candidates and stable releases.
    /// RC builds are feature-complete and undergoing final testing.
    ReleaseCandidate,

    /// Receive beta, RC, and stable releases.
    /// Beta builds have stable APIs but may have bugs.
    Beta,

    /// Receive all releases including alpha (least conservative).
    /// Alpha builds are experimental and may be unstable.
    Alpha,
}

impl UpdateChannel {
    /// Returns all channel variants for UI enumeration.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Stable,
            Self::ReleaseCandidate,
            Self::Beta,
            Self::Alpha,
        ]
    }

    /// Check if a version should be shown for this channel.
    #[must_use]
    pub fn includes(&self, version: &Version) -> bool {
        match (self, &version.pre_release) {
            // Stable channel: only stable versions
            (Self::Stable, None) => true,
            (Self::Stable, Some(_)) => false,

            // RC channel: stable + RC
            (Self::ReleaseCandidate, None) => true,
            (Self::ReleaseCandidate, Some(PreRelease::ReleaseCandidate(_))) => true,
            (Self::ReleaseCandidate, Some(_)) => false,

            // Beta channel: stable + RC + beta
            (Self::Beta, None) => true,
            (Self::Beta, Some(PreRelease::ReleaseCandidate(_))) => true,
            (Self::Beta, Some(PreRelease::Beta(_))) => true,
            (Self::Beta, Some(PreRelease::Alpha(_))) => false,

            // Alpha channel: everything
            (Self::Alpha, _) => true,
        }
    }

    /// Get a human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::ReleaseCandidate => "Release Candidate",
            Self::Beta => "Beta",
            Self::Alpha => "Alpha",
        }
    }

    /// Get a description of this channel.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Stable => "Receive only stable releases (recommended)",
            Self::ReleaseCandidate => "Receive release candidates and stable releases",
            Self::Beta => "Receive beta, release candidate, and stable releases",
            Self::Alpha => "Receive all releases including experimental alpha builds",
        }
    }
}

impl fmt::Display for UpdateChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Minimum time between automatic startup checks (in hours).
pub const AUTO_CHECK_INTERVAL_HOURS: i64 = 24;

/// Minimum time between manual update checks to prevent spam (in seconds).
pub const MANUAL_CHECK_COOLDOWN_SECS: i64 = 300; // 5 minutes

/// User settings for the update system.
///
/// By default, automatic startup checks are disabled, giving users
/// control over when checks happen. Users can always manually check via menu.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSettings {
    /// Automatically check for updates on app startup.
    /// If false, user must manually check via menu.
    /// Default: `false`
    #[serde(default)]
    pub check_on_startup: bool,

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

impl UpdateSettings {
    /// Check if automatic startup check should run.
    ///
    /// Returns `true` only if:
    /// - Automatic startup checks are enabled
    /// - At least 24 hours have passed since the last check
    #[must_use]
    pub fn should_auto_check(&self) -> bool {
        self.check_on_startup && self.hours_since_last_check() >= AUTO_CHECK_INTERVAL_HOURS
    }

    /// Check if enough time has passed to allow a manual check.
    ///
    /// Returns `true` if at least 5 minutes have passed since the last check (cooldown).
    #[must_use]
    pub fn can_check_manually(&self) -> bool {
        self.seconds_since_last_check() >= MANUAL_CHECK_COOLDOWN_SECS
    }

    /// Get the number of hours since the last check.
    #[must_use]
    pub fn hours_since_last_check(&self) -> i64 {
        match self.last_check {
            None => i64::MAX, // Never checked, so "infinite" hours ago
            Some(last) => {
                let now = Utc::now();
                now.signed_duration_since(last).num_hours()
            }
        }
    }

    /// Get the number of seconds since the last check.
    #[must_use]
    pub fn seconds_since_last_check(&self) -> i64 {
        match self.last_check {
            None => i64::MAX, // Never checked
            Some(last) => {
                let now = Utc::now();
                now.signed_duration_since(last).num_seconds()
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

    /// Enable automatic startup checks.
    pub fn enable_auto_check(&mut self) {
        self.check_on_startup = true;
    }

    /// Disable automatic startup checks.
    pub fn disable_auto_check(&mut self) {
        self.check_on_startup = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = UpdateSettings::default();
        // Default: no auto-check on startup
        assert!(!settings.check_on_startup);
        assert_eq!(settings.channel, UpdateChannel::Stable);
        assert!(settings.skipped_version.is_none());
        assert!(settings.last_check.is_none());
    }

    #[test]
    fn test_should_auto_check_no_startup_check() {
        let settings = UpdateSettings::default();
        // check_on_startup=false by default
        assert!(!settings.should_auto_check());
    }

    #[test]
    fn test_should_auto_check_with_startup_check() {
        let mut settings = UpdateSettings::default();
        settings.check_on_startup = true;
        // No last_check, so should return true
        assert!(settings.should_auto_check());
    }

    #[test]
    fn test_should_auto_check_respects_interval() {
        let mut settings = UpdateSettings::default();
        settings.check_on_startup = true;
        settings.last_check = Some(Utc::now()); // Just checked

        // Should not auto-check because we just checked
        assert!(!settings.should_auto_check());
    }

    #[test]
    fn test_can_check_manually_no_cooldown() {
        let settings = UpdateSettings::default();
        // Never checked, so can check manually
        assert!(settings.can_check_manually());
    }

    #[test]
    fn test_can_check_manually_with_cooldown() {
        let mut settings = UpdateSettings::default();
        settings.last_check = Some(Utc::now()); // Just checked

        // Should not be able to check manually due to cooldown
        assert!(!settings.can_check_manually());
    }

    use std::str::FromStr;

    #[test]
    fn test_channel_includes() {
        let stable = Version::new(1, 0, 0);
        let rc = Version::from_str("1.0.0-rc.1").unwrap();
        let beta = Version::from_str("1.0.0-beta.1").unwrap();
        let alpha = Version::from_str("1.0.0-alpha.1").unwrap();

        // Stable channel: only stable versions
        assert!(UpdateChannel::Stable.includes(&stable));
        assert!(!UpdateChannel::Stable.includes(&rc));
        assert!(!UpdateChannel::Stable.includes(&beta));
        assert!(!UpdateChannel::Stable.includes(&alpha));

        // RC channel: stable + RC
        assert!(UpdateChannel::ReleaseCandidate.includes(&stable));
        assert!(UpdateChannel::ReleaseCandidate.includes(&rc));
        assert!(!UpdateChannel::ReleaseCandidate.includes(&beta));
        assert!(!UpdateChannel::ReleaseCandidate.includes(&alpha));

        // Beta channel: stable + RC + beta
        assert!(UpdateChannel::Beta.includes(&stable));
        assert!(UpdateChannel::Beta.includes(&rc));
        assert!(UpdateChannel::Beta.includes(&beta));
        assert!(!UpdateChannel::Beta.includes(&alpha));

        // Alpha channel: everything
        assert!(UpdateChannel::Alpha.includes(&stable));
        assert!(UpdateChannel::Alpha.includes(&rc));
        assert!(UpdateChannel::Alpha.includes(&beta));
        assert!(UpdateChannel::Alpha.includes(&alpha));
    }

    #[test]
    fn test_channel_all() {
        let all = UpdateChannel::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], UpdateChannel::Stable);
        assert_eq!(all[1], UpdateChannel::ReleaseCandidate);
        assert_eq!(all[2], UpdateChannel::Beta);
        assert_eq!(all[3], UpdateChannel::Alpha);
    }

    #[test]
    fn test_channel_labels() {
        assert_eq!(UpdateChannel::Stable.label(), "Stable");
        assert_eq!(UpdateChannel::ReleaseCandidate.label(), "Release Candidate");
        assert_eq!(UpdateChannel::Beta.label(), "Beta");
        assert_eq!(UpdateChannel::Alpha.label(), "Alpha");
    }

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

    #[test]
    fn test_enable_disable_auto_check() {
        let mut settings = UpdateSettings::default();
        assert!(!settings.check_on_startup);

        settings.enable_auto_check();
        assert!(settings.check_on_startup);

        settings.disable_auto_check();
        assert!(!settings.check_on_startup);
    }
}
