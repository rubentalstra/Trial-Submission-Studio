//! Trial Submission Studio - GUI Library
//!
//! This module provides the core application types and modules for the
//! Trial Submission Studio desktop application.
//!
//! Built with Iced 0.14.0 using the Elm architecture.

// Application constants
pub mod constants;

// Error types
pub mod error;

// Utility macros
mod util;

// Core modules (new Iced implementation)
pub mod app;
pub mod component;
pub mod handler;
pub mod menu;
pub mod message;
pub mod state;
pub mod theme;
pub mod view;

// Service modules for background tasks
pub mod service;
