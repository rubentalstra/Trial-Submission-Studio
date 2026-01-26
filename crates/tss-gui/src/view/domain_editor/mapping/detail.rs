//! Detail panel components for the Mapping tab.
//!
//! Contains the right-side detail view with metadata, status,
//! controlled terminology, and source column picker.

use iced::widget::{Space, column, container, pick_list, row, rule, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::display::{MetadataCard, StatusCard};
use crate::component::panels::DetailHeader;
use crate::message::domain_editor::MappingMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, SourceDomainState, ViewState};
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
};

use tss_standards::CoreDesignation;
use tss_submit::VariableStatus;

use super::actions::{view_mapping_actions, view_not_collected_inline_edit};

// =============================================================================
// DETAIL PANEL
// =============================================================================

pub(super) fn view_variable_detail<'a>(
    state: &'a AppState,
    source: &'a SourceDomainState,
    var: &'a tss_standards::SdtmVariable,
) -> Element<'a, Message> {
    let status = source.mapping.status(&var.name);
    let not_collected_edit = match &state.view {
        ViewState::DomainEditor(editor) => editor.mapping_ui.not_collected_edit.as_ref(),
        _ => None,
    };

    let header = DetailHeader::new(&var.name)
        .subtitle(var.label.as_deref().unwrap_or("No label"))
        .view();

    let metadata = view_variable_metadata(var);
    let ct_section: Element<'a, Message> = if var.codelist_code.is_some() {
        view_controlled_terminology(state, var)
    } else {
        Space::new().height(0.0).into()
    };
    let mapping_status = view_mapping_status(source, var, status);
    let source_picker: Element<'a, Message> =
        if matches!(status, VariableStatus::Unmapped | VariableStatus::Suggested) {
            view_source_column_picker(source, var)
        } else {
            Space::new().height(0.0).into()
        };

    let is_required = var.core == Some(CoreDesignation::Required);

    // Use explicit pattern matching to avoid conditional unwrap (#268)
    let actions: Element<'a, Message> = match not_collected_edit {
        Some(edit) if edit.variable == var.name => view_not_collected_inline_edit(var, edit),
        _ if is_required && !matches!(status, VariableStatus::Accepted) => {
            Space::new().height(0.0).into()
        }
        _ => view_mapping_actions(source, var, status),
    };

    scrollable(column![
        header,
        Space::new().height(SPACING_MD),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        metadata,
        Space::new().height(SPACING_LG),
        ct_section,
        mapping_status,
        Space::new().height(SPACING_MD),
        source_picker,
        Space::new().height(SPACING_LG),
        actions,
        Space::new().height(SPACING_MD),
    ])
    .height(Length::Fill)
    .into()
}

fn view_variable_metadata<'a>(var: &'a tss_standards::SdtmVariable) -> Element<'a, Message> {
    let type_str = match var.data_type {
        tss_standards::VariableType::Char => "Character",
        tss_standards::VariableType::Num => "Numeric",
    };

    let mut card = MetadataCard::new();

    if let Some(role) = var.role {
        card = card.row("Role", role.as_str());
    }
    if let Some(core) = var.core {
        card = card.row("Core", core.as_str());
    }
    card = card.row("Type", type_str);
    if let Some(length) = var.length {
        card = card.row("Length", length.to_string());
    }
    if let Some(ref ct_code) = var.codelist_code {
        card = card.row("Codelist", ct_code);
    }
    if let Some(ref dvd) = var.described_value_domain {
        card = card.row("Format", dvd);
    }

    card.view()
}

fn view_mapping_status<'a>(
    source: &'a SourceDomainState,
    var: &'a tss_standards::SdtmVariable,
    status: VariableStatus,
) -> Element<'a, Message> {
    let title = text("Mapping Status")
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    let status_content: Element<'a, Message> = match status {
        VariableStatus::Accepted => {
            if let Some((col, conf)) = source.mapping.accepted(&var.name) {
                let conf_pct = (conf * 100.0) as u32;
                StatusCard::new(container(lucide::circle_check().size(16)).style(
                    |theme: &Theme| container::Style {
                        text_color: Some(theme.extended_palette().success.base.color),
                        ..Default::default()
                    },
                ))
                .title("Mapped to:")
                .value(col)
                .description(format!("{}% confidence", conf_pct))
                .background_themed(|theme: &Theme| theme.clinical().status_success_light)
                .border_color_themed(|theme: &Theme| theme.extended_palette().success.base.color)
                .view()
            } else {
                view_status_unmapped()
            }
        }
        VariableStatus::AutoGenerated => StatusCard::new(
            container(lucide::settings().size(16)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_muted),
                ..Default::default()
            }),
        )
        .value("Auto-generated")
        .description("This variable is populated automatically by the system")
        .view(),
        VariableStatus::Suggested => {
            if let Some((col, conf)) = source.mapping.suggestion(&var.name) {
                let conf_pct = (conf * 100.0) as u32;
                let var_name = var.name.clone();
                StatusCard::new(
                    container(lucide::lightbulb().size(16)).style(|theme: &Theme| {
                        container::Style {
                            text_color: Some(theme.extended_palette().warning.base.color),
                            ..Default::default()
                        }
                    }),
                )
                .title("Suggested mapping:")
                .value(col)
                .description(format!("{}% confidence", conf_pct))
                .action(
                    "Accept Suggestion",
                    Message::DomainEditor(DomainEditorMessage::Mapping(
                        MappingMessage::AcceptSuggestion(var_name),
                    )),
                )
                .background_themed(|theme: &Theme| theme.clinical().status_warning_light)
                .border_color_themed(|theme: &Theme| theme.extended_palette().warning.base.color)
                .view()
            } else {
                view_status_unmapped()
            }
        }
        VariableStatus::NotCollected => {
            let reason = source
                .mapping
                .not_collected_reason(&var.name)
                .unwrap_or("No reason provided");
            StatusCard::new(container(lucide::ban().size(16)).style(|theme: &Theme| {
                container::Style {
                    text_color: Some(theme.clinical().text_muted),
                    ..Default::default()
                }
            }))
            .value("Not Collected")
            .description(reason)
            .view()
        }
        VariableStatus::Omitted => StatusCard::new(container(lucide::eye_off().size(16)).style(
            |theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_muted),
                ..Default::default()
            },
        ))
        .value("Omitted")
        .description("This permissible variable will not be included in output")
        .view(),
        VariableStatus::Unmapped => view_status_unmapped(),
    };

    column![title, Space::new().height(SPACING_SM), status_content].into()
}

fn view_status_unmapped<'a, M: Clone + 'a>() -> Element<'a, M> {
    StatusCard::new(
        container(lucide::circle().size(16)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_disabled),
            ..Default::default()
        }),
    )
    .value("Not Mapped")
    .description("Select a source column below to map this variable")
    .view()
}

// =============================================================================
// CONTROLLED TERMINOLOGY
// =============================================================================

fn view_controlled_terminology<'a>(
    state: &'a AppState,
    var: &'a tss_standards::SdtmVariable,
) -> Element<'a, Message> {
    let ct_code = match &var.codelist_code {
        Some(code) => code,
        None => return Space::new().height(0.0).into(),
    };

    let resolved = state
        .terminology
        .as_ref()
        .and_then(|reg| reg.resolve(ct_code, None));

    let title_row = row![
        container(lucide::list().size(14)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().primary.base.color),
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text("Controlled Terminology")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary)
            }),
    ]
    .align_y(Alignment::Center);

    let content: Element<'a, Message> = if let Some(resolved) = resolved {
        let codelist = resolved.codelist;
        let codelist_name = text(&codelist.name)
            .size(13)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            });
        let codelist_code_text = text(format!("({})", ct_code))
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            });

        let extensible_badge = if codelist.extensible {
            container(
                text("Extensible")
                    .size(10)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted),
                    }),
            )
            .padding([2.0, 6.0])
            .style(|theme: &Theme| container::Style {
                background: Some(theme.clinical().border_default.into()),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
        } else {
            container(
                text("Non-extensible")
                    .size(10)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.extended_palette().danger.base.color),
                    }),
            )
            .padding([2.0, 6.0])
            .style(|theme: &Theme| container::Style {
                background: Some(theme.clinical().status_error_light.into()),
                border: Border {
                    radius: 4.0.into(),
                    color: theme.extended_palette().danger.base.color,
                    width: 1.0,
                },
                ..Default::default()
            })
        };

        let header_row = row![
            codelist_name,
            Space::new().width(SPACING_XS),
            codelist_code_text,
            Space::new().width(Length::Fill),
            extensible_badge,
        ]
        .align_y(Alignment::Center);

        let terms: Vec<_> = codelist.terms.values().collect();
        let show_count = terms.len().min(10);
        let has_more = terms.len() > 10;

        let mut terms_list = column![].spacing(2.0);
        let terms_header = row![
            text("Value")
                .size(10)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted)
                })
                .width(Length::Fixed(350.0)),
            text("Meaning").size(10).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted)
            }),
        ]
        .padding(SPACING_SM);

        terms_list = terms_list.push(terms_header);
        terms_list = terms_list.push(rule::horizontal(1).style(|theme: &Theme| rule::Style {
            color: theme.clinical().border_default,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }));

        for term in terms.iter().take(show_count) {
            let meaning = term
                .preferred_term
                .as_deref()
                .unwrap_or(&term.submission_value);
            let submission_value = term.submission_value.clone();
            let meaning_owned = meaning.to_string();
            let term_row = row![
                text(submission_value)
                    .size(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.extended_palette().primary.base.color)
                    })
                    .width(Length::Fixed(350.0)),
                text(meaning_owned)
                    .size(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_secondary)
                    }),
            ]
            .padding(SPACING_SM)
            .align_y(Alignment::Center);
            terms_list = terms_list.push(term_row);
        }

        if has_more {
            let remaining = terms.len() - show_count;
            terms_list = terms_list.push(
                container(
                    text(format!("... and {} more values", remaining))
                        .size(11)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_muted),
                        }),
                )
                .padding([4.0, 8.0])
                .width(Length::Fill),
            );
        }

        let terms_container = container(terms_list)
            .width(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.clinical().background_elevated.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    color: theme.clinical().border_default,
                    width: 1.0,
                },
                ..Default::default()
            });

        column![
            header_row,
            Space::new().height(SPACING_SM),
            text("Allowed Values:")
                .size(11)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted)
                }),
            Space::new().height(SPACING_XS),
            terms_container,
        ]
        .into()
    } else {
        row![
            container(lucide::triangle_alert().size(12)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().warning.base.color),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text(format!("Codelist {} not found in terminology", ct_code))
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted)
                }),
        ]
        .align_y(Alignment::Center)
        .into()
    };

    column![
        title_row,
        Space::new().height(SPACING_SM),
        container(content)
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
        Space::new().height(SPACING_LG),
    ]
    .into()
}

// =============================================================================
// SOURCE COLUMN PICKER
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct ColumnOption {
    name: String,
    is_suggested: bool,
    confidence: Option<u32>,
}

impl std::fmt::Display for ColumnOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_suggested {
            if let Some(conf) = self.confidence {
                write!(f, "{} (Best match - {}%)", self.name, conf)
            } else {
                write!(f, "{} (Suggested)", self.name)
            }
        } else {
            write!(f, "{}", self.name)
        }
    }
}

fn view_source_column_picker<'a>(
    source: &'a SourceDomainState,
    var: &'a tss_standards::SdtmVariable,
) -> Element<'a, Message> {
    let source_columns = source.source.column_names();
    let mapped_columns: std::collections::BTreeSet<String> = source
        .mapping
        .all_accepted()
        .values()
        .map(|(col, _): &(String, f32)| col.clone())
        .collect();

    let suggestion = source.mapping.suggestion(&var.name);
    let suggested_col: Option<&str> = suggestion.as_ref().map(|(col, _)| *col);
    let suggested_conf = suggestion.as_ref().map(|(_, conf)| (*conf * 100.0) as u32);

    let mut column_options: Vec<ColumnOption> = source_columns
        .into_iter()
        .filter(|col| !mapped_columns.contains(col))
        .map(|col| {
            let is_suggested = suggested_col == Some(col.as_str());
            ColumnOption {
                name: col,
                is_suggested,
                confidence: if is_suggested { suggested_conf } else { None },
            }
        })
        .collect();

    column_options.sort_by(|a, b| match (a.is_suggested, b.is_suggested) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    if column_options.is_empty() {
        return container(
            row![
                container(lucide::info().size(14)).style(|theme: &Theme| container::Style {
                    text_color: Some(theme.clinical().text_muted),
                    ..Default::default()
                }),
                Space::new().width(SPACING_SM),
                text("All source columns are already mapped")
                    .size(13)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted)
                    }),
            ]
            .align_y(Alignment::Center),
        )
        .padding(SPACING_MD)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into();
    }

    let var_name = var.name.clone();
    let column_count = column_options.len();
    let has_suggestion = column_options.iter().any(|c| c.is_suggested);

    let picker = pick_list(
        column_options,
        None::<ColumnOption>,
        move |selected: ColumnOption| {
            Message::DomainEditor(DomainEditorMessage::Mapping(MappingMessage::ManualMap {
                variable: var_name.clone(),
                column: selected.name,
            }))
        },
    )
    .placeholder("Select a source column...")
    .width(Length::Fill)
    .padding([10.0, 14.0])
    .text_size(14.0)
    .style(|theme: &Theme, status| {
        let clinical = theme.clinical();
        let palette = theme.extended_palette();
        let border_color = match status {
            pick_list::Status::Active => palette.primary.base.color,
            pick_list::Status::Hovered => clinical.text_disabled,
            pick_list::Status::Opened { .. } => palette.primary.base.color,
        };
        pick_list::Style {
            text_color: palette.background.base.text,
            placeholder_color: clinical.text_muted,
            handle_color: clinical.text_secondary,
            background: clinical.background_elevated.into(),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                width: 1.0,
                color: border_color,
            },
        }
    });

    let helper_text = if has_suggestion {
        format!(
            "{} columns available - best match shown first",
            column_count
        )
    } else {
        format!("{} columns available", column_count)
    };

    column![
        text("Map to Source Column")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary)
            }),
        Space::new().height(SPACING_SM),
        picker,
        Space::new().height(SPACING_XS),
        text(helper_text)
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted)
            }),
    ]
    .into()
}
