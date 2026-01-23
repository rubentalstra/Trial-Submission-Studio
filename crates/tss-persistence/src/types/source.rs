//! Source file assignment types.

use rkyv::{Archive, Deserialize, Serialize};

/// Assignment of a source CSV file to a domain.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct SourceAssignment {
    /// Path to the source CSV file (relative to project file or absolute).
    pub file_path: String,

    /// Domain code (e.g., "DM", "AE").
    pub domain_code: String,

    /// SHA-256 hash of the file content at save time.
    /// Used for change detection on project load.
    pub content_hash: String,

    /// File size in bytes at save time.
    pub file_size: u64,

    /// Last modified timestamp (RFC 3339 format).
    pub last_modified: Option<String>,
}

impl SourceAssignment {
    /// Create a new source assignment.
    pub fn new(
        file_path: impl Into<String>,
        domain_code: impl Into<String>,
        content_hash: impl Into<String>,
        file_size: u64,
    ) -> Self {
        Self {
            file_path: file_path.into(),
            domain_code: domain_code.into(),
            content_hash: content_hash.into(),
            file_size,
            last_modified: None,
        }
    }

    /// Set the last modified timestamp.
    pub fn with_last_modified(mut self, timestamp: impl Into<String>) -> Self {
        self.last_modified = Some(timestamp.into());
        self
    }
}
