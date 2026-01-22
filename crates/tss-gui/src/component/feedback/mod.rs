//! Feedback components for user notifications.
//!
//! This module contains components for user feedback:
//!
//! - **Toast**: Non-blocking notifications
//! - **Modal**: Dialog overlays
//! - **ProgressModal**: Progress indicators in modal
//! - **ProgressBar**: Progress indicators
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::feedback::{modal, ProgressBar};
//!
//! // Modal dialog
//! modal("Title", content, on_close);
//!
//! // Progress bar
//! ProgressBar::new(0.75)
//!     .label("Processing...")
//!     .view();
//! ```

mod modal;
mod progress_bar;
mod progress_modal;
pub mod toast;

pub use modal::{confirm_modal, modal};
pub use progress_bar::ProgressBar;
pub use progress_modal::progress_modal;
