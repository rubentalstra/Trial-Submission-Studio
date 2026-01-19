//! macOS native menu bar implementation using the `muda` crate.
//!
//! This module provides:
//! - Native NSMenu via muda with proper lifetime management
//! - Polling-based event handling (50ms interval)
//! - Dynamic recent studies submenu updates

mod menu_bar;
mod recent_studies;
mod subscription;

pub use menu_bar::create_menu;
pub use recent_studies::{RecentStudyInfo, update_recent_studies_menu};
pub use subscription::menu_subscription;
