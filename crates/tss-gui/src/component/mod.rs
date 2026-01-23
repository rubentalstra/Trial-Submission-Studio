//! Reusable UI components for Trial Submission Studio.
//!
//! Components are organized into categories:
//! - `layout`: Page structure, sidebars, split views, tab bars
//! - `panels`: List panels, detail panels, headers, sections
//! - `display`: Badges, cards, tables, list items
//! - `inputs`: Search boxes, text fields, form fields
//! - `feedback`: Toasts, modals, progress indicators
//!
//! # Usage
//!
//! Import components from their respective submodules:
//!
//! ```rust,ignore
//! use tss_gui::component::layout::{SplitView, PageHeader, sidebar};
//! use tss_gui::component::panels::{DetailPanel, SearchFilterBar};
//! use tss_gui::component::display::{DomainCard, status_badge, EmptyState};
//! use tss_gui::component::inputs::{TextField, search_box};
//! use tss_gui::component::feedback::{modal, ProgressBar};
//! ```

/// Layout components (split views, page structure, navigation)
pub mod layout;

/// Panel components (list panels, detail panels, sections)
pub mod panels;

/// Display components (badges, cards, tables, list items)
pub mod display;

/// Input components (search, text fields, forms)
pub mod inputs;

/// Feedback components (toasts, modals, progress)
pub mod feedback;

// Icon font bytes for convenience
pub use iced_fonts::LUCIDE_FONT_BYTES;
