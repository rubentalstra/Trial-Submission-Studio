//! Persistence types for project serialization.
//!
//! These types are designed for zero-copy deserialization with rkyv.
//! They mirror the GUI state types but are optimized for storage.

mod domain;
mod placeholders;
mod project;
mod source;
mod supp;

pub use domain::{DomainSnapshot, MappingEntry, MappingSnapshot};
pub use placeholders::ProjectPlaceholders;
pub use project::{ProjectFile, StudyMetadata, WorkflowTypeSnapshot};
pub use source::SourceAssignment;
pub use supp::{SuppActionSnapshot, SuppColumnSnapshot, SuppOriginSnapshot};

/// Current schema version.
///
/// Increment this when making breaking changes to the persistence format.
/// The loader will reject files with version > CURRENT_SCHEMA_VERSION.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Magic bytes at the start of .tss files.
///
/// Format: "TSS" + version byte (0x01 for v1)
pub const MAGIC_BYTES: [u8; 4] = [b'T', b'S', b'S', 0x01];
