//! Message handlers organized by category.
//!
//! Each handler module contains `impl App` blocks with handler methods:
//! - `home` - Home view messages (open study, recent, close study)
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
mod source_assignment;
mod supp;
mod validation;

// All handlers are implemented as `impl App` blocks in their respective modules.
// No re-exports needed since the methods are defined on the App type directly.
