//! Export functionality module.
//!
//! This module handles exporting SDTM domains to XPT or Dataset-XML format,
//! with automatic SUPP domain generation and Define-XML.

pub mod gating;
pub mod supp;
pub mod types;
pub mod writer;

// Re-export items used by other modules
pub use gating::{can_export_domain, count_bypassed_issues, get_domain_status};
pub use supp::{count_supp_columns, estimate_supp_row_count};
pub use types::{
    ExportBypasses, ExportConfig, ExportPhase, ExportStep, ExportUiState, ExportUpdate,
};
pub use writer::spawn_export;
