//! Layout components for page structure.
//!
//! This module contains components for structuring page layouts:
//!
//! - **SplitView**: Flexible master-detail split layout with builder pattern
//! - **PageHeader**: Headers for pages with title, subtitle, and actions
//! - **Sidebar**: Navigation sidebar with items
//! - **TabBar**: Tab navigation bar
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::layout::{SplitView, PageHeader, sidebar};
//!
//! // Simple split view
//! SplitView::new(list_content, detail_content)
//!     .master_width(280.0)
//!     .view();
//!
//! // Page header
//! PageHeader::new("Settings")
//!     .subtitle("Configure application preferences")
//!     .view();
//!
//! // Sidebar navigation
//! sidebar(&items, selected_idx, |idx| Msg::Navigate(idx));
//! ```

mod page_header;
mod sidebar;
mod split_view;
mod tab_bar;

pub use page_header::{PageHeader, page_header_simple};
pub use sidebar::{SidebarItem, sidebar};
pub use split_view::{PanelScroll, SplitView, split_view, split_view_with_header};
pub use tab_bar::{Tab, tab_bar};
