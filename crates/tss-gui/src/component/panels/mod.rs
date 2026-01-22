//! Panel components for content organization.
//!
//! This module contains components for organizing content within panels:
//!
//! - **ListPanel**: Comprehensive list panel with search, filters, and stats
//! - **ScrollableListPanel**: ListPanel wrapped in a scrollable container
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::panels::ListPanel;
//!
//! // Full-featured list panel
//! ListPanel::new()
//!     .title("Variables")
//!     .search(&search, "Search...", |s| Msg::Search(s))
//!     .filter("Unmapped", unmapped, Msg::ToggleUnmapped)
//!     .stats("15/20 mapped")
//!     .items(variable_items)
//!     .view();
//! ```

mod list_panel;

pub use list_panel::{ListPanel, ScrollableListPanel};
