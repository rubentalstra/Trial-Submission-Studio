//! macOS native menu bar implementation using the `muda` crate.
//!
//! This module provides:
//! - Native NSMenu via muda with proper lifetime management
//! - Channel-based event delivery with background forwarder thread
//! - Dynamic recent studies submenu updates
//!
//! # Architecture
//!
//! Menu events flow through a channel-based system:
//! 1. muda generates MenuEvent when user clicks a menu item
//! 2. Forwarder thread (blocks on muda's receiver) converts to MenuAction
//! 3. MenuAction is sent through mpsc channel
//! 4. Iced subscription polls the channel and dispatches to app

mod channel;
mod menu_bar;
mod recent_studies;
mod subscription;

pub use channel::init_menu_channel;
pub use menu_bar::create_menu;
pub use recent_studies::{RecentStudyInfo, update_recent_studies_menu};
pub use subscription::menu_subscription;
