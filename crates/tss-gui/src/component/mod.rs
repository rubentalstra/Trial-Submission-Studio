//! Reusable UI components for Trial Submission Studio.
//!
//! This module provides building blocks for constructing views:
//!
//! - **Layout**: `master_detail`, `sidebar`, `tab_bar`, `MasterPanelHeader`
//! - **Overlays**: `modal`, `progress_modal`
//! - **Form**: `search_box`, `form_field`, `TextField`
//! - **Display**: `status_badge`, `data_table`, `MetadataCard`, `StatusCard`
//! - **Feedback**: `EmptyState`, `LoadingState`, `ErrorState`
//! - **List Items**: `VariableListItem`, `SimpleListItem`, `SelectableRow`
//! - **Actions**: `ActionButton`, `ActionButtonList`
//! - **Icons**: Use `iced_fonts::lucide::*` directly (see <https://lucide.dev/icons/>)
//!
//! # Design Philosophy
//!
//! Components use the builder pattern and return `Element<M>`.
//! This provides:
//! - Simple composition
//! - Type-safe message passing
//! - Easy customization per use case
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::{
//!     master_detail_with_pinned_header, SearchFilterBar, VariableListItem,
//!     DetailHeader, MetadataCard, StatusCard,
//! };
//!
//! // Search + filter bar
//! let header = SearchFilterBar::new(&search, "Search...", |s| Msg::Search(s))
//!     .filter("Unmapped", unmapped, Msg::Toggle)
//!     .stats("15/20 mapped")
//!     .view();
//!
//! // Variable list item
//! let item = VariableListItem::new("STUDYID", Msg::Select(0))
//!     .label("Study Identifier")
//!     .leading_icon(lucide::circle_check().size(12).color(SUCCESS))
//!     .trailing_badge("Req", ERROR)
//!     .selected(true)
//!     .view();
//! ```

// Allow unused exports - these are public API items that may not be used internally
#![allow(unused_imports)]
#![allow(dead_code)]

// Core modules
mod core_badge;
mod data_table;
mod domain_badge;
mod empty_state;
mod form_field;
mod master_detail;
mod modal;
mod page_header;
mod progress_modal;
mod search_box;
mod section_card;
mod selectable_row;
mod sidebar;
mod status_badge;
mod tab_bar;

// New comprehensive layout components
mod action_button;
mod detail_header;
mod domain_card;
mod master_panel;
mod metadata_card;
mod progress_bar;
mod search_filter_bar;
mod status_card;
mod text_field;
mod variable_list_item;

// Layout components
pub use master_detail::{
    master_detail, master_detail_with_header, master_detail_with_pinned_header,
};
pub use sidebar::{SidebarItem, sidebar};
pub use tab_bar::{Tab, tab_bar};

// Master panel components
pub use master_panel::{MasterPanelHeader, MasterPanelSection, master_panel_empty};

// Search and filter components
pub use search_filter_bar::{FilterToggle, SearchFilterBar};

// Overlay components
pub use modal::{confirm_modal, modal};
pub use progress_modal::progress_modal;

// Form components
pub use form_field::{form_field, number_field};
pub use search_box::search_box;
pub use text_field::{TextAreaField, TextField};

// Display components
pub use data_table::{TableColumn, data_table};
pub use status_badge::{Status, status_badge};

// Metadata display components
pub use metadata_card::{MetadataCard, metadata_row, metadata_row_wide};

// Status card components
pub use status_card::{
    StatusCard, status_card_neutral, status_card_success, status_card_unmapped, status_card_warning,
};

// Feedback components (builder pattern)
pub use empty_state::{EmptyState, ErrorState, LoadingState, NoFilteredResults};

// Section components
pub use section_card::{SectionCard, panel, status_panel};

// Badge components
pub use core_badge::{core_badge, core_badge_if_important};
pub use domain_badge::{domain_badge, domain_badge_small};

// Progress components
pub use progress_bar::ProgressBar;

// Domain card component (for home view)
pub use domain_card::DomainCard;

// List item components
pub use selectable_row::{DomainListItem, SelectableRow};
pub use variable_list_item::{SimpleListItem, VariableListItem};

// Header components
pub use detail_header::DetailHeader;
pub use page_header::{PageHeader, page_header_simple};

// Action button components
pub use action_button::{ActionButton, ActionButtonList, ActionButtonStyle, edit_buttons};

// Icons: Use iced_fonts::lucide directly
// Re-export font bytes for convenience (load in main.rs)
pub use iced_fonts::LUCIDE_FONT_BYTES;
