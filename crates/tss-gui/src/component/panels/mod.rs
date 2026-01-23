//! Panel components for content organization.
//!
//! This module contains components for organizing content within panels:
//!
//! - **ListPanel**: Comprehensive list panel with search, filters, and stats
//! - **ScrollableListPanel**: ListPanel wrapped in a scrollable container
//! - **DetailPanel**: Detail view panel with header, sections, and actions
//! - **EmptyDetailView**: Empty state for detail panels
//! - **DetailHeader**: Header for detail views with title and actions
//! - **MasterPanel**: Master panel components for layouts
//! - **SectionCard**: Card sections for content grouping
//! - **SearchFilterBar**: Search and filter controls
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::panels::{ListPanel, DetailPanel, DetailHeader};
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

mod detail_header;
mod detail_panel;
mod list_panel;
mod master_panel;
mod search_filter_bar;
mod section_card;

pub use detail_header::DetailHeader;
pub use detail_panel::{DetailPanel, EmptyDetailView};
pub use list_panel::{ListPanel, ScrollableListPanel};
pub use master_panel::{MasterPanelHeader, MasterPanelSection, master_panel_empty};
pub use search_filter_bar::{FilterToggle, SearchFilterBar};
pub use section_card::{SectionCard, panel, status_panel};
