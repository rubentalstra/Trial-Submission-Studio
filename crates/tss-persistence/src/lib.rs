//! Persistent storage for Trial Submission Studio projects.
//!
//! This crate provides functionality to save and load `.tss` project files,
//! enabling users to save their work and continue later.
//!
//! # Features
//!
//! - **Zero-copy serialization** with rkyv for fast load times
//! - **Atomic writes** to prevent data corruption
//! - **Source change detection** via SHA-256 hashing
//! - **Auto-save** with debounce support
//!
//! # File Format
//!
//! `.tss` files use a simple binary format:
//!
//! ```text
//! +------------------+
//! | Magic: "TSS\x01" | 4 bytes - file identification
//! +------------------+
//! | Version: 1       | 4 bytes - u32 little-endian schema version
//! +------------------+
//! | rkyv Payload     | Variable - zero-copy deserializable
//! +------------------+
//! ```
//!
//! # Example
//!
//! ```ignore
//! use tss_persistence::{ProjectFile, StudyMetadata, WorkflowTypeSnapshot};
//! use tss_persistence::{save_project, load_project};
//!
//! // Create a new project
//! let study = StudyMetadata::new("DEMO_STUDY", "/path/to/study", WorkflowTypeSnapshot::Sdtm);
//! let mut project = ProjectFile::new(study);
//!
//! // Save to disk
//! save_project(&mut project, Path::new("demo.tss"))?;
//!
//! // Load from disk
//! let loaded = load_project(Path::new("demo.tss"))?;
//! ```
//!
//! # Architecture
//!
//! The crate is organized into:
//!
//! - `types/` - Persistence types (rkyv-serializable snapshots)
//! - `io/` - File I/O operations (save, load, hash)
//! - `autosave/` - Auto-save infrastructure (DirtyTracker, config)
//! - `convert.rs` - Conversion traits and helpers
//! - `error.rs` - Error types with user-friendly messages

mod autosave;
mod convert;
mod error;
mod io;
mod types;

// Re-export main types
pub use autosave::{AutoSaveConfig, DirtyTracker};
pub use convert::{
    FromSnapshot, SuppActionConvert, SuppOriginConvert, ToSnapshot, mapping_to_snapshot,
    supp_column_to_snapshot,
};
pub use error::{PersistenceError, Result};
pub use io::{
    compute_file_hash, load_project, load_project_async, save_project, save_project_async,
    verify_file_hash,
};
pub use types::{
    CURRENT_SCHEMA_VERSION, DomainSnapshot, MAGIC_BYTES, MappingEntry, MappingSnapshot,
    ProjectFile, ProjectPlaceholders, SourceAssignment, StudyMetadata, SuppActionSnapshot,
    SuppColumnSnapshot, SuppOriginSnapshot, WorkflowTypeSnapshot,
};
