//! Placeholder types for future features.
//!
//! These are reserved fields in the project file format to enable
//! forward-compatible schema evolution.

use rkyv::{Archive, Deserialize, Serialize};

/// Placeholders for future features.
///
/// Adding new optional fields here allows future versions to store
/// additional data without breaking older project files.
#[derive(Debug, Clone, Default, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct ProjectPlaceholders {
    /// CO (Comments) domain data.
    pub co_domain: Option<String>,

    /// RELREC (Related Records) domain data.
    pub relrec_domain: Option<String>,

    /// RELSPEC (Related Specimens) domain data.
    pub relspec_domain: Option<String>,

    /// RELSUB (Related Subjects) domain data.
    pub relsub_domain: Option<String>,

    /// aCRF generation parameters.
    pub acrf_params: Option<String>,

    /// cSDRG generation parameters.
    pub csdrg_params: Option<String>,

    /// Sponsor information.
    pub sponsor: Option<String>,

    /// Protocol information.
    pub protocol: Option<String>,

    /// Export history/audit trail.
    pub export_history: Option<String>,

    /// User-defined metadata (JSON blob).
    pub custom_metadata: Option<String>,
}
