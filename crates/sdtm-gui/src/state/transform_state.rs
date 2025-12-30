//! Transform configuration state
//!
//! Uses the transformation pipeline from sdtm-transform crate.
//! Provides GUI-specific display helpers on top of the core types.

// Re-export core types from sdtm-transform
pub use sdtm_transform::{
    DomainPipeline, TransformRule, TransformType, build_pipeline_from_domain,
};

/// GUI-specific display helpers for TransformType
pub trait TransformTypeDisplay {
    /// Returns the egui_phosphor icon for this transform type
    fn icon(&self) -> &'static str;

    /// Returns a human-readable category name
    fn category(&self) -> &'static str;

    /// Whether this is an auto-generated transform (not CT normalization)
    fn is_generated(&self) -> bool;
}

impl TransformTypeDisplay for TransformType {
    fn icon(&self) -> &'static str {
        match self {
            Self::CopyDirect => egui_phosphor::regular::COPY,
            Self::Constant => egui_phosphor::regular::LOCK,
            Self::UsubjidPrefix => egui_phosphor::regular::USER,
            Self::SequenceNumber => egui_phosphor::regular::HASH,
            Self::CtNormalization { .. } => egui_phosphor::regular::LIST_CHECKS,
            Self::Iso8601DateTime | Self::Iso8601Date => egui_phosphor::regular::CALENDAR,
            Self::Iso8601Duration => egui_phosphor::regular::TIMER,
            Self::StudyDay { .. } => egui_phosphor::regular::CALENDAR_CHECK,
            Self::NumericConversion => egui_phosphor::regular::FUNCTION,
            // Handle future variants
            _ => egui_phosphor::regular::QUESTION,
        }
    }

    fn category(&self) -> &'static str {
        match self {
            Self::CopyDirect => "Copy",
            Self::Constant => "Constant",
            Self::UsubjidPrefix => "Derived",
            Self::SequenceNumber => "Sequence",
            Self::CtNormalization { .. } => "CT Normalization",
            Self::Iso8601DateTime => "ISO 8601 DateTime",
            Self::Iso8601Date => "ISO 8601 Date",
            Self::Iso8601Duration => "ISO 8601 Duration",
            Self::StudyDay { .. } => "Study Day",
            Self::NumericConversion => "Numeric",
            // Handle future variants
            _ => "Unknown",
        }
    }

    fn is_generated(&self) -> bool {
        // Auto-generated transforms (not CT normalization or direct copy)
        !matches!(self, Self::CtNormalization { .. } | Self::CopyDirect)
    }
}

/// GUI-specific display helpers for TransformRule
pub trait TransformRuleDisplay {
    /// Returns the egui_phosphor icon for this rule's transform type
    fn icon(&self) -> &'static str;

    /// Returns a human-readable category name
    fn category(&self) -> &'static str;

    /// Whether this is an auto-generated transform
    fn is_generated(&self) -> bool;
}

impl TransformRuleDisplay for TransformRule {
    fn icon(&self) -> &'static str {
        self.transform_type.icon()
    }

    fn category(&self) -> &'static str {
        self.transform_type.category()
    }

    fn is_generated(&self) -> bool {
        self.transform_type.is_generated()
    }
}

/// State for transform display in the GUI
#[derive(Debug, Clone, Default)]
pub struct TransformState {
    /// The transformation pipeline for this domain
    pub pipeline: Option<DomainPipeline>,
    /// Selected rule index (for detail view)
    pub selected_idx: Option<usize>,
}

impl TransformState {
    /// Create a new transform state from a pipeline
    pub fn new(pipeline: DomainPipeline) -> Self {
        Self {
            pipeline: Some(pipeline),
            selected_idx: None,
        }
    }

    /// Get the transformation rules
    pub fn rules(&self) -> &[TransformRule] {
        self.pipeline
            .as_ref()
            .map(|p| p.rules.as_slice())
            .unwrap_or(&[])
    }

    /// Count of auto-generated transforms
    pub fn generated_count(&self) -> usize {
        self.rules().iter().filter(|r| r.is_generated()).count()
    }

    /// Count of CT normalization transforms
    pub fn ct_count(&self) -> usize {
        self.rules()
            .iter()
            .filter(|r| matches!(r.transform_type, TransformType::CtNormalization { .. }))
            .count()
    }
}
