//! Panel components for content organization.
//!
//! This module contains components for organizing content within panels:
//!
//! - **ListPanel**: Comprehensive list panel with search, filters, and stats
//! - **ScrollableListPanel**: ListPanel wrapped in a scrollable container
//! - **DetailPanel**: Detail view panel with header, sections, and actions
//! - **EmptyDetailView**: Empty state for detail panels
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::panels::{ListPanel, DetailPanel};
//!
//! // Full-featured list panel
//! ListPanel::new()
//!     .title("Variables")
//!     .search(&search, "Search...", |s| Msg::Search(s))
//!     .filter("Unmapped", unmapped, Msg::ToggleUnmapped)
//!     .stats("15/20 mapped")
//!     .items(variable_items)
//!     .view();
//!
//! // Detail panel
//! DetailPanel::new("STUDYID")
//!     .subtitle("Study Identifier")
//!     .section(metadata_card)
//!     .actions(action_buttons)
//!     .view();
//! ```

mod detail_panel;
mod list_panel;

pub use detail_panel::{DetailPanel, EmptyDetailView};
pub use list_panel::{ListPanel, ScrollableListPanel};
