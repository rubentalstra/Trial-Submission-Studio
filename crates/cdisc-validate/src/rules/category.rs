//! P21 rule categories from CSV.

use serde::{Deserialize, Serialize};

/// Validation rule category (parsed from CSV).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Terminology,
    Presence,
    Format,
    Consistency,
    Limit,
    Metadata,
    CrossReference,
    Structure,
    Unknown,
}

impl Category {
    /// Parse category from CSV string.
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "terminology" => Self::Terminology,
            "presence" => Self::Presence,
            "format" => Self::Format,
            "consistency" => Self::Consistency,
            "limit" => Self::Limit,
            "metadata" => Self::Metadata,
            "cross reference" | "crossreference" | "cross-reference" => Self::CrossReference,
            "structure" => Self::Structure,
            _ => Self::Unknown,
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Terminology => "Terminology",
            Self::Presence => "Presence",
            Self::Format => "Format",
            Self::Consistency => "Consistency",
            Self::Limit => "Limit",
            Self::Metadata => "Metadata",
            Self::CrossReference => "Cross Reference",
            Self::Structure => "Structure",
            Self::Unknown => "Unknown",
        }
    }
}

impl Default for Category {
    fn default() -> Self {
        Self::Unknown
    }
}
