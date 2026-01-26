//! Helper functions for the Normalization tab.
//!
//! Contains color and label utilities for transformation types.

use iced::Color;
use tss_submit::NormalizationType;

// =============================================================================
// COLOR HELPERS
// =============================================================================

pub(super) fn get_transform_color(transform_type: &NormalizationType) -> Color {
    match transform_type {
        NormalizationType::Constant => Color::from_rgb(0.50, 0.50, 0.55),
        NormalizationType::UsubjidPrefix | NormalizationType::SequenceNumber => {
            Color::from_rgb(0.13, 0.53, 0.90)
        }
        // Use semantic colors for better accessibility support
        NormalizationType::Iso8601DateTime
        | NormalizationType::Iso8601Date
        | NormalizationType::Iso8601Duration => Color::from_rgb(0.25, 0.55, 0.85),
        NormalizationType::StudyDay { .. } => Color::from_rgb(0.35, 0.65, 0.95),
        NormalizationType::CtNormalization { .. } => Color::from_rgb(0.20, 0.78, 0.35),
        NormalizationType::NumericConversion => Color::from_rgb(0.95, 0.65, 0.15),
        NormalizationType::CopyDirect => Color::from_rgb(0.50, 0.50, 0.55),
        _ => Color::from_rgb(0.50, 0.50, 0.55),
    }
}

// =============================================================================
// LABEL HELPERS
// =============================================================================

pub(super) fn get_transform_label(transform_type: &NormalizationType) -> &'static str {
    match transform_type {
        NormalizationType::Constant => "Constant Value",
        NormalizationType::UsubjidPrefix => "USUBJID Derivation",
        NormalizationType::SequenceNumber => "Sequence Number",
        NormalizationType::Iso8601DateTime => "ISO 8601 DateTime",
        NormalizationType::Iso8601Date => "ISO 8601 Date",
        NormalizationType::Iso8601Duration => "ISO 8601 Duration",
        NormalizationType::StudyDay { .. } => "Study Day Calculation",
        NormalizationType::CtNormalization { .. } => "Controlled Terminology",
        NormalizationType::NumericConversion => "Numeric Conversion",
        NormalizationType::CopyDirect => "Direct Copy",
        _ => "Transform",
    }
}

pub(super) fn get_transform_explanation(transform_type: &NormalizationType) -> &'static str {
    match transform_type {
        NormalizationType::Constant => {
            "This value is set automatically from study configuration (STUDYID) or domain code (DOMAIN)."
        }
        NormalizationType::UsubjidPrefix => {
            "Unique Subject Identifier is derived by combining STUDYID with SUBJID in the format 'STUDYID-SUBJID'."
        }
        NormalizationType::SequenceNumber => {
            "A unique sequence number is generated for each record within a subject (USUBJID) in this domain."
        }
        NormalizationType::Iso8601DateTime => {
            "Date and time values are formatted to ISO 8601 standard (YYYY-MM-DDTHH:MM:SS)."
        }
        NormalizationType::Iso8601Date => {
            "Date values are formatted to ISO 8601 standard (YYYY-MM-DD)."
        }
        NormalizationType::Iso8601Duration => {
            "Duration values are formatted to ISO 8601 standard (PnYnMnDTnHnMnS or PnW)."
        }
        NormalizationType::StudyDay { .. } => {
            "Study day is calculated as the number of days from the reference start date (RFSTDTC from DM)."
        }
        NormalizationType::CtNormalization { .. } => {
            "Values are normalized against CDISC Controlled Terminology."
        }
        NormalizationType::NumericConversion => {
            "Text values are converted to numeric (Float64) format."
        }
        NormalizationType::CopyDirect => {
            "Value is copied directly from the source column without modification."
        }
        _ => "Custom transformation applied to this variable.",
    }
}
