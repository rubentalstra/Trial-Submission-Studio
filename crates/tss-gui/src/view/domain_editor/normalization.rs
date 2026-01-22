//! Normalization tab view.
//!
//! The normalization tab displays data normalization rules that are automatically
//! inferred from SDTM variable metadata. Users can see what transformations
//! will be applied to each variable during export.
//!
//! - **Left (Master)**: List of variables with their normalization types
//! - **Right (Detail)**: Detailed view of the selected rule

use iced::widget::{Space, column, container, row, rule, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::{
    DetailHeader, EmptyState, MetadataCard, VariableListItem, master_detail_with_pinned_header,
};
use crate::message::domain_editor::NormalizationMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, DomainState, NormalizationUiState, ViewState};
use crate::theme::{
    BORDER_RADIUS_SM, MASTER_WIDTH, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, colors,
};

use tss_standards::TerminologyRegistry;
use tss_submit::NormalizationType;
use tss_submit::VariableStatus;

// =============================================================================
// MAIN VIEW
// =============================================================================

/// Render the normalization tab content using master-detail layout.
pub fn view_normalization_tab<'a>(
    state: &'a AppState,
    domain_code: &'a str,
) -> Element<'a, Message> {
    let c = colors();

    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return EmptyState::new(
                lucide::circle_alert().size(48).color(c.text_disabled),
                "Domain not found",
            )
            .centered()
            .view();
        }
    };

    let normalization_ui = match &state.view {
        ViewState::DomainEditor {
            normalization_ui, ..
        } => normalization_ui,
        _ => return text("Invalid view state").into(),
    };

    let normalization = &domain.normalization;
    let sdtm_domain = domain.mapping.domain();

    let master_header = view_rules_header(normalization.rules.len(), &normalization.rules);
    let master_content = view_rules_list(domain, &normalization.rules, normalization_ui);
    let detail = if let Some(selected_idx) = normalization_ui.selected_rule {
        if let Some(rule) = normalization.rules.get(selected_idx) {
            view_rule_detail(domain, rule, sdtm_domain, state.terminology.as_ref())
        } else {
            view_no_selection()
        }
    } else {
        view_no_selection()
    };

    master_detail_with_pinned_header(master_header, master_content, detail, MASTER_WIDTH)
}

// =============================================================================
// MASTER PANEL
// =============================================================================

fn view_rules_header<'a>(
    total_rules: usize,
    rules: &[tss_submit::NormalizationRule],
) -> Element<'a, Message> {
    let c = colors();

    let title = text("Normalization Rules").size(14).color(c.text_secondary);

    let auto_count = rules
        .iter()
        .filter(|r| {
            matches!(
                r.transform_type,
                NormalizationType::Constant
                    | NormalizationType::UsubjidPrefix
                    | NormalizationType::SequenceNumber
            )
        })
        .count();
    let transform_count = rules
        .iter()
        .filter(|r| {
            matches!(
                r.transform_type,
                NormalizationType::Iso8601DateTime
                    | NormalizationType::Iso8601Date
                    | NormalizationType::Iso8601Duration
                    | NormalizationType::StudyDay { .. }
                    | NormalizationType::NumericConversion
            )
        })
        .count();
    let ct_count = rules
        .iter()
        .filter(|r| matches!(r.transform_type, NormalizationType::CtNormalization { .. }))
        .count();

    let stats = row![
        text(format!("{} rules", total_rules))
            .size(12)
            .color(c.text_secondary),
        Space::new().width(SPACING_SM),
        text("•").size(12).color(c.text_disabled),
        Space::new().width(SPACING_SM),
        text(format!("{} auto", auto_count))
            .size(11)
            .color(c.text_muted),
        Space::new().width(4.0),
        text(format!("{} transform", transform_count))
            .size(11)
            .color(c.text_muted),
        Space::new().width(4.0),
        text(format!("{} CT", ct_count))
            .size(11)
            .color(c.text_muted),
    ]
    .align_y(Alignment::Center);

    column![
        title,
        Space::new().height(SPACING_XS),
        stats,
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
    ]
    .into()
}

fn view_rules_list<'a>(
    domain: &'a DomainState,
    rules: &'a [tss_submit::NormalizationRule],
    ui_state: &'a NormalizationUiState,
) -> Element<'a, Message> {
    let mut items = column![].spacing(2.0);
    for (idx, rule) in rules.iter().enumerate() {
        let is_selected = ui_state.selected_rule == Some(idx);
        let var_status = domain.mapping.status(&rule.target_variable);
        items = items.push(view_rule_row(idx, rule, var_status, is_selected));
    }
    items.into()
}

fn view_rule_row<'a>(
    index: usize,
    rule: &'a tss_submit::NormalizationRule,
    var_status: VariableStatus,
    is_selected: bool,
) -> Element<'a, Message> {
    let c = colors();
    let icon_color = get_transform_color(&rule.transform_type);
    let type_label = get_transform_short_label(&rule.transform_type);

    // Build icon
    let icon: Element<'a, Message> = match &rule.transform_type {
        NormalizationType::Constant => lucide::hash().size(14).color(icon_color).into(),
        NormalizationType::UsubjidPrefix => lucide::user().size(14).color(icon_color).into(),
        NormalizationType::SequenceNumber => {
            lucide::list_ordered().size(14).color(icon_color).into()
        }
        NormalizationType::Iso8601DateTime | NormalizationType::Iso8601Date => {
            lucide::calendar().size(14).color(icon_color).into()
        }
        NormalizationType::Iso8601Duration => lucide::timer().size(14).color(icon_color).into(),
        NormalizationType::StudyDay { .. } => {
            lucide::calendar_days().size(14).color(icon_color).into()
        }
        NormalizationType::CtNormalization { .. } => {
            lucide::list().size(14).color(icon_color).into()
        }
        NormalizationType::NumericConversion => {
            lucide::calculator().size(14).color(icon_color).into()
        }
        NormalizationType::CopyDirect => lucide::copy().size(14).color(icon_color).into(),
        _ => lucide::wand_sparkles().size(14).color(icon_color).into(),
    };

    // Status dot color - use semantic colors
    let dot_color = match var_status {
        VariableStatus::Accepted | VariableStatus::Suggested => c.status_success,
        VariableStatus::AutoGenerated => c.accent_primary,
        _ => c.border_default,
    };

    let mut item = VariableListItem::new(
        &rule.target_variable,
        Message::DomainEditor(DomainEditorMessage::Normalization(
            NormalizationMessage::RuleSelected(index),
        )),
    )
    .leading_icon(icon)
    .label(type_label)
    .selected(is_selected);

    // Add trailing status indicator as text for now
    // (VariableListItem doesn't support arbitrary trailing content, but we can use the badge)
    item = item.trailing_badge("●", dot_color);

    item.view()
}

// =============================================================================
// DETAIL PANEL
// =============================================================================

fn view_rule_detail<'a>(
    domain: &'a DomainState,
    rule: &'a tss_submit::NormalizationRule,
    sdtm_domain: &'a tss_standards::SdtmDomain,
    terminology: Option<&'a TerminologyRegistry>,
) -> Element<'a, Message> {
    let c = colors();

    let variable = sdtm_domain
        .variables
        .iter()
        .find(|v| v.name == rule.target_variable);
    let icon_color = get_transform_color(&rule.transform_type);
    let type_label = get_transform_label(&rule.transform_type);

    // Build badge icon
    let badge_icon: Element<'a, Message> = match &rule.transform_type {
        NormalizationType::Constant => lucide::hash().size(12).color(c.text_on_accent).into(),
        NormalizationType::UsubjidPrefix => lucide::user().size(12).color(c.text_on_accent).into(),
        NormalizationType::SequenceNumber => lucide::list_ordered()
            .size(12)
            .color(c.text_on_accent)
            .into(),
        NormalizationType::Iso8601DateTime | NormalizationType::Iso8601Date => {
            lucide::calendar().size(12).color(c.text_on_accent).into()
        }
        NormalizationType::Iso8601Duration => {
            lucide::timer().size(12).color(c.text_on_accent).into()
        }
        NormalizationType::StudyDay { .. } => lucide::calendar_days()
            .size(12)
            .color(c.text_on_accent)
            .into(),
        NormalizationType::CtNormalization { .. } => {
            lucide::list().size(12).color(c.text_on_accent).into()
        }
        NormalizationType::NumericConversion => {
            lucide::calculator().size(12).color(c.text_on_accent).into()
        }
        NormalizationType::CopyDirect => lucide::copy().size(12).color(c.text_on_accent).into(),
        _ => lucide::wand_sparkles()
            .size(12)
            .color(c.text_on_accent)
            .into(),
    };

    let header = DetailHeader::new(&rule.target_variable)
        .subtitle(
            variable
                .and_then(|v| v.label.as_deref())
                .unwrap_or("No label"),
        )
        .badge(badge_icon, type_label, icon_color)
        .view();

    let metadata_section: Element<'a, Message> = if let Some(var) = variable {
        view_metadata_with_transform(var, rule)
    } else {
        view_transformation_only(rule)
    };

    let preview_section = view_before_after_preview(domain, rule, terminology);

    scrollable(column![
        header,
        Space::new().height(SPACING_MD),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        metadata_section,
        Space::new().height(SPACING_LG),
        preview_section,
        Space::new().height(SPACING_MD),
    ])
    .height(Length::Fill)
    .into()
}

fn view_metadata_with_transform<'a>(
    var: &'a tss_standards::SdtmVariable,
    rule: &'a tss_submit::NormalizationRule,
) -> Element<'a, Message> {
    let c = colors();

    let type_str = match var.data_type {
        tss_standards::VariableType::Char => "Character",
        tss_standards::VariableType::Num => "Numeric",
    };

    let title_row = row![
        lucide::info().size(14).color(c.accent_primary),
        Space::new().width(SPACING_SM),
        text("Variable Information")
            .size(14)
            .color(c.text_secondary),
    ]
    .align_y(Alignment::Center);

    let mut card = MetadataCard::new().row("Type", type_str);

    if let Some(length) = var.length {
        card = card.row("Length", length.to_string());
    }
    if let Some(role) = var.role {
        card = card.row("Role", role.as_str());
    }
    if let Some(core) = var.core {
        card = card.row("Core", core.as_str());
    }
    if let Some(ref ct_code) = var.codelist_code {
        card = card.row("Codelist", ct_code);
    }
    if let Some(ref dvd) = var.described_value_domain {
        card = card.row("Format", dvd);
    }

    let transform_title = row![
        lucide::wand_sparkles().size(12).color(c.accent_primary),
        Space::new().width(SPACING_XS),
        text("Transformation").size(12).color(c.text_secondary),
    ]
    .align_y(Alignment::Center);

    let explanation = get_transform_explanation(&rule.transform_type);

    column![
        title_row,
        Space::new().height(SPACING_SM),
        card.view(),
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
        transform_title,
        Space::new().height(SPACING_XS),
        text(&rule.description).size(12).color(c.text_secondary),
        text(explanation).size(11).color(c.text_muted),
    ]
    .into()
}

fn view_transformation_only<'a>(rule: &'a tss_submit::NormalizationRule) -> Element<'a, Message> {
    let c = colors();

    let title_row = row![
        lucide::wand_sparkles().size(14).color(c.accent_primary),
        Space::new().width(SPACING_SM),
        text("Transformation").size(14).color(c.text_secondary),
    ]
    .align_y(Alignment::Center);

    let explanation = get_transform_explanation(&rule.transform_type);
    let bg_secondary = c.background_secondary;

    column![
        title_row,
        Space::new().height(SPACING_SM),
        container(column![
            text(&rule.description).size(13).color(c.text_secondary),
            Space::new().height(SPACING_SM),
            text(explanation).size(12).color(c.text_muted),
        ])
        .padding(SPACING_MD)
        .width(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(bg_secondary.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    ]
    .into()
}

// =============================================================================
// BEFORE/AFTER PREVIEW
// =============================================================================

fn view_before_after_preview<'a>(
    domain: &'a DomainState,
    rule: &'a tss_submit::NormalizationRule,
    terminology: Option<&'a TerminologyRegistry>,
) -> Element<'a, Message> {
    let c = colors();
    let bg_secondary = c.background_secondary;

    let title_row = row![
        lucide::split().size(14).color(c.accent_primary),
        Space::new().width(SPACING_SM),
        text("Before / After").size(14).color(c.text_secondary),
    ]
    .align_y(Alignment::Center);

    let preview_content = build_preview_content(domain, rule, terminology);

    column![
        title_row,
        Space::new().height(SPACING_SM),
        container(preview_content)
            .padding(SPACING_MD)
            .width(Length::Fill)
            .style(move |_: &Theme| container::Style {
                background: Some(bg_secondary.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .into()
}

fn build_preview_content<'a>(
    domain: &'a DomainState,
    rule: &'a tss_submit::NormalizationRule,
    terminology: Option<&'a TerminologyRegistry>,
) -> Element<'a, Message> {
    let c = colors();

    let source_column = domain
        .mapping
        .accepted(&rule.target_variable)
        .map(|(col, _)| col);

    match &rule.transform_type {
        NormalizationType::Constant => {
            let value = if rule.target_variable == "STUDYID" || rule.target_variable == "DOMAIN" {
                domain.mapping.domain().name.clone()
            } else {
                "—".to_string()
            };
            column![
                text("Generated Value").size(11).color(c.text_muted),
                Space::new().height(SPACING_XS),
                preview_value_box(value),
            ]
            .into()
        }
        NormalizationType::SequenceNumber => column![
            text("Generated Sequence").size(11).color(c.text_muted),
            Space::new().height(SPACING_XS),
            row![
                preview_value_box("1"),
                Space::new().width(SPACING_XS),
                preview_value_box("2"),
                Space::new().width(SPACING_XS),
                preview_value_box("3"),
                Space::new().width(SPACING_XS),
                text("...").size(12).color(c.text_muted),
            ]
            .align_y(Alignment::Center),
            Space::new().height(SPACING_XS),
            text("Unique per subject within domain")
                .size(11)
                .color(c.text_muted),
        ]
        .into(),
        NormalizationType::UsubjidPrefix => {
            let study_id = &domain.mapping.domain().name;
            column![
                row![
                    column![
                        text("STUDYID").size(10).color(c.text_muted),
                        preview_value_box(study_id)
                    ],
                    Space::new().width(SPACING_SM),
                    text("+").size(16).color(c.text_disabled),
                    Space::new().width(SPACING_SM),
                    column![
                        text("SUBJID").size(10).color(c.text_muted),
                        preview_value_box("001")
                    ],
                    Space::new().width(SPACING_SM),
                    text("→").size(16).color(c.accent_primary),
                    Space::new().width(SPACING_SM),
                    column![
                        text("USUBJID").size(10).color(c.text_muted),
                        preview_value_box(format!("{}-001", study_id))
                    ],
                ]
                .align_y(Alignment::End),
            ]
            .into()
        }
        _ => {
            if let Some(col_name) = source_column {
                let samples = get_unique_values(&domain.source.data, col_name, 5);
                if samples.is_empty() {
                    text("No sample data available")
                        .size(12)
                        .color(c.text_muted)
                        .into()
                } else {
                    let mut table_rows = column![].spacing(2.0);
                    table_rows = table_rows.push(
                        row![
                            text("Before")
                                .size(10)
                                .color(c.text_muted)
                                .width(Length::FillPortion(1)),
                            Space::new().width(SPACING_MD),
                            text("After")
                                .size(10)
                                .color(c.text_muted)
                                .width(Length::FillPortion(1)),
                        ]
                        .padding([0, SPACING_SM as u16]),
                    );
                    for sample in samples {
                        let after_value =
                            simulate_transform(&sample, &rule.transform_type, terminology);
                        table_rows = table_rows.push(view_preview_row(sample, after_value));
                    }
                    table_rows.into()
                }
            } else {
                column![
                    row![
                        lucide::circle_alert().size(14).color(c.status_warning),
                        Space::new().width(SPACING_SM),
                        text("Source column not mapped")
                            .size(12)
                            .color(c.text_secondary),
                    ]
                    .align_y(Alignment::Center),
                    Space::new().height(SPACING_XS),
                    text("Map this variable in the Mapping tab to see preview")
                        .size(11)
                        .color(c.text_muted),
                ]
                .into()
            }
        }
    }
}

fn preview_value_box<'a>(value: impl Into<String>) -> Element<'a, Message> {
    let c = colors();
    let text_primary = c.text_primary;
    let bg_elevated = c.background_elevated;
    let border_default = c.border_default;

    container(text(value.into()).size(12).color(text_primary))
        .padding([SPACING_XS as u16, SPACING_SM as u16])
        .style(move |_: &Theme| container::Style {
            background: Some(bg_elevated.into()),
            border: Border {
                color: border_default,
                width: 1.0,
                radius: BORDER_RADIUS_SM.into(),
            },
            ..Default::default()
        })
        .into()
}

fn view_preview_row<'a>(before: String, after: String) -> Element<'a, Message> {
    let c = colors();
    let text_primary = c.text_primary;
    let text_disabled = c.text_disabled;
    let bg_elevated = c.background_elevated;
    let border_default = c.border_default;

    row![
        container(text(before).size(12).color(text_primary))
            .padding([SPACING_XS as u16, SPACING_SM as u16])
            .width(Length::FillPortion(1))
            .style(move |_: &Theme| container::Style {
                background: Some(bg_elevated.into()),
                border: Border {
                    color: border_default,
                    width: 1.0,
                    radius: BORDER_RADIUS_SM.into()
                },
                ..Default::default()
            }),
        Space::new().width(SPACING_SM),
        text("→").size(12).color(text_disabled),
        Space::new().width(SPACING_SM),
        container(text(after).size(12).color(text_primary))
            .padding([SPACING_XS as u16, SPACING_SM as u16])
            .width(Length::FillPortion(1))
            .style(move |_: &Theme| container::Style {
                background: Some(bg_elevated.into()),
                border: Border {
                    color: border_default,
                    width: 1.0,
                    radius: BORDER_RADIUS_SM.into()
                },
                ..Default::default()
            }),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn get_unique_values(df: &polars::prelude::DataFrame, column: &str, limit: usize) -> Vec<String> {
    use polars::prelude::*;
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    if let Ok(col) = df.column(column) {
        let series = col.as_materialized_series();
        for i in 0..series.len().min(1000) {
            if let Ok(Some(v)) = series.get(i).map(Some) {
                let str_val = match v {
                    AnyValue::String(s) => s.to_string(),
                    AnyValue::Int64(n) => n.to_string(),
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    AnyValue::Null => continue,
                    _ => format!("{}", v),
                };
                if !str_val.is_empty() && seen.insert(str_val.clone()) {
                    result.push(str_val);
                    if result.len() >= limit {
                        break;
                    }
                }
            }
        }
    }
    result
}

fn simulate_transform(
    input: &str,
    transform_type: &NormalizationType,
    terminology: Option<&TerminologyRegistry>,
) -> String {
    match transform_type {
        NormalizationType::CopyDirect => input.to_string(),
        NormalizationType::Iso8601DateTime | NormalizationType::Iso8601Date => {
            if input.contains('/') {
                input.replace('/', "-")
            } else {
                input.to_string()
            }
        }
        NormalizationType::NumericConversion => {
            if input.parse::<f64>().is_ok() {
                input.to_string()
            } else {
                "—".to_string()
            }
        }
        NormalizationType::CtNormalization { codelist_code } => {
            if let Some(registry) = terminology
                && let Some(resolved) = registry.resolve(codelist_code, None)
                && let Some(submission_value) = resolved.find_submission_value(input)
            {
                return submission_value.to_string();
            }
            input.to_string()
        }
        _ => input.to_string(),
    }
}

// =============================================================================
// EMPTY STATE
// =============================================================================

fn view_no_selection<'a>() -> Element<'a, Message> {
    let c = colors();

    EmptyState::new(
        lucide::wand_sparkles().size(48).color(c.text_disabled),
        "Select a Rule",
    )
    .description("Click a variable from the list to view its normalization details")
    .centered()
    .view()
}

// =============================================================================
// HELPERS
// =============================================================================

fn get_transform_color(transform_type: &NormalizationType) -> Color {
    let c = colors();

    match transform_type {
        NormalizationType::Constant => c.text_secondary,
        NormalizationType::UsubjidPrefix | NormalizationType::SequenceNumber => c.accent_primary,
        // Use semantic colors for better accessibility support
        NormalizationType::Iso8601DateTime
        | NormalizationType::Iso8601Date
        | NormalizationType::Iso8601Duration => c.status_info,
        NormalizationType::StudyDay { .. } => c.accent_primary_medium,
        NormalizationType::CtNormalization { .. } => c.status_success,
        NormalizationType::NumericConversion => c.status_warning,
        NormalizationType::CopyDirect => c.text_muted,
        _ => c.text_muted,
    }
}

fn get_transform_short_label(transform_type: &NormalizationType) -> &'static str {
    match transform_type {
        NormalizationType::Constant => "Constant",
        NormalizationType::UsubjidPrefix => "USUBJID",
        NormalizationType::SequenceNumber => "Sequence",
        NormalizationType::Iso8601DateTime => "DateTime",
        NormalizationType::Iso8601Date => "Date",
        NormalizationType::Iso8601Duration => "Duration",
        NormalizationType::StudyDay { .. } => "Study Day",
        NormalizationType::CtNormalization { .. } => "CT",
        NormalizationType::NumericConversion => "Numeric",
        NormalizationType::CopyDirect => "Copy",
        _ => "Transform",
    }
}

fn get_transform_label(transform_type: &NormalizationType) -> &'static str {
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

fn get_transform_explanation(transform_type: &NormalizationType) -> &'static str {
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
