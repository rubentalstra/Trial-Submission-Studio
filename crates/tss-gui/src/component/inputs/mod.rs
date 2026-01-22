//! Input components for user interaction.
//!
//! This module contains components for user input:
//!
//! - **SearchBox**: Search input with icon and clear button
//! - **TextField**: Text input with validation and labels
//! - **FormField**: Labeled form fields with various input types
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::inputs::{TextField, search_box};
//!
//! // Text field with validation
//! TextField::new("Username", &value, "Enter username", |s| Msg::Input(s))
//!     .required(true)
//!     .max_length(50)
//!     .view();
//!
//! // Search box
//! search_box(&search_text, "Search...", |s| Msg::Search(s));
//! ```

mod form_field;
mod search_box;
mod text_field;

pub use form_field::{form_field, number_field};
pub use search_box::{search_box, search_box_compact, search_box_with_filter};
pub use text_field::{TextAreaField, TextField};
