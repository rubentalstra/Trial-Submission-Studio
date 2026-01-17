//! Individual steps of the update process.
//!
//! Each step is implemented as a separate module with functions that
//! can be called by the orchestrator.

pub mod check;
pub mod download;
pub mod extract;
pub mod install;
pub mod signature;
pub mod verify;
