//! Version-tracked cache wrapper for derived state.
//!
//! This module provides `Versioned<T>` which wraps cached data with
//! the version number it was computed from. This enables automatic
//! cache invalidation when the source data changes.

/// Wrapper for cached data that tracks when it was computed.
///
/// The `source_version` field stores the `DomainState.version` at
/// the time this data was computed. When checking validity, compare
/// against the current version to determine if rebuild is needed.
#[derive(Debug, Clone)]
pub struct Versioned<T> {
    /// The cached data
    pub data: T,
    /// Version of DomainState when this was computed
    pub source_version: u64,
}

impl<T> Versioned<T> {
    /// Check if this cache entry is stale.
    ///
    /// Returns `true` if the source version has changed since this
    /// data was computed, meaning it should be rebuilt.
    #[inline]
    pub fn is_stale(&self, current_version: u64) -> bool {
        self.source_version != current_version
    }

    /// Check if this cache entry is current.
    #[inline]
    pub fn is_current(&self, current_version: u64) -> bool {
        self.source_version == current_version
    }
}
