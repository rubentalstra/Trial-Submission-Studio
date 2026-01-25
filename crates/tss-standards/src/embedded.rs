//! Embedded standards data.
//!
//! All CDISC standards are embedded at compile time using `include_str!()`.
//! This eliminates runtime file I/O and path resolution issues.
//!
//! # Structure
//!
//! - SDTM-IG v3.4: Datasets and Variables
//! - ADaM-IG v1.3: DataStructures and Variables
//! - SEND-IG v3.1.1: Datasets and Variables
//! - Controlled Terminology: Multiple versions
//!
//! # Adding New CT Versions
//!
//! See `data/terminology/README.md` for step-by-step instructions.

// =============================================================================
// SDTM-IG v3.4
// =============================================================================

/// SDTM-IG v3.4 Datasets.csv
pub const SDTM_IG_DATASETS: &str = include_str!("../data/sdtm/ig/v3.4/Datasets.csv");

/// SDTM-IG v3.4 Variables.csv
pub const SDTM_IG_VARIABLES: &str = include_str!("../data/sdtm/ig/v3.4/Variables.csv");

// =============================================================================
// ADaM-IG v1.3
// =============================================================================

/// ADaM-IG v1.3 DataStructures.csv
pub const ADAM_IG_DATA_STRUCTURES: &str = include_str!("../data/adam/ig/v1.3/DataStructures.csv");

/// ADaM-IG v1.3 Variables.csv
pub const ADAM_IG_VARIABLES: &str = include_str!("../data/adam/ig/v1.3/Variables.csv");

// =============================================================================
// SEND-IG v3.1.1
// =============================================================================

/// SEND-IG v3.1.1 Datasets.csv
pub const SEND_IG_DATASETS: &str = include_str!("../data/send/ig/v3.1.1/Datasets.csv");

/// SEND-IG v3.1.1 Variables.csv
pub const SEND_IG_VARIABLES: &str = include_str!("../data/send/ig/v3.1.1/Variables.csv");

// =============================================================================
// Controlled Terminology - 2024-03-29 (Default)
// =============================================================================

/// SDTM CT 2024-03-29
pub const CT_2024_03_29_SDTM: &str =
    include_str!("../data/terminology/2024-03-29/SDTM_CT_2024-03-29.csv");

/// ADaM CT 2024-03-29
pub const CT_2024_03_29_ADAM: &str =
    include_str!("../data/terminology/2024-03-29/ADaM_CT_2024-03-29.csv");

/// SEND CT 2024-03-29
pub const CT_2024_03_29_SEND: &str =
    include_str!("../data/terminology/2024-03-29/SEND_CT_2024-03-29.csv");

/// Define-XML CT 2024-03-29
pub const CT_2024_03_29_DEFINE_XML: &str =
    include_str!("../data/terminology/2024-03-29/Define-XML_CT_2024-03-29.csv");

/// Protocol CT 2024-03-29
pub const CT_2024_03_29_PROTOCOL: &str =
    include_str!("../data/terminology/2024-03-29/Protocol_CT_2024-03-29.csv");

/// DDF CT 2024-03-29
pub const CT_2024_03_29_DDF: &str =
    include_str!("../data/terminology/2024-03-29/DDF_CT_2024-03-29.csv");

/// MRCT CT 2024-03-29
pub const CT_2024_03_29_MRCT: &str =
    include_str!("../data/terminology/2024-03-29/MRCT_CT_2024-03-29.csv");

// =============================================================================
// Controlled Terminology - 2025-03-28
// =============================================================================

/// SDTM CT 2025-03-28
pub const CT_2025_03_28_SDTM: &str =
    include_str!("../data/terminology/2025-03-28/SDTM_CT_2025-03-28.csv");

/// ADaM CT 2025-03-28
pub const CT_2025_03_28_ADAM: &str =
    include_str!("../data/terminology/2025-03-28/ADaM_CT_2025-03-28.csv");

/// SEND CT 2025-03-28
pub const CT_2025_03_28_SEND: &str =
    include_str!("../data/terminology/2025-03-28/SEND_CT_2025-03-28.csv");

/// Define-XML CT 2025-03-28
pub const CT_2025_03_28_DEFINE_XML: &str =
    include_str!("../data/terminology/2025-03-28/Define-XML_CT_2025-03-28.csv");

/// Protocol CT 2025-03-28
pub const CT_2025_03_28_PROTOCOL: &str =
    include_str!("../data/terminology/2025-03-28/Protocol_CT_2025-03-28.csv");

/// CDASH CT 2025-03-28
pub const CT_2025_03_28_CDASH: &str =
    include_str!("../data/terminology/2025-03-28/CDASH_CT_2025-03-28.csv");

// =============================================================================
// Controlled Terminology - 2025-09-26 (Latest)
// =============================================================================

/// SDTM CT 2025-09-26
pub const CT_2025_09_26_SDTM: &str =
    include_str!("../data/terminology/2025-09-26/SDTM_CT_2025-09-26.csv");

/// ADaM CT 2025-09-26
pub const CT_2025_09_26_ADAM: &str =
    include_str!("../data/terminology/2025-09-26/ADaM_CT_2025-09-26.csv");

/// SEND CT 2025-09-26
pub const CT_2025_09_26_SEND: &str =
    include_str!("../data/terminology/2025-09-26/SEND_CT_2025-09-26.csv");

/// Define-XML CT 2025-09-26
pub const CT_2025_09_26_DEFINE_XML: &str =
    include_str!("../data/terminology/2025-09-26/Define-XML_CT_2025-09-26.csv");

/// Protocol CT 2025-09-26
pub const CT_2025_09_26_PROTOCOL: &str =
    include_str!("../data/terminology/2025-09-26/Protocol_CT_2025-09-26.csv");

/// DDF CT 2025-09-26
pub const CT_2025_09_26_DDF: &str =
    include_str!("../data/terminology/2025-09-26/DDF_CT_2025-09-26.csv");

/// MRCT CT 2025-09-26
pub const CT_2025_09_26_MRCT: &str =
    include_str!("../data/terminology/2025-09-26/MRCT_CT_2025-09-26.csv");

/// Glossary CT 2025-09-26
pub const CT_2025_09_26_GLOSSARY: &str =
    include_str!("../data/terminology/2025-09-26/Glossary_CT_2025-09-26.csv");

// =============================================================================
// Helper: Get all CT files for a version
// =============================================================================

use crate::ct::CtVersion;

/// Get all CT CSV content for a specific version.
///
/// Returns tuples of (filename, content) for each CT file in the version.
pub fn ct_files_for_version(version: CtVersion) -> Vec<(&'static str, &'static str)> {
    match version {
        CtVersion::V2024_03_29 => vec![
            ("SDTM_CT_2024-03-29.csv", CT_2024_03_29_SDTM),
            ("ADaM_CT_2024-03-29.csv", CT_2024_03_29_ADAM),
            ("SEND_CT_2024-03-29.csv", CT_2024_03_29_SEND),
            ("Define-XML_CT_2024-03-29.csv", CT_2024_03_29_DEFINE_XML),
            ("Protocol_CT_2024-03-29.csv", CT_2024_03_29_PROTOCOL),
            ("DDF_CT_2024-03-29.csv", CT_2024_03_29_DDF),
            ("MRCT_CT_2024-03-29.csv", CT_2024_03_29_MRCT),
        ],
        CtVersion::V2025_03_28 => vec![
            ("SDTM_CT_2025-03-28.csv", CT_2025_03_28_SDTM),
            ("ADaM_CT_2025-03-28.csv", CT_2025_03_28_ADAM),
            ("SEND_CT_2025-03-28.csv", CT_2025_03_28_SEND),
            ("Define-XML_CT_2025-03-28.csv", CT_2025_03_28_DEFINE_XML),
            ("Protocol_CT_2025-03-28.csv", CT_2025_03_28_PROTOCOL),
            ("CDASH_CT_2025-03-28.csv", CT_2025_03_28_CDASH),
        ],
        CtVersion::V2025_09_26 => vec![
            ("SDTM_CT_2025-09-26.csv", CT_2025_09_26_SDTM),
            ("ADaM_CT_2025-09-26.csv", CT_2025_09_26_ADAM),
            ("SEND_CT_2025-09-26.csv", CT_2025_09_26_SEND),
            ("Define-XML_CT_2025-09-26.csv", CT_2025_09_26_DEFINE_XML),
            ("Protocol_CT_2025-09-26.csv", CT_2025_09_26_PROTOCOL),
            ("DDF_CT_2025-09-26.csv", CT_2025_09_26_DDF),
            ("MRCT_CT_2025-09-26.csv", CT_2025_09_26_MRCT),
            ("Glossary_CT_2025-09-26.csv", CT_2025_09_26_GLOSSARY),
        ],
    }
}

/// Get SDTM CT content for a specific version.
pub fn sdtm_ct_for_version(version: CtVersion) -> (&'static str, &'static str) {
    match version {
        CtVersion::V2024_03_29 => ("SDTM_CT_2024-03-29.csv", CT_2024_03_29_SDTM),
        CtVersion::V2025_03_28 => ("SDTM_CT_2025-03-28.csv", CT_2025_03_28_SDTM),
        CtVersion::V2025_09_26 => ("SDTM_CT_2025-09-26.csv", CT_2025_09_26_SDTM),
    }
}
