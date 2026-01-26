//! Shared helper components for the SUPP tab.
//!
//! Contains reusable UI components like sample data display,
//! editable fields, and utility functions.

use iced::widget::{Space, column, container, pick_list, row, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;
use polars::prelude::AnyValue;

use crate::component::inputs::TextField;
use crate::component::panels::DetailHeader;
use crate::message::domain_editor::SuppMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{SourceDomainState, SuppAction, SuppColumnConfig, SuppOrigin};
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, MAX_CHARS_SHORT_LABEL, MAX_CHARS_VARIABLE_NAME, SPACING_MD,
    SPACING_SM, SPACING_XS,
};

// =============================================================================
// SHARED COMPONENTS
// =============================================================================

pub(super) fn build_detail_header(col_name: &str, domain_code: &str) -> Element<'static, Message> {
    let col_display = col_name.to_string();
    let target = format!("SUPP{}", domain_code);

    DetailHeader::new("Configure SUPP Variable")
        .subtitle(format!("Source: {} → {}", col_display, target))
        .view()
}

pub(super) fn build_sample_data(
    source: &SourceDomainState,
    col_name: &str,
) -> Element<'static, Message> {
    let samples = get_sample_values(source, col_name, 5);

    let sample_chips: Vec<Element<'static, Message>> = samples
        .into_iter()
        .map(|s| {
            container(text(s).size(11).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }))
            .padding([4.0, 8.0])
            .style(|theme: &Theme| container::Style {
                background: Some(theme.clinical().background_secondary.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
        })
        .collect();

    let sample_content: Element<'static, Message> = if sample_chips.is_empty() {
        text("No data available")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_disabled),
            })
            .into()
    } else {
        row(sample_chips).spacing(SPACING_XS).wrap().into()
    };

    container(
        column![
            row![
                container(lucide::database().size(14)).style(|theme: &Theme| container::Style {
                    text_color: Some(theme.clinical().text_muted),
                    ..Default::default()
                }),
                Space::new().width(SPACING_SM),
                text("Sample Values")
                    .size(13)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_secondary),
                    }),
            ]
            .align_y(Alignment::Center),
            Space::new().height(SPACING_SM),
            sample_content,
        ]
        .width(Length::Fill),
    )
    .padding(SPACING_MD)
    .style(|theme: &Theme| container::Style {
        background: Some(theme.clinical().background_secondary.into()),
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

pub(super) fn build_editable_fields(
    config: &SuppColumnConfig,
    qnam_conflict_error: Option<String>,
) -> Element<'static, Message> {
    // QNAM field (required) - check empty first, then conflict
    let qnam_error: Option<String> = if config.qnam.trim().is_empty() {
        Some("QNAM is required".to_string())
    } else {
        qnam_conflict_error
    };
    let qnam_field = TextField::new("QNAM", &config.qnam, "Qualifier name (max 8 chars)", |v| {
        Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QnamChanged(
            v.to_uppercase(),
        )))
    })
    .max_length(MAX_CHARS_VARIABLE_NAME)
    .required(true)
    .error(qnam_error)
    .view();

    // QLABEL field (required) - validate empty
    let qlabel_error: Option<String> = if config.qlabel.trim().is_empty() {
        Some("QLABEL is required".to_string())
    } else {
        None
    };
    let qlabel_field = TextField::new(
        "QLABEL",
        &config.qlabel,
        "Describe what this value represents...",
        |v| Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QlabelChanged(v))),
    )
    .max_length(MAX_CHARS_SHORT_LABEL)
    .required(true)
    .error(qlabel_error)
    .view();

    // QORIG picker
    let qorig_field = build_origin_picker(config.qorig);

    // QEVAL field (optional)
    let qeval_str = config.qeval.as_deref().unwrap_or("");
    let qeval_field = TextField::new("QEVAL", qeval_str, "Evaluator (e.g., INVESTIGATOR)", |v| {
        Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QevalChanged(v)))
    })
    .max_length(40)
    .view();

    column![qnam_field, qlabel_field, qorig_field, qeval_field,]
        .spacing(SPACING_MD)
        .into()
}

pub(super) fn build_origin_picker(current: SuppOrigin) -> Element<'static, Message> {
    column![
        text("QORIG").size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        }),
        Space::new().height(4.0),
        pick_list(&SuppOrigin::ALL[..], Some(current), |origin| {
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QorigChanged(origin)))
        })
        .text_size(14)
        .padding([10.0, 12.0])
        .width(Length::Fill),
    ]
    .into()
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

pub(super) fn get_sample_values(
    source: &SourceDomainState,
    col_name: &str,
    max: usize,
) -> Vec<String> {
    let mut samples = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Ok(col) = source.source.data.column(col_name) {
        for i in 0..col.len().min(100) {
            if let Ok(val) = col.get(i) {
                let s = format_value(&val);
                if !s.is_empty() && seen.insert(s.clone()) {
                    samples.push(s);
                    if samples.len() >= max {
                        break;
                    }
                }
            }
        }
    }

    samples
}

fn format_value(value: &AnyValue) -> String {
    match value {
        AnyValue::Null => String::new(),
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        _ => format!("{}", value),
    }
}

pub(super) fn check_qnam_conflict(
    source: &SourceDomainState,
    current_col: &str,
    qnam: &str,
) -> Option<String> {
    if qnam.is_empty() {
        return None;
    }

    // Only check against columns already included in SUPP
    for (col, config) in &source.supp_config {
        if col != current_col
            && config.action == SuppAction::Include
            && config.qnam.eq_ignore_ascii_case(qnam)
        {
            return Some(format!("QNAM '{}' already used by '{}'", qnam, col));
        }
    }

    None
}

// =============================================================================
// DISPLAY IMPLEMENTATIONS
// =============================================================================

impl std::fmt::Display for SuppOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}
