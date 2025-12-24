#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Minimal placeholder for Phase 0. Later this crate will emit JSON/Markdown/HTML reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportHeader {
    pub schema: String,
    pub schema_version: u32,
}
