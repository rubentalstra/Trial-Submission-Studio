//! Semantic versioning support for update comparisons.
//!
//! Handles parsing of version strings from GitHub tags (e.g., "v0.1.0", "0.1.0-beta.1")
//! and comparison logic for determining if an update is available.

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use crate::error::{Result, UpdateError};

/// Pre-release identifier for version comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreRelease {
    /// Alpha release (e.g., "alpha.1").
    Alpha(u32),
    /// Beta release (e.g., "beta.2").
    Beta(u32),
    /// Release candidate (e.g., "rc.1").
    ReleaseCandidate(u32),
}

impl PreRelease {
    /// Parse a pre-release string (e.g., "beta.1", "alpha.2", "rc.3").
    fn parse(s: &str) -> Option<Self> {
        let s = s.to_lowercase();

        if let Some(num) = s.strip_prefix("alpha.") {
            num.parse().ok().map(PreRelease::Alpha)
        } else if let Some(num) = s.strip_prefix("beta.") {
            num.parse().ok().map(PreRelease::Beta)
        } else if let Some(num) = s.strip_prefix("rc.") {
            num.parse().ok().map(PreRelease::ReleaseCandidate)
        } else if s == "alpha" {
            Some(PreRelease::Alpha(0))
        } else if s == "beta" {
            Some(PreRelease::Beta(0))
        } else if s == "rc" {
            Some(PreRelease::ReleaseCandidate(0))
        } else {
            None
        }
    }

    /// Get the ordering priority (alpha < beta < rc).
    fn priority(&self) -> u8 {
        match self {
            PreRelease::Alpha(_) => 0,
            PreRelease::Beta(_) => 1,
            PreRelease::ReleaseCandidate(_) => 2,
        }
    }

    /// Get the numeric component.
    fn number(&self) -> u32 {
        match self {
            PreRelease::Alpha(n) | PreRelease::Beta(n) | PreRelease::ReleaseCandidate(n) => *n,
        }
    }
}

impl PartialOrd for PreRelease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreRelease {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority().cmp(&other.priority()) {
            Ordering::Equal => self.number().cmp(&other.number()),
            other => other,
        }
    }
}

impl fmt::Display for PreRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreRelease::Alpha(n) => write!(f, "alpha.{n}"),
            PreRelease::Beta(n) => write!(f, "beta.{n}"),
            PreRelease::ReleaseCandidate(n) => write!(f, "rc.{n}"),
        }
    }
}

/// A semantic version with optional pre-release tag.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Version {
    /// Major version number.
    pub major: u32,
    /// Minor version number.
    pub minor: u32,
    /// Patch version number.
    pub patch: u32,
    /// Optional pre-release identifier.
    pub pre_release: Option<PreRelease>,
}

impl Version {
    /// Create a new stable version.
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    /// Create a new version with a pre-release tag.
    #[must_use]
    pub const fn with_pre_release(
        major: u32,
        minor: u32,
        patch: u32,
        pre_release: PreRelease,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: Some(pre_release),
        }
    }

    /// Get the current application version from Cargo.toml.
    #[must_use]
    pub fn current() -> Self {
        Self::from_str(env!("CARGO_PKG_VERSION")).unwrap_or_else(|_| Self::new(0, 0, 0))
    }

    /// Check if this version is a pre-release (alpha, beta, or rc).
    #[must_use]
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }

    /// Check if this version is a stable release.
    #[must_use]
    pub fn is_stable(&self) -> bool {
        self.pre_release.is_none()
    }

    /// Parse a version from a GitHub release tag (handles "v" prefix).
    pub fn from_tag(tag: &str) -> Result<Self> {
        let tag = tag.strip_prefix('v').unwrap_or(tag);
        Self::from_str(tag)
    }
}

impl FromStr for Version {
    type Err = UpdateError;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        let s = s.strip_prefix('v').unwrap_or(s);

        // Split into version and pre-release parts
        let (version_part, pre_release) = if let Some(idx) = s.find('-') {
            let (v, p) = s.split_at(idx);
            (v, PreRelease::parse(&p[1..]))
        } else {
            (s, None)
        };

        // Parse major.minor.patch
        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(UpdateError::InvalidVersion(s.to_string()));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| UpdateError::InvalidVersion(s.to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| UpdateError::InvalidVersion(s.to_string()))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| UpdateError::InvalidVersion(s.to_string()))?;

        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre_release {
            write!(f, "-{pre}")?;
        }
        Ok(())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major.minor.patch first
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            other => return other,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            other => return other,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            other => return other,
        }

        // Pre-release versions are always less than stable versions
        // e.g., 1.0.0-beta.1 < 1.0.0
        match (&self.pre_release, &other.pre_release) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stable_version() {
        let v = Version::from_str("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.is_stable());
    }

    #[test]
    fn test_parse_version_with_v_prefix() {
        let v = Version::from_tag("v1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_parse_beta_version() {
        let v = Version::from_str("1.0.0-beta.1").unwrap();
        assert_eq!(v.pre_release, Some(PreRelease::Beta(1)));
        assert!(v.is_pre_release());
    }

    #[test]
    fn test_parse_alpha_version() {
        let v = Version::from_str("2.0.0-alpha.5").unwrap();
        assert_eq!(v.pre_release, Some(PreRelease::Alpha(5)));
    }

    #[test]
    fn test_parse_rc_version() {
        let v = Version::from_str("1.0.0-rc.2").unwrap();
        assert_eq!(v.pre_release, Some(PreRelease::ReleaseCandidate(2)));
    }

    #[test]
    fn test_version_ordering() {
        let v1 = Version::from_str("1.0.0").unwrap();
        let v2 = Version::from_str("1.0.1").unwrap();
        let v3 = Version::from_str("1.1.0").unwrap();
        let v4 = Version::from_str("2.0.0").unwrap();

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
    }

    #[test]
    fn test_prerelease_ordering() {
        let alpha = Version::from_str("1.0.0-alpha.1").unwrap();
        let beta = Version::from_str("1.0.0-beta.1").unwrap();
        let rc = Version::from_str("1.0.0-rc.1").unwrap();
        let stable = Version::from_str("1.0.0").unwrap();

        assert!(alpha < beta);
        assert!(beta < rc);
        assert!(rc < stable);
    }

    #[test]
    fn test_prerelease_number_ordering() {
        let beta1 = Version::from_str("1.0.0-beta.1").unwrap();
        let beta2 = Version::from_str("1.0.0-beta.2").unwrap();
        let beta10 = Version::from_str("1.0.0-beta.10").unwrap();

        assert!(beta1 < beta2);
        assert!(beta2 < beta10);
    }

    #[test]
    fn test_version_display() {
        let v = Version::from_str("1.2.3-beta.4").unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta.4");
    }

    #[test]
    fn test_invalid_version() {
        assert!(Version::from_str("invalid").is_err());
        assert!(Version::from_str("1.2").is_err());
        assert!(Version::from_str("1.2.3.4").is_err());
    }
}
