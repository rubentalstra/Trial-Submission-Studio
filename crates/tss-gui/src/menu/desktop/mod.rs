//! Desktop (Windows + Linux) in-app menu bar implementation.
//!
//! This module provides:
//! - Custom Iced-rendered menu bar matching the app style
//! - Dropdown state management
//! - Reusable menu components

mod components;
mod menu_bar;
mod state;

pub use components::{
    view_dropdown_container, view_menu_item, view_menu_item_disabled, view_menu_label,
    view_separator,
};
pub use menu_bar::view_menu_bar;
pub use state::{DropdownId, MenuDropdownState};
