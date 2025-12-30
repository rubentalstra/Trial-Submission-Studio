//! Configuration options for SDTM processing.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Mode for controlled terminology matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CtMatchingMode {
    /// Require exact or synonym matches only.
    Strict,
    /// Allow lenient matching (case-insensitive, ignoring non-alphanumeric).
    #[default]
    Lenient,
}

/// Options for normalization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizationOptions {
    /// Matching strictness (Strict vs Lenient)
    pub matching_mode: CtMatchingMode,
    
    /// Allow mapping invalid non-extensible values to "OTHER" if available.
    /// Default: false (Safety first).
    pub enable_other_fallback: bool,
    
    /// Allow mapping unknown-like values to "UNKNOWN" if available.
    /// Default: true.
    pub enable_unknown_fallback: bool,
    
    /// Custom value mappings (Raw -> Submission Value).
    /// Key: Raw value (normalized to uppercase/trimmed for lookup).
    /// Value: Submission Value.
    pub custom_maps: HashMap<String, String>,
}

impl NormalizationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_other_fallback(mut self, enable: bool) -> Self {
        self.enable_other_fallback = enable;
        self
    }

    pub fn with_custom_map(mut self, map: HashMap<String, String>) -> Self {
        self.custom_maps = map;
        self
    }
}

/// Mode for applying STUDYID prefixes to USUBJID values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsubjidPrefixMode {
    /// Do not add STUDYID prefixes.
    Skip,
    /// Add STUDYID prefixes when missing.
    Prefix,
}

/// Mode for assigning --SEQ values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceAssignmentMode {
    /// Do not assign sequence values.
    Skip,
    /// Assign sequence values when missing or invalid.
    Assign,
}

/// Options controlling SDTM processing behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    /// Add STUDYID prefix to USUBJID values.
    ///
    /// SDTMIG 4.1.2: "USUBJID is a unique subject identifier that is a
    /// concatenation of STUDYID and a subject identifier unique within that study."
    pub usubjid_prefix: UsubjidPrefixMode,

    /// Automatically assign sequence numbers (--SEQ).
    ///
    /// SDTMIG 4.1.5: "The --SEQ variable [...] is a unique number for each record
    /// within a domain for a subject."
    pub sequence_assignment: SequenceAssignmentMode,

    /// Log warnings when values are rewritten/normalized.
    pub warn_on_rewrite: bool,

    /// Normalization options (CT, Date, etc.)
    pub normalization: NormalizationOptions,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            usubjid_prefix: UsubjidPrefixMode::Prefix,
            sequence_assignment: SequenceAssignmentMode::Assign,
            warn_on_rewrite: true,
            normalization: NormalizationOptions::default(),
        }
    }
}

impl ProcessingOptions {
    /// Create options for strict SDTMIG-conformant processing.
    ///
    /// This disables lenient CT matching while preserving documented SDTMIG
    /// derivations (USUBJID prefix and sequence assignment).
    pub fn strict() -> Self {
        Self {
            usubjid_prefix: UsubjidPrefixMode::Prefix,
            sequence_assignment: SequenceAssignmentMode::Assign,
            warn_on_rewrite: true,
            normalization: NormalizationOptions {
                matching_mode: CtMatchingMode::Strict,
                ..Default::default()
            },
        }
    }
}
