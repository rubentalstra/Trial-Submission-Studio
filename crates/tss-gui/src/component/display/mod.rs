//! Display components for data presentation.
//!
//! This module contains components for displaying data:
//!
//! - **DataTable**: Tabular data display with columns
//! - **StatusBadge**: Status indicators and badges
//! - **EmptyState**: Empty state placeholders with actions
//! - **CoreBadge**: CDISC core designation badges
//! - **DomainBadge**: Domain type indicators
//! - **VariableListItem**: List items for variable display
//! - **DomainCard**: Cards for domain display
//! - **MetadataCard**: Cards for metadata key-value display
//! - **StatusCard**: Cards for status indicators
//! - **SelectableRow**: Selectable list items
//! - **ActionButton**: Action buttons with icons
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::display::{EmptyState, status_badge, VariableListItem, DomainCard};
//!
//! // Empty state with action
//! EmptyState::new(icon, "No items found")
//!     .description("Try a different search")
//!     .action("Clear", Msg::Clear)
//!     .view();
//!
//! // Variable list item
//! VariableListItem::new("STUDYID", Msg::Select)
//!     .label("Study Identifier")
//!     .selected(true)
//!     .view();
//!
//! // Domain card
//! DomainCard::new(&domain)
//!     .on_click(Msg::OpenDomain)
//!     .view();
//! ```

mod action_button;
mod core_badge;
mod data_table;
mod domain_badge;
mod domain_card;
mod empty_state;
mod metadata_card;
mod selectable_row;
mod status_badge;
mod status_card;
mod variable_list_item;

pub use action_button::{ActionButton, ActionButtonList, ActionButtonStyle, edit_buttons};
pub use core_badge::{core_badge, core_badge_if_important};
pub use data_table::{TableColumn, data_table};
pub use domain_badge::{domain_badge, domain_badge_small};
pub use domain_card::DomainCard;
pub use empty_state::{EmptyState, ErrorState, LoadingState, NoFilteredResults};
pub use metadata_card::{MetadataCard, metadata_row, metadata_row_wide};
pub use selectable_row::{DomainListItem, SelectableRow};
pub use status_badge::{
    Status, count_badge, mapping_status_badge, status_badge, status_badge_with_icon, status_dot,
    validation_badge,
};
pub use status_card::{
    StatusCard, status_card_neutral, status_card_success, status_card_unmapped, status_card_warning,
};
pub use variable_list_item::{SimpleListItem, VariableListItem};
