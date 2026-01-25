//! Persistence types for project serialization.
//!
//! These types are designed for zero-copy deserialization with rkyv.
//! They mirror the GUI state types but are optimized for storage.

mod domain;
mod generated_domains;
mod placeholders;
mod project;
mod source;
mod supp;

pub use domain::{
    DomainSnapshot, GeneratedDomainSnapshot, MappingEntry, MappingSnapshot, SourceDomainSnapshot,
};
pub use generated_domains::{
    CommentEntrySnapshot, GeneratedDomainEntrySnapshot, GeneratedDomainTypeSnapshot,
    RelrecEntrySnapshot, RelrecRelTypeSnapshot, RelspecEntrySnapshot, RelsubEntrySnapshot,
};
pub use placeholders::ProjectPlaceholders;
pub use project::{ProjectFile, StudyMetadata, WorkflowTypeSnapshot};
pub use source::SourceAssignment;
pub use supp::{SuppActionSnapshot, SuppColumnSnapshot, SuppOriginSnapshot};

/// Current schema version.
///
/// Increment this when making breaking changes to the persistence format.
/// The loader will reject files with version > CURRENT_SCHEMA_VERSION.
///
/// v2: Added generated domain support (DomainSnapshot now enum with Source/Generated variants)
pub const CURRENT_SCHEMA_VERSION: u32 = 2;

/// Magic bytes at the start of .tss files.
///
/// Format: "TSS" + version byte (0x02 for v2)
pub const MAGIC_BYTES: [u8; 4] = [b'T', b'S', b'S', 0x02];
