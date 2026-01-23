//! Auto-save functionality for projects.
//!
//! Provides:
//! - `DirtyTracker` - Tracks unsaved changes with debounce
//! - `AutoSaveConfig` - User settings for auto-save behavior

mod config;
mod tracker;

pub use config::AutoSaveConfig;
pub use tracker::DirtyTracker;
