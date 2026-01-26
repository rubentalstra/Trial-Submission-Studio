//! Detail panel components for the Normalization tab.
//!
//! Contains the right-side detail view with metadata, transformation info,
//! and before/after preview.

use iced::widget::{Space, column, container, row, rule, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;
use polars::prelude::AnyValue;

use crate::component::display::MetadataCard;
use crate::component::panels::DetailHeader;
use crate::message::Message;
use crate::state::SourceDomainState;
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
};

use tss_standards::TerminologyRegistry;
use tss_submit::NormalizationType;

use super::helpers::{get_transform_color, get_transform_explanation, get_transform_label};

// =============================================================================
// DETAIL PANEL
// =============================================================================

pub(super) fn view_rule_detail<'a>(
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
