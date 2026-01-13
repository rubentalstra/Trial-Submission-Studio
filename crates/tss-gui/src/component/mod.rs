//! Reusable UI components for Trial Submission Studio.
//!
//! This module provides building blocks for constructing views:
//!
//! - **Layout**: `master_detail`, `sidebar`, `tab_bar`
//! - **Overlays**: `modal`, `progress_modal`
//! - **Form**: `search_box`, `form_field`
//! - **Display**: `status_badge`, `data_table`
//! - **Helpers**: `icon`
//!
//! # Design Philosophy
//!
//! Components are functions that return `Element<M>`, not custom widgets.
//! This provides:
//! - Simple composition
//! - Type-safe message passing
//! - Easy customization per use case
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::{master_detail, search_box, status_badge, Status};
//!
//! let view = master_detail(
//!     search_box(&state.search, "Search...", Message::SearchChanged, Message::SearchCleared),
//!     detail_panel,
//!     280.0,
//! );
//! ```

mod data_table;
mod form_field;
mod icon;
mod master_detail;
mod modal;
mod progress_modal;
mod search_box;
mod sidebar;
mod status_badge;
mod tab_bar;

// Layout components
pub use master_detail::{master_detail, master_detail_with_header};
pub use sidebar::{SidebarItem, sidebar};
pub use tab_bar::{Tab, tab_bar};

// Overlay components
pub use modal::{confirm_modal, modal};
pub use progress_modal::progress_modal;

// Form components
pub use form_field::{form_field, number_field};
pub use search_box::search_box;

// Display components
pub use data_table::{TableColumn, data_table};
pub use status_badge::{Status, status_badge};

// Helper components
pub use icon::{icon, icon_check, icon_error, icon_file, icon_folder, icon_search, icon_warning};
