//! View components
//!
//! Each view represents a major screen in the application.

mod about_dialog;
mod domain_editor;
mod export;
mod home;
mod update_dialog;

pub use about_dialog::show_about_dialog;
pub use domain_editor::DomainEditorView;
pub use egui_commonmark::CommonMarkCache;
pub use export::ExportView;
pub use home::{HomeAction, HomeView};
pub use update_dialog::{UpdateDialogAction, show_update_dialog};
