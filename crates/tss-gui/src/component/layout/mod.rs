//! Layout components for page structure.
//!
//! This module contains components for structuring page layouts:
//!
//! - **SplitView**: Flexible master-detail split layout with builder pattern
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::layout::SplitView;
//!
//! // Simple split view
//! SplitView::new(list_content, detail_content)
//!     .master_width(280.0)
//!     .view();
//!
//! // With pinned header on master panel
//! SplitView::new(list_content, detail_content)
//!     .master_width(300.0)
//!     .master_header(search_bar)
//!     .view();
//! ```

mod split_view;

pub use split_view::{PanelScroll, SplitView, split_view, split_view_with_header};
