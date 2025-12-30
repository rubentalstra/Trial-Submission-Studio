//! View components
//!
//! Each view represents a major screen in the application.

mod home;
mod domain_editor;
mod export;

pub use home::HomeView;
pub use domain_editor::DomainEditorView;
pub use export::ExportView;
