//! Trial Submission Studio - GUI Library
//!
//! This module provides the core application types and modules for the
//! Trial Submission Studio desktop application.
//!
//! Built with Iced 0.14.0 using the Elm architecture.

// Core modules (new Iced implementation)
#[allow(dead_code)]
#[allow(unused_imports)]
pub mod component;
#[allow(dead_code)]
pub mod menu;
pub mod message;
pub mod state;
#[allow(dead_code)]
#[allow(unused_imports)]
pub mod theme;
pub mod view;

// Service modules for background tasks
pub mod service;
