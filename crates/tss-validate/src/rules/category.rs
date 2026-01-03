//! P21 rule categories from CSV.

use serde::{Deserialize, Serialize};

/// Validation rule category (parsed from CSV).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Category {
    Terminology,
    Presence,
    Format,
    Consistency,
    Limit,
    Metadata,
    CrossReference,
    Structure,
    #[default]
    Unknown,
}

impl Category {
    /// Get all validation categories.
    pub const fn all() -> &'static [Self] {
        &[
            Self::Terminology,
            Self::Presence,
            Self::Format,
            Self::Consistency,
            Self::Limit,
            Self::Metadata,
            Self::CrossReference,
            Self::Structure,
        ]
    }

    /// Parse category from CSV string.
    pub fn parse(s: &str) -> Self {
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

    /// Display name for UI.
    pub fn display_name(&self) -> &'static str {
        self.label()
    }

    /// Description for UI.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Terminology => "Controlled terminology and codelist checks",
            Self::Presence => "Required and expected variable checks",
            Self::Format => "Data format and type validation",
            Self::Consistency => "Cross-variable consistency checks",
            Self::Limit => "Value length and range limits",
            Self::Metadata => "Metadata completeness checks",
            Self::CrossReference => "Cross-domain reference checks",
            Self::Structure => "Dataset structure validation",
            Self::Unknown => "Uncategorized rules",
        }
    }
}
