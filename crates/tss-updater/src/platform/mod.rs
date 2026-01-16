//! Platform-specific installation logic.
//!
//! This module provides a clean abstraction for platform-specific update installation:
//! - **macOS**: Downloads full .app bundle, verifies signature, spawns helper to swap bundles
//! - **Windows/Linux**: Uses `self_replace` for in-place binary replacement

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(target_os = "macos"))]
mod desktop;
#[cfg(not(target_os = "macos"))]
pub use desktop::*;
