//! Message handlers organized by category.
//!
//! Each handler module contains functions to process specific message types:
//! - `home` - Home view messages
//! - `domain_editor` - Domain editor tab navigation
//! - `mapping` - Variable mapping messages
//! - `normalization` - Normalization rule messages
//! - `preview` - Preview pagination messages
//! - `validation` - Validation refresh/filter messages
//! - `supp` - SUPP qualifier editing messages
//! - `export` - Export flow messages
//! - `dialog` - Dialog (about, settings, etc.) messages
//! - `menu` - Menu action messages
//! - `keyboard` - Keyboard shortcut messages

mod dialog;
mod domain_editor;
mod export;
mod home;
mod keyboard;
mod mapping;
mod menu;
mod normalization;
mod preview;
mod supp;
mod validation;

// Re-exports will be added as handlers are implemented
