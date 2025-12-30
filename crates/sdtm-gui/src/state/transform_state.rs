//! Transform configuration state
//!
//! Simple types for tracking which SDTM transformations apply.
//! Display data is derived on-the-fly from mapping state, not stored here.

/// A transformation that will be applied during export
#[derive(Debug, Clone)]
pub enum AutoTransform {
    /// Add STUDYID prefix to USUBJID (SDTMIG 4.1.2)
    UsUbjIdPrefix,

    /// Assign sequence numbers for --SEQ column (SDTMIG 4.1.5)
    SequenceNumbers { seq_column: String },

    /// Normalize values against Controlled Terminology
    CtNormalization { variable: String, codelist_code: String },
}

impl AutoTransform {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::UsUbjIdPrefix => "USUBJID Prefix",
            Self::SequenceNumbers { .. } => "Sequence Numbers",
            Self::CtNormalization { .. } => "CT Normalization",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::UsUbjIdPrefix => egui_phosphor::regular::USER,
            Self::SequenceNumbers { .. } => egui_phosphor::regular::HASH,
            Self::CtNormalization { .. } => egui_phosphor::regular::LIST_CHECKS,
        }
    }

    pub fn target_variable(&self) -> &str {
        match self {
            Self::UsUbjIdPrefix => "USUBJID",
            Self::SequenceNumbers { seq_column, .. } => seq_column,
            Self::CtNormalization { variable, .. } => variable,
        }
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::UsUbjIdPrefix | Self::SequenceNumbers { .. })
    }
}

/// Minimal state for transform display
#[derive(Debug, Clone, Default)]
pub struct TransformState {
    pub transforms: Vec<AutoTransform>,
    pub selected_idx: Option<usize>,
}
