//! Transform configuration state
//!
//! Simple types for tracking which SDTM transformations apply.
//! Display data is derived on-the-fly from mapping state, not stored here.

/// A transformation rule that will be applied during export
#[derive(Debug, Clone)]
pub enum TransformRule {
    /// Populate STUDYID from study configuration
    StudyIdConstant,

    /// Populate DOMAIN from the dataset code
    DomainConstant,

    /// Derive USUBJID from STUDYID + subject identifier
    UsubjidDerivation,

    /// Assign sequence numbers for --SEQ column (SDTMIG 4.1.5)
    SequenceNumbers { seq_column: String },

    /// Normalize values against Controlled Terminology
    CtNormalization {
        variable: String,
        codelist_code: String,
    },
}

impl TransformRule {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::StudyIdConstant | Self::DomainConstant => "Constant",
            Self::UsubjidDerivation => "Derived",
            Self::SequenceNumbers { .. } => "Sequence Numbers",
            Self::CtNormalization { .. } => "CT Normalization",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::StudyIdConstant => egui_phosphor::regular::IDENTIFICATION_BADGE,
            Self::DomainConstant => egui_phosphor::regular::TABLE,
            Self::UsubjidDerivation => egui_phosphor::regular::USER,
            Self::SequenceNumbers { .. } => egui_phosphor::regular::HASH,
            Self::CtNormalization { .. } => egui_phosphor::regular::LIST_CHECKS,
        }
    }

    pub fn target_variable(&self) -> &str {
        match self {
            Self::StudyIdConstant => "STUDYID",
            Self::DomainConstant => "DOMAIN",
            Self::UsubjidDerivation => "USUBJID",
            Self::SequenceNumbers { seq_column, .. } => seq_column,
            Self::CtNormalization { variable, .. } => variable,
        }
    }

    pub fn is_generated(&self) -> bool {
        !matches!(self, Self::CtNormalization { .. })
    }
}

/// Minimal state for transform display
#[derive(Debug, Clone, Default)]
pub struct TransformState {
    pub transforms: Vec<TransformRule>,
    pub selected_idx: Option<usize>,
}
