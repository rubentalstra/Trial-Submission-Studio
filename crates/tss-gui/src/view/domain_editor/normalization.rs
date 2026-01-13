//! Normalization tab view.
//!
//! The normalization tab displays data normalization rules that are automatically
//! inferred from SDTM variable metadata. Users can see what transformations
//! will be applied to each variable during export.
//!
//! - **Left (Master)**: List of variables with their normalization types
//! - **Right (Detail)**: Detailed view of the selected rule

use iced::widget::{Space, button, column, container, row, rule, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::master_detail_with_pinned_header;
use crate::message::domain_editor::NormalizationMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, NormalizationUiState, ViewState};
use crate::theme::{
    BORDER_RADIUS_SM, ERROR, GRAY_100, GRAY_300, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800,
    GRAY_900, PRIMARY_100, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, SUCCESS,
    WARNING, WHITE,
};

use tss_map::VariableStatus;
use tss_normalization::NormalizationType;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Width of the master (rules list) panel.
const MASTER_WIDTH: f32 = 340.0;

// =============================================================================
// MAIN NORMALIZATION TAB VIEW
// =============================================================================

/// Render the normalization tab content using master-detail layout.
///
/// The normalization pipeline is pre-computed and stored in domain state,
/// following Iced's Elm architecture (view reads from state, never computes).
pub fn view_normalization_tab<'a>(
    state: &'a AppState,
    domain_code: &'a str,
) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return container(text("Domain not found").size(14).color(GRAY_500))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Shrink)
                .center_y(Length::Shrink)
                .into();
        }
    };

    // Get UI state
    let normalization_ui = match &state.view {
        ViewState::DomainEditor {
            normalization_ui, ..
        } => normalization_ui,
        _ => return text("Invalid view state").into(),
    };

    // Read normalization from state (computed when domain was loaded)
    let normalization = &domain.normalization;
    let sdtm_domain = domain.mapping.domain();

    // Build master panel header
    let master_header = view_rules_header(normalization.rules.len(), &normalization.rules);

    // Build master panel content (rules list)
    let master_content = view_rules_list(domain, &normalization.rules, normalization_ui);

    // Build detail panel
    let detail = if let Some(selected_idx) = normalization_ui.selected_rule {
        if let Some(rule) = normalization.rules.get(selected_idx) {
            view_rule_detail(domain, rule, sdtm_domain)
        } else {
            view_no_selection()
        }
    } else {
        view_no_selection()
    };

    master_detail_with_pinned_header(master_header, master_content, detail, MASTER_WIDTH)
}

// =============================================================================
// MASTER PANEL HEADER
// =============================================================================

/// Header with title and stats for the rules list.
fn view_rules_header<'a>(
    total_rules: usize,
    rules: &[tss_normalization::NormalizationRule],
) -> Element<'a, Message> {
    let title = text("Normalization Rules").size(14).color(GRAY_700);

    // Count rules by category
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
            .color(GRAY_600),
        Space::new().width(SPACING_SM),
        text("•").size(12).color(GRAY_400),
        Space::new().width(SPACING_SM),
        text(format!("{} auto", auto_count))
            .size(11)
            .color(GRAY_500),
        Space::new().width(4.0),
        text(format!("{} transform", transform_count))
            .size(11)
            .color(GRAY_500),
        Space::new().width(4.0),
        text(format!("{} CT", ct_count)).size(11).color(GRAY_500),
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

// =============================================================================
// MASTER PANEL CONTENT (RULES LIST)
// =============================================================================

/// Rules list content.
fn view_rules_list<'a>(
    domain: &'a crate::state::Domain,
    rules: &'a [tss_normalization::NormalizationRule],
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

/// Single rule row in the list.
fn view_rule_row<'a>(
    index: usize,
    rule: &'a tss_normalization::NormalizationRule,
    var_status: VariableStatus,
    is_selected: bool,
) -> Element<'a, Message> {
    // Get icon color for transformation type
    let icon_color = get_transform_color(&rule.transform_type);

    // Status indicator for mapping
    let mapping_indicator = match var_status {
        VariableStatus::Accepted | VariableStatus::Suggested => {
            container(Space::new().width(6.0).height(6.0)).style(move |_theme: &Theme| {
                container::Style {
                    background: Some(SUCCESS.into()),
                    border: Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
        }
        VariableStatus::AutoGenerated => {
            container(Space::new().width(6.0).height(6.0)).style(move |_theme: &Theme| {
                container::Style {
                    background: Some(PRIMARY_500.into()),
                    border: Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
        }
        _ => container(Space::new().width(6.0).height(6.0)).style(move |_theme: &Theme| {
            container::Style {
                background: Some(GRAY_300.into()),
                border: Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }),
    };

    // Transform type label
    let type_label = get_transform_short_label(&rule.transform_type);

    let name = text(&rule.target_variable).size(13).color(GRAY_800);
    let type_text = text(type_label).size(11).color(GRAY_500);

    // Build icon based on transformation type
    let icon_element: Element<'_, Message> = match &rule.transform_type {
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

    let content = row![
        icon_element,
        Space::new().width(SPACING_SM),
        column![name, type_text].width(Length::Fill),
        mapping_indicator,
    ]
    .align_y(Alignment::Center)
    .padding([SPACING_SM, SPACING_SM]);

    button(content)
        .on_press(Message::DomainEditor(DomainEditorMessage::Normalization(
            NormalizationMessage::RuleSelected(index),
        )))
        .width(Length::Fill)
        .style(move |_theme: &Theme, _status| {
            let bg = if is_selected {
                PRIMARY_100
            } else {
                iced::Color::TRANSPARENT
            };
            iced::widget::button::Style {
                background: Some(bg.into()),
                text_color: GRAY_800,
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

// =============================================================================
// DETAIL PANEL
// =============================================================================

/// Detail view for a selected rule.
fn view_rule_detail<'a>(
    domain: &'a crate::state::Domain,
    rule: &'a tss_normalization::NormalizationRule,
    sdtm_domain: &'a tss_model::Domain,
) -> Element<'a, Message> {
    // Find the variable definition
    let variable = sdtm_domain
        .variables
        .iter()
        .find(|v| v.name == rule.target_variable);

    // Header
    let header = view_detail_header(rule, variable);

    // Transformation info
    let transform_info = view_transformation_info(rule);

    // Source mapping info
    let mapping_info = view_mapping_info(domain, rule);

    // Variable metadata (if available)
    let metadata: Element<'a, Message> = if let Some(var) = variable {
        view_variable_metadata(var)
    } else {
        Space::new().height(0.0).into()
    };

    scrollable(column![
        header,
        Space::new().height(SPACING_MD),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        transform_info,
        Space::new().height(SPACING_LG),
        mapping_info,
        Space::new().height(SPACING_LG),
        metadata,
        Space::new().height(SPACING_MD),
    ])
    .height(Length::Fill)
    .into()
}

/// Detail header with variable name and transformation type.
fn view_detail_header<'a>(
    rule: &'a tss_normalization::NormalizationRule,
    variable: Option<&'a tss_model::Variable>,
) -> Element<'a, Message> {
    let name = text(&rule.target_variable).size(20).color(GRAY_900);

    let label = if let Some(var) = variable {
        text(var.label.as_deref().unwrap_or("No label"))
            .size(14)
            .color(GRAY_600)
    } else {
        text("Variable definition not found")
            .size(14)
            .color(GRAY_500)
    };

    // Transform type badge
    let icon_color = get_transform_color(&rule.transform_type);
    let type_label = get_transform_label(&rule.transform_type);

    // Build icon based on transformation type
    let icon_element: Element<'_, Message> = match &rule.transform_type {
        NormalizationType::Constant => lucide::hash().size(12).color(WHITE).into(),
        NormalizationType::UsubjidPrefix => lucide::user().size(12).color(WHITE).into(),
        NormalizationType::SequenceNumber => lucide::list_ordered().size(12).color(WHITE).into(),
        NormalizationType::Iso8601DateTime | NormalizationType::Iso8601Date => {
            lucide::calendar().size(12).color(WHITE).into()
        }
        NormalizationType::Iso8601Duration => lucide::timer().size(12).color(WHITE).into(),
        NormalizationType::StudyDay { .. } => lucide::calendar_days().size(12).color(WHITE).into(),
        NormalizationType::CtNormalization { .. } => lucide::list().size(12).color(WHITE).into(),
        NormalizationType::NumericConversion => lucide::calculator().size(12).color(WHITE).into(),
        NormalizationType::CopyDirect => lucide::copy().size(12).color(WHITE).into(),
        _ => lucide::wand_sparkles().size(12).color(WHITE).into(),
    };

    let badge = container(
        row![
            icon_element,
            Space::new().width(SPACING_XS),
            text(type_label).size(11).color(WHITE),
        ]
        .align_y(Alignment::Center),
    )
    .padding([4.0, 10.0])
    .style(move |_theme: &Theme| container::Style {
        background: Some(icon_color.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    column![
        row![name, Space::new().width(Length::Fill), badge].align_y(Alignment::Center),
        Space::new().height(SPACING_XS),
        label,
    ]
    .into()
}

/// Transformation information section.
fn view_transformation_info<'a>(
    rule: &'a tss_normalization::NormalizationRule,
) -> Element<'a, Message> {
    let title_row = row![
        lucide::wand_sparkles().size(14).color(PRIMARY_500),
        Space::new().width(SPACING_SM),
        text("Transformation").size(14).color(GRAY_700),
    ]
    .align_y(Alignment::Center);

    // Description
    let description = text(&rule.description).size(13).color(GRAY_700);

    // Detailed explanation based on type
    let explanation = get_transform_explanation(&rule.transform_type);
    let explanation_text = text(explanation).size(12).color(GRAY_500);

    let content = column![
        description,
        Space::new().height(SPACING_SM),
        explanation_text,
    ];

    column![
        title_row,
        Space::new().height(SPACING_SM),
        container(content)
            .padding(SPACING_MD)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(GRAY_100.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .into()
}

/// Mapping information section.
fn view_mapping_info<'a>(
    domain: &'a crate::state::Domain,
    rule: &tss_normalization::NormalizationRule,
) -> Element<'a, Message> {
    let title_row = row![
        lucide::link().size(14).color(PRIMARY_500),
        Space::new().width(SPACING_SM),
        text("Source Mapping").size(14).color(GRAY_700),
    ]
    .align_y(Alignment::Center);

    let status = domain.mapping.status(&rule.target_variable);

    let content: Element<'a, Message> = match status {
        VariableStatus::Accepted => {
            let (source, confidence) = domain
                .mapping
                .accepted(&rule.target_variable)
                .unwrap_or(("Unknown", 0.0));
            let conf_pct = (confidence * 100.0) as u32;

            row![
                lucide::circle_check().size(16).color(SUCCESS),
                Space::new().width(SPACING_SM),
                column![
                    text("Mapped to source column").size(12).color(GRAY_600),
                    text(source).size(14).color(GRAY_900),
                    text(format!("{}% confidence", conf_pct))
                        .size(11)
                        .color(GRAY_500),
                ],
            ]
            .align_y(Alignment::Center)
            .into()
        }
        VariableStatus::Suggested => {
            let (source, confidence) = domain
                .mapping
                .suggestion(&rule.target_variable)
                .unwrap_or(("Unknown", 0.0));
            let conf_pct = (confidence * 100.0) as u32;

            row![
                lucide::lightbulb().size(16).color(WARNING),
                Space::new().width(SPACING_SM),
                column![
                    text(format!("Suggested: {} ({}% confidence)", source, conf_pct))
                        .size(13)
                        .color(GRAY_700),
                    text("Accept suggestion in Mapping tab")
                        .size(11)
                        .color(GRAY_500),
                ],
            ]
            .align_y(Alignment::Center)
            .into()
        }
        VariableStatus::AutoGenerated => row![
            lucide::cpu().size(16).color(PRIMARY_500),
            Space::new().width(SPACING_SM),
            column![
                text("Auto-generated").size(13).color(GRAY_700),
                text("Value computed automatically during export")
                    .size(11)
                    .color(GRAY_500),
            ],
        ]
        .align_y(Alignment::Center)
        .into(),
        VariableStatus::NotCollected => {
            let reason = domain
                .mapping
                .all_not_collected()
                .get(&rule.target_variable)
                .cloned()
                .unwrap_or_default();

            let reason_text = if reason.is_empty() {
                String::from("No reason provided")
            } else {
                reason
            };

            row![
                lucide::ban().size(16).color(GRAY_500),
                Space::new().width(SPACING_SM),
                column![
                    text("Not Collected").size(13).color(GRAY_700),
                    text(reason_text).size(11).color(GRAY_500),
                ],
            ]
            .align_y(Alignment::Center)
            .into()
        }
        VariableStatus::Omitted => row![
            lucide::eye_off().size(16).color(GRAY_500),
            Space::new().width(SPACING_SM),
            column![
                text("Omitted from output").size(13).color(GRAY_700),
                text("Variable will not be included in export")
                    .size(11)
                    .color(GRAY_500),
            ],
        ]
        .align_y(Alignment::Center)
        .into(),
        VariableStatus::Unmapped => row![
            lucide::circle_alert().size(16).color(ERROR),
            Space::new().width(SPACING_SM),
            column![
                text("Not Mapped").size(13).color(ERROR),
                text("Configure mapping in the Mapping tab")
                    .size(11)
                    .color(GRAY_500),
            ],
        ]
        .align_y(Alignment::Center)
        .into(),
    };

    column![
        title_row,
        Space::new().height(SPACING_SM),
        container(content)
            .padding(SPACING_MD)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(GRAY_100.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .into()
}

/// Variable metadata section.
fn view_variable_metadata<'a>(var: &'a tss_model::Variable) -> Element<'a, Message> {
    let title_row = row![
        lucide::info().size(14).color(PRIMARY_500),
        Space::new().width(SPACING_SM),
        text("Variable Metadata").size(14).color(GRAY_700),
    ]
    .align_y(Alignment::Center);

    let mut rows = column![].spacing(SPACING_SM);

    // Data type
    let type_str = match var.data_type {
        tss_model::VariableType::Char => "Character",
        tss_model::VariableType::Num => "Numeric",
    };
    rows = rows.push(view_metadata_row("Type", type_str));

    // Length
    if let Some(length) = var.length {
        rows = rows.push(view_metadata_row("Length", length.to_string()));
    }

    // Role
    if let Some(role) = var.role {
        rows = rows.push(view_metadata_row("Role", role.as_str()));
    }

    // Core designation
    if let Some(core) = var.core {
        rows = rows.push(view_metadata_row("Core", core.as_str()));
    }

    // Codelist
    if let Some(ref ct_code) = var.codelist_code {
        rows = rows.push(view_metadata_row("Codelist", ct_code));
    }

    // Described Value Domain
    if let Some(ref dvd) = var.described_value_domain {
        rows = rows.push(view_metadata_row("Format", dvd));
    }

    column![
        title_row,
        Space::new().height(SPACING_SM),
        container(rows)
            .padding(SPACING_MD)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(GRAY_100.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .into()
}

/// Metadata row helper.
fn view_metadata_row<'a>(label: &'static str, value: impl Into<String>) -> Element<'a, Message> {
    row![
        text(label)
            .size(12)
            .color(GRAY_600)
            .width(Length::Fixed(80.0)),
        text(value.into()).size(12).color(GRAY_800),
    ]
    .align_y(Alignment::Center)
    .into()
}

// =============================================================================
// NO SELECTION VIEW
// =============================================================================

/// Empty state when no rule is selected.
fn view_no_selection<'a>() -> Element<'a, Message> {
    container(
        column![
            lucide::wand_sparkles().size(48).color(GRAY_400),
            Space::new().height(SPACING_MD),
            text("Select a Rule").size(16).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Click a variable from the list to view its normalization details")
                .size(13)
                .color(GRAY_500),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}

// =============================================================================
// HELPERS
// =============================================================================

/// Get icon color for a transformation type.
fn get_transform_color(transform_type: &NormalizationType) -> iced::Color {
    match transform_type {
        NormalizationType::Constant => GRAY_600,
        NormalizationType::UsubjidPrefix => PRIMARY_500,
        NormalizationType::SequenceNumber => PRIMARY_500,
        NormalizationType::Iso8601DateTime | NormalizationType::Iso8601Date => {
            iced::Color::from_rgb(0.2, 0.6, 0.9)
        }
        NormalizationType::Iso8601Duration => iced::Color::from_rgb(0.2, 0.6, 0.9),
        NormalizationType::StudyDay { .. } => iced::Color::from_rgb(0.6, 0.4, 0.8),
        NormalizationType::CtNormalization { .. } => iced::Color::from_rgb(0.2, 0.7, 0.5),
        NormalizationType::NumericConversion => iced::Color::from_rgb(0.9, 0.5, 0.2),
        NormalizationType::CopyDirect => GRAY_500,
        _ => GRAY_500, // Fallback for non_exhaustive
    }
}

/// Get short label for a transformation type.
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

/// Get full label for a transformation type.
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

/// Get detailed explanation for a transformation type.
fn get_transform_explanation(transform_type: &NormalizationType) -> &'static str {
    match transform_type {
        NormalizationType::Constant => {
            "This value is set automatically from study configuration (STUDYID) or domain code (DOMAIN). No source mapping required."
        }
        NormalizationType::UsubjidPrefix => {
            "Unique Subject Identifier is derived by combining STUDYID with SUBJID in the format 'STUDYID-SUBJID'. This ensures uniqueness across studies."
        }
        NormalizationType::SequenceNumber => {
            "A unique sequence number is generated for each record within a subject (USUBJID) in this domain. Starts at 1 and increments."
        }
        NormalizationType::Iso8601DateTime => {
            "Date and time values are formatted to ISO 8601 standard (YYYY-MM-DDTHH:MM:SS). Partial dates are preserved (e.g., 2024-03 stays 2024-03)."
        }
        NormalizationType::Iso8601Date => {
            "Date values are formatted to ISO 8601 standard (YYYY-MM-DD). Partial dates are preserved to indicate data precision."
        }
        NormalizationType::Iso8601Duration => {
            "Duration values are formatted to ISO 8601 standard (PnYnMnDTnHnMnS or PnW). For example, '3 months 5 days' becomes 'P3M5D'."
        }
        NormalizationType::StudyDay { .. } => {
            "Study day is calculated as the number of days from the reference start date (RFSTDTC from DM). Day 1 is the reference date itself."
        }
        NormalizationType::CtNormalization { .. } => {
            "Values are normalized against CDISC Controlled Terminology. Synonyms are mapped to submission values, and invalid values are flagged."
        }
        NormalizationType::NumericConversion => {
            "Text values are converted to numeric (Float64) format. Non-numeric values result in null with a warning."
        }
        NormalizationType::CopyDirect => {
            "Value is copied directly from the source column without modification. Use for text fields that don't require normalization."
        }
        _ => "Custom transformation applied to this variable.",
    }
}
