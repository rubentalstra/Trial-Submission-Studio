//! Core designation parsing per SDTMIG v3.4 Section 4.1.5.

use serde::{Deserialize, Serialize};

/// Core designation per SDTMIG v3.4 Section 4.1.5.
///
/// Parsed from the "Core" column in Variables.csv.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CoreDesignation {
    /// Required: Must be included and cannot be null
    Req,
    /// Expected: Should be included, may be null, requires Define-XML comment if not collected
    Exp,
    /// Permissible: Include if collected, omit if not
    Perm,
}

impl CoreDesignation {
    /// Returns the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            CoreDesignation::Req => "Req",
            CoreDesignation::Exp => "Exp",
            CoreDesignation::Perm => "Perm",
        }
    }

    /// Parse a core designation from a string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "REQ" | "REQUIRED" => Some(CoreDesignation::Req),
            "EXP" | "EXPECTED" => Some(CoreDesignation::Exp),
            "PERM" | "PERMISSIBLE" => Some(CoreDesignation::Perm),
            _ => None,
        }
    }

    /// Returns the priority for core designation (higher = more important).
    /// Per SDTMIG v3.4: Required > Expected > Permissible
    pub fn priority(&self) -> u8 {
        match self {
            CoreDesignation::Req => 3,
            CoreDesignation::Exp => 2,
            CoreDesignation::Perm => 1,
        }
    }

    /// Returns true if this is a Required designation.
    pub fn is_required(&self) -> bool {
        matches!(self, CoreDesignation::Req)
    }

    /// Returns true if this is an Expected designation.
    pub fn is_expected(&self) -> bool {
        matches!(self, CoreDesignation::Exp)
    }

    /// Returns true if this is a Permissible designation.
    pub fn is_permissible(&self) -> bool {
        matches!(self, CoreDesignation::Perm)
    }
}
