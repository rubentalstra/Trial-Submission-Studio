//! Trial Submission Studio - GUI Library
//!
//! This module provides the core application types and modules for the
//! Trial Submission Studio desktop application.
//!
//! Built with Iced 0.14.0 using the Elm architecture.

// Core modules (new Iced implementation)
pub mod component;
pub mod message;
pub mod state;
pub mod theme;
pub mod view;

// Service modules for background tasks
pub mod service;

// These modules will be restored as they're ported to Iced:
// pub mod export;
// pub mod menu;
// pub mod settings;

// Legacy modules (commented out during migration)
// pub mod services;
// pub mod views;
