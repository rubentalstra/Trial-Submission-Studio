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

use crate::component::display::{EmptyState, MetadataCard, VariableListItem};
use crate::component::layout::SplitView;
use crate::component::panels::DetailHeader;
use crate::message::domain_editor::NormalizationMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, NormalizationUiState, SourceDomainState, ViewState};
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, MASTER_WIDTH, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
};
use crate::view::domain_editor::detail_no_selection;

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
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return EmptyState::new(
                container(lucide::circle_alert().size(48)).style(|theme: &Theme| {
                    container::Style {
                        text_color: Some(theme.clinical().text_disabled),
                        ..Default::default()
                    }
                }),
                "Domain not found",
            )
            .centered()
            .view();
        }
    };

    // Normalization only applies to source domains
    let source = match domain.as_source() {
        Some(s) => s,
        None => {
            return EmptyState::new(
                container(lucide::info().size(48)).style(|theme: &Theme| container::Style {
                    text_color: Some(theme.clinical().text_muted),
                    ..Default::default()
                }),
                "Generated domains do not require normalization",
            )
            .centered()
            .view();
        }
    };

    let normalization_ui = match &state.view {
        ViewState::DomainEditor(editor) => &editor.normalization_ui,
        _ => return text("Invalid view state").into(),
    };

    let normalization = &source.normalization;
    let sdtm_domain = source.mapping.domain();

    let master_header = view_rules_header(normalization.rules.len(), &normalization.rules);
    let master_content = view_rules_list(source, &normalization.rules, normalization_ui);
    let detail = if let Some(selected_idx) = normalization_ui.selected_rule {
        if let Some(rule) = normalization.rules.get(selected_idx) {
            view_rule_detail(source, rule, sdtm_domain, state.terminology.as_ref())
        } else {
            detail_no_selection(
                lucide::wand_sparkles().size(48),
                "Select a Rule",
                "Click a variable from the list to view its normalization details",
            )
        }
    } else {
        detail_no_selection(
            lucide::wand_sparkles().size(48),
            "Select a Rule",
            "Click a variable from the list to view its normalization details",
        )
    };

    SplitView::new(master_content, detail)
        .master_width(MASTER_WIDTH)
        .master_header(master_header)
        .view()
}

// =============================================================================
// MASTER PANEL
// =============================================================================

fn view_rules_header<'a>(
    total_rules: usize,
    rules: &[tss_submit::NormalizationRule],
) -> Element<'a, Message> {
    let title = text("Normalization Rules")
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

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
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
        Space::new().width(SPACING_SM),
        text("•").size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_disabled),
        }),
        Space::new().width(SPACING_SM),
        text(format!("{} auto", auto_count))
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().width(4.0),
        text(format!("{} transform", transform_count))
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().width(4.0),
        text(format!("{} CT", ct_count))
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
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
    domain: &'a SourceDomainState,
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
    let dot_color = get_status_dot_color(var_status);

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

/// Get the status dot color based on variable status.
/// This function returns a static Color that works across all themes.
fn get_status_dot_color(var_status: VariableStatus) -> Color {
    match var_status {
        VariableStatus::Accepted | VariableStatus::Suggested => {
            // Success green
            Color::from_rgb(0.20, 0.78, 0.35)
        }
        VariableStatus::AutoGenerated => {
            // Primary blue
            Color::from_rgb(0.13, 0.53, 0.90)
        }
        _ => {
            // Border default gray
            Color::from_rgb(0.75, 0.75, 0.78)
        }
    }
}

// =============================================================================
// DETAIL PANEL
// =============================================================================

fn view_rule_detail<'a>(
    domain: &'a SourceDomainState,
    rule: &'a tss_submit::NormalizationRule,
    sdtm_domain: &'a tss_standards::SdtmDomain,
    terminology: Option<&'a TerminologyRegistry>,
) -> Element<'a, Message> {
    let variable = sdtm_domain
        .variables
        .iter()
        .find(|v| v.name == rule.target_variable);
    let icon_color = get_transform_color(&rule.transform_type);
    let type_label = get_transform_label(&rule.transform_type);

    // Build badge icon - wrap icon in container for text_color styling
    let badge_icon: Element<'a, Message> = match &rule.transform_type {
        NormalizationType::Constant => container(lucide::hash().size(12))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            })
            .into(),
        NormalizationType::UsubjidPrefix => container(lucide::user().size(12))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            })
            .into(),
        NormalizationType::SequenceNumber => container(lucide::list_ordered().size(12))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            })
            .into(),
        NormalizationType::Iso8601DateTime | NormalizationType::Iso8601Date => {
            container(lucide::calendar().size(12))
                .style(|theme: &Theme| container::Style {
                    text_color: Some(theme.clinical().text_on_accent),
                    ..Default::default()
                })
                .into()
        }
        NormalizationType::Iso8601Duration => container(lucide::timer().size(12))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            })
            .into(),
        NormalizationType::StudyDay { .. } => container(lucide::calendar_days().size(12))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            })
            .into(),
        NormalizationType::CtNormalization { .. } => container(lucide::list().size(12))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            })
            .into(),
        NormalizationType::NumericConversion => container(lucide::calculator().size(12))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            })
            .into(),
        NormalizationType::CopyDirect => container(lucide::copy().size(12))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            })
            .into(),
        _ => container(lucide::wand_sparkles().size(12))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            })
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
    let type_str = match var.data_type {
        tss_standards::VariableType::Char => "Character",
        tss_standards::VariableType::Num => "Numeric",
    };

    let title_row = row![
        container(lucide::info().size(14)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().primary.base.color),
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text("Variable Information")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
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
        container(lucide::wand_sparkles().size(12)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().primary.base.color),
            ..Default::default()
        }),
        Space::new().width(SPACING_XS),
        text("Transformation")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
    ]
    .align_y(Alignment::Center);

    let explanation = get_transform_explanation(&rule.transform_type);
    let description = rule.description.clone();

    column![
        title_row,
        Space::new().height(SPACING_SM),
        card.view(),
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
        transform_title,
        Space::new().height(SPACING_XS),
        text(description)
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
        text(explanation)
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
    ]
    .into()
}

fn view_transformation_only<'a>(rule: &'a tss_submit::NormalizationRule) -> Element<'a, Message> {
    let title_row = row![
        container(lucide::wand_sparkles().size(14)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().primary.base.color),
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text("Transformation")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
    ]
    .align_y(Alignment::Center);

    let explanation = get_transform_explanation(&rule.transform_type);
    let description = rule.description.clone();

    column![
        title_row,
        Space::new().height(SPACING_SM),
        container(column![
            text(description)
                .size(13)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_secondary),
                }),
            Space::new().height(SPACING_SM),
            text(explanation)
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
        ])
        .padding(SPACING_MD)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
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
    domain: &'a SourceDomainState,
    rule: &'a tss_submit::NormalizationRule,
    terminology: Option<&'a TerminologyRegistry>,
) -> Element<'a, Message> {
    let title_row = row![
        container(lucide::split().size(14)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().primary.base.color),
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text("Before / After")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
    ]
    .align_y(Alignment::Center);

    let preview_content = build_preview_content(domain, rule, terminology);

    column![
        title_row,
        Space::new().height(SPACING_SM),
        container(preview_content)
            .padding(SPACING_MD)
            .width(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.clinical().background_secondary.into()),
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
    domain: &'a SourceDomainState,
    rule: &'a tss_submit::NormalizationRule,
    terminology: Option<&'a TerminologyRegistry>,
) -> Element<'a, Message> {
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
                text("Generated Value")
                    .size(11)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted),
                    }),
                Space::new().height(SPACING_XS),
                preview_value_box(value),
            ]
            .into()
        }
        NormalizationType::SequenceNumber => column![
            text("Generated Sequence")
                .size(11)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
            Space::new().height(SPACING_XS),
            row![
                preview_value_box("1"),
                Space::new().width(SPACING_XS),
                preview_value_box("2"),
                Space::new().width(SPACING_XS),
                preview_value_box("3"),
                Space::new().width(SPACING_XS),
                text("...").size(12).style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
            ]
            .align_y(Alignment::Center),
            Space::new().height(SPACING_XS),
            text("Unique per subject within domain")
                .size(11)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
        ]
        .into(),
        NormalizationType::UsubjidPrefix => {
            let study_id = domain.mapping.domain().name.clone();
            let study_id_clone = study_id.clone();
            column![
                row![
                    column![
                        text("STUDYID").size(10).style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_muted),
                        }),
                        preview_value_box(study_id)
                    ],
                    Space::new().width(SPACING_SM),
                    text("+").size(16).style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_disabled),
                    }),
                    Space::new().width(SPACING_SM),
                    column![
                        text("SUBJID").size(10).style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_muted),
                        }),
                        preview_value_box("001")
                    ],
                    Space::new().width(SPACING_SM),
                    text("→").size(16).style(|theme: &Theme| text::Style {
                        color: Some(theme.extended_palette().primary.base.color),
                    }),
                    Space::new().width(SPACING_SM),
                    column![
                        text("USUBJID").size(10).style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_muted),
                        }),
                        preview_value_box(format!("{}-001", study_id_clone))
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
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_muted),
                        })
                        .into()
                } else {
                    let mut table_rows = column![].spacing(2.0);
                    table_rows = table_rows.push(
                        row![
                            text("Before")
                                .size(10)
                                .style(|theme: &Theme| text::Style {
                                    color: Some(theme.clinical().text_muted),
                                })
                                .width(Length::FillPortion(1)),
                            Space::new().width(SPACING_MD),
                            text("After")
                                .size(10)
                                .style(|theme: &Theme| text::Style {
                                    color: Some(theme.clinical().text_muted),
                                })
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
                        container(lucide::circle_alert().size(14)).style(|theme: &Theme| {
                            container::Style {
                                text_color: Some(theme.extended_palette().warning.base.color),
                                ..Default::default()
                            }
                        }),
                        Space::new().width(SPACING_SM),
                        text("Source column not mapped")
                            .size(12)
                            .style(|theme: &Theme| text::Style {
                                color: Some(theme.clinical().text_secondary),
                            }),
                    ]
                    .align_y(Alignment::Center),
                    Space::new().height(SPACING_XS),
                    text("Map this variable in the Mapping tab to see preview")
                        .size(11)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_muted),
                        }),
                ]
                .into()
            }
        }
    }
}

fn preview_value_box<'a>(value: impl Into<String>) -> Element<'a, Message> {
    let value_str = value.into();
    container(text(value_str).size(12).style(|theme: &Theme| text::Style {
        color: Some(theme.extended_palette().background.base.text),
    }))
    .padding([SPACING_XS as u16, SPACING_SM as u16])
    .style(|theme: &Theme| container::Style {
        background: Some(theme.clinical().background_elevated.into()),
        border: Border {
            color: theme.clinical().border_default,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    })
    .into()
}

fn view_preview_row<'a>(before: String, after: String) -> Element<'a, Message> {
    row![
        container(text(before).size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        }))
        .padding([SPACING_XS as u16, SPACING_SM as u16])
        .width(Length::FillPortion(1))
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_elevated.into()),
            border: Border {
                color: theme.clinical().border_default,
                width: 1.0,
                radius: BORDER_RADIUS_SM.into()
            },
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text("→").size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_disabled),
        }),
        Space::new().width(SPACING_SM),
        container(text(after).size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        }))
        .padding([SPACING_XS as u16, SPACING_SM as u16])
        .width(Length::FillPortion(1))
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_elevated.into()),
            border: Border {
                color: theme.clinical().border_default,
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
// HELPERS
// =============================================================================

fn get_transform_color(transform_type: &NormalizationType) -> Color {
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
