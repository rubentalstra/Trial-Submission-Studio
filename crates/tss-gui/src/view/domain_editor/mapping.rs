//! Mapping tab view.
//!
//! The mapping tab displays a master-detail interface for mapping
//! source columns to SDTM target variables.
//!
//! - **Left (Master)**: List of TARGET SDTM variables with status indicators
//! - **Right (Detail)**: Selected variable details with mapping controls

use iced::widget::{Space, button, column, container, row, rule, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length, Theme};

use crate::component::master_detail;
use crate::message::domain_editor::MappingMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, MappingUiState, ViewState};
use crate::theme::{
    BORDER_RADIUS_SM, ERROR, GRAY_100, GRAY_200, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800,
    GRAY_900, PRIMARY_100, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, SUCCESS,
    WARNING, WHITE, button_primary, button_secondary,
};

use tss_map::VariableStatus;
use tss_model::CoreDesignation;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Width of the master (variable list) panel.
const MASTER_WIDTH: f32 = 320.0;

// =============================================================================
// MAIN MAPPING TAB VIEW
// =============================================================================

/// Render the mapping tab content using master-detail layout.
pub fn view_mapping_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
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
    let mapping_ui = match &state.view {
        ViewState::DomainEditor { mapping_ui, .. } => mapping_ui,
        _ => return text("Invalid view state").into(),
    };

    // Get target variables from the SDTM domain definition
    let sdtm_domain = domain.mapping.domain();

    // Apply filters - collect indices of variables that match
    let filtered_indices: Vec<usize> = sdtm_domain
        .variables
        .iter()
        .enumerate()
        .filter(|(_idx, var)| {
            // Search filter
            if !mapping_ui.search_filter.is_empty() {
                let search = mapping_ui.search_filter.to_lowercase();
                let name_match = var.name.to_lowercase().contains(&search);
                let label_match = var
                    .label
                    .as_ref()
                    .map(|l| l.to_lowercase().contains(&search))
                    .unwrap_or(false);
                if !name_match && !label_match {
                    return false;
                }
            }

            // Unmapped filter
            if mapping_ui.filter_unmapped {
                let status = domain.mapping.status(&var.name);
                if !matches!(status, VariableStatus::Unmapped | VariableStatus::Suggested) {
                    return false;
                }
            }

            // Required filter
            if mapping_ui.filter_required && var.core != Some(CoreDesignation::Required) {
                return false;
            }

            true
        })
        .map(|(idx, _)| idx)
        .collect();

    // Build master panel (variable list)
    let master = view_variable_list(domain, &filtered_indices, mapping_ui);

    // Build detail panel (selected variable)
    let detail = if let Some(selected_idx) = mapping_ui.selected_variable {
        if let Some(var) = sdtm_domain.variables.get(selected_idx) {
            view_variable_detail(domain, var)
        } else {
            view_no_selection()
        }
    } else {
        view_no_selection()
    };

    // Use master-detail layout
    master_detail(master, detail, MASTER_WIDTH)
}

// =============================================================================
// MASTER PANEL: VARIABLE LIST
// =============================================================================

/// Left panel: list of target SDTM variables.
fn view_variable_list<'a>(
    domain: &'a crate::state::Domain,
    filtered_indices: &[usize],
    mapping_ui: &'a MappingUiState,
) -> Element<'a, Message> {
    // Header with search and filters
    let header = view_list_header(mapping_ui);

    // Summary stats
    let summary = domain.summary();
    let stats = row![
        text(format!("{}/{}", summary.mapped, summary.total_variables))
            .size(12)
            .color(GRAY_600),
        Space::new().width(4.0),
        text("mapped").size(11).color(GRAY_500),
    ]
    .align_y(Alignment::Center);

    // Get the SDTM domain for variable lookup
    let sdtm_domain = domain.mapping.domain();

    // Variable items
    let mut items = column![].spacing(SPACING_XS);

    for &idx in filtered_indices {
        if let Some(var) = sdtm_domain.variables.get(idx) {
            let status = domain.mapping.status(&var.name);
            let is_selected = mapping_ui.selected_variable == Some(idx);
            let item = view_variable_item(idx, var, status, is_selected);
            items = items.push(item);
        }
    }

    // Empty state
    let content: Element<'a, Message> = if filtered_indices.is_empty() {
        container(
            column![
                text("No variables match your filters")
                    .size(13)
                    .color(GRAY_500),
                Space::new().height(SPACING_SM),
                button(text("Clear filters").size(12))
                    .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
                        MappingMessage::SearchCleared,
                    )))
                    .padding([6.0, 12.0])
                    .style(button_secondary),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(SPACING_LG)
        .center_x(Length::Shrink)
        .into()
    } else {
        scrollable(items).height(Length::Fill).into()
    };

    column![
        header,
        Space::new().height(SPACING_SM),
        stats,
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
        content,
    ]
    .into()
}

/// Header with search box and filter toggles.
fn view_list_header<'a>(mapping_ui: &'a MappingUiState) -> Element<'a, Message> {
    // Search input
    let search_input = text_input("Search variables...", &mapping_ui.search_filter)
        .on_input(|s| {
            Message::DomainEditor(DomainEditorMessage::Mapping(MappingMessage::SearchChanged(
                s,
            )))
        })
        .padding([8.0, 12.0])
        .size(13)
        .width(Length::Fill);

    // Filter buttons
    let filter_unmapped = mapping_ui.filter_unmapped;
    let unmapped_btn = button(text("Unmapped").size(11))
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::FilterUnmappedToggled(!filter_unmapped),
        )))
        .padding([4.0, 8.0])
        .style(move |theme: &Theme, status| {
            if filter_unmapped {
                iced::widget::button::Style {
                    background: Some(PRIMARY_100.into()),
                    text_color: PRIMARY_500,
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        color: PRIMARY_500,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            } else {
                button_secondary(theme, status)
            }
        });

    let filter_required = mapping_ui.filter_required;
    let required_btn = button(text("Required").size(11))
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::FilterRequiredToggled(!filter_required),
        )))
        .padding([4.0, 8.0])
        .style(move |theme: &Theme, status| {
            if filter_required {
                iced::widget::button::Style {
                    background: Some(PRIMARY_100.into()),
                    text_color: PRIMARY_500,
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        color: PRIMARY_500,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            } else {
                button_secondary(theme, status)
            }
        });

    let filters = row![unmapped_btn, required_btn,].spacing(SPACING_XS);

    column![search_input, Space::new().height(SPACING_XS), filters,].into()
}

/// Single variable item in the list.
fn view_variable_item<'a>(
    index: usize,
    var: &'a tss_model::Variable,
    status: VariableStatus,
    is_selected: bool,
) -> Element<'a, Message> {
    // Status indicator
    let (status_icon, status_color) = match status {
        VariableStatus::Accepted => ("\u{f058}", SUCCESS), // check-circle
        VariableStatus::AutoGenerated => ("\u{f013}", GRAY_500), // cog
        VariableStatus::Suggested => ("\u{f0eb}", WARNING), // lightbulb
        VariableStatus::NotCollected => ("\u{f05e}", GRAY_400), // ban
        VariableStatus::Omitted => ("\u{f070}", GRAY_400), // eye-slash
        VariableStatus::Unmapped => ("\u{f111}", GRAY_400), // circle (outline)
    };

    let status_icon_el = text(status_icon)
        .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
        .size(12)
        .color(status_color);

    // Core designation badge
    let core_badge = match var.core {
        Some(CoreDesignation::Required) => Some(("Req", ERROR)),
        Some(CoreDesignation::Expected) => Some(("Exp", WARNING)),
        Some(CoreDesignation::Permissible) => Some(("Perm", GRAY_500)),
        None => None,
    };

    let core_el: Element<'a, Message> = if let Some((label, color)) = core_badge {
        container(text(label).size(9).color(WHITE))
            .padding([2.0, 4.0])
            .style(move |_theme: &Theme| container::Style {
                background: Some(color.into()),
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    } else {
        Space::new().width(0.0).into()
    };

    // Variable name and label
    let name = text(&var.name).size(13).color(GRAY_900);
    let label = text(var.label.as_deref().unwrap_or(""))
        .size(11)
        .color(GRAY_500);

    let content = row![
        status_icon_el,
        Space::new().width(SPACING_SM),
        column![name, label,].width(Length::Fill),
        core_el,
    ]
    .align_y(Alignment::Center)
    .padding([SPACING_SM, SPACING_SM]);

    button(content)
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::VariableSelected(index),
        )))
        .width(Length::Fill)
        .style(move |_theme: &Theme, btn_status| {
            let bg = if is_selected {
                Some(PRIMARY_100.into())
            } else {
                match btn_status {
                    iced::widget::button::Status::Hovered => Some(GRAY_100.into()),
                    _ => None,
                }
            };
            let border_color = if is_selected { PRIMARY_500 } else { GRAY_200 };
            iced::widget::button::Style {
                background: bg,
                text_color: GRAY_900,
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    color: border_color,
                    width: if is_selected { 1.0 } else { 0.0 },
                },
                ..Default::default()
            }
        })
        .into()
}

// =============================================================================
// DETAIL PANEL: SELECTED VARIABLE
// =============================================================================

/// Right panel: details for selected variable.
fn view_variable_detail<'a>(
    domain: &'a crate::state::Domain,
    var: &'a tss_model::Variable,
) -> Element<'a, Message> {
    let status = domain.mapping.status(&var.name);

    // Header with variable name
    let header = view_detail_header(var);

    // Metadata section
    let metadata = view_variable_metadata(var);

    // Current mapping status
    let mapping_status = view_mapping_status(domain, var, status);

    // Actions section
    let actions = view_mapping_actions(var, status);

    column![
        header,
        Space::new().height(SPACING_MD),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        metadata,
        Space::new().height(SPACING_LG),
        mapping_status,
        Space::new().height(SPACING_LG),
        actions,
    ]
    .into()
}

/// Detail header with variable name.
fn view_detail_header<'a>(var: &'a tss_model::Variable) -> Element<'a, Message> {
    let name = text(&var.name).size(20).color(GRAY_900);
    let label = text(var.label.as_deref().unwrap_or("No label"))
        .size(14)
        .color(GRAY_600);

    column![name, Space::new().height(SPACING_XS), label,].into()
}

/// Variable metadata display.
fn view_variable_metadata<'a>(var: &'a tss_model::Variable) -> Element<'a, Message> {
    let mut rows = column![].spacing(SPACING_SM);

    // Role
    if let Some(role) = var.role {
        rows = rows.push(view_metadata_row("Role", role.as_str()));
    }

    // Core designation
    if let Some(core) = var.core {
        rows = rows.push(view_metadata_row("Core", core.as_str()));
    }

    // Data type
    let type_str = match var.data_type {
        tss_model::VariableType::Char => "Character",
        tss_model::VariableType::Num => "Numeric",
    };
    rows = rows.push(view_metadata_row("Type", type_str));

    // Length
    if let Some(length) = var.length {
        rows = rows.push(view_metadata_row("Length", &length.to_string()));
    }

    // Controlled Terminology
    if let Some(ref ct_code) = var.codelist_code {
        rows = rows.push(view_metadata_row("Codelist", ct_code));
    }

    // Described Value Domain
    if let Some(ref dvd) = var.described_value_domain {
        rows = rows.push(view_metadata_row("Format", dvd));
    }

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
        })
        .into()
}

/// Single metadata row.
fn view_metadata_row<'a>(label: &'static str, value: impl ToString) -> Element<'a, Message> {
    row![
        text(label)
            .size(12)
            .color(GRAY_600)
            .width(Length::Fixed(80.0)),
        text(value.to_string()).size(12).color(GRAY_800),
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Current mapping status display.
fn view_mapping_status<'a>(
    domain: &'a crate::state::Domain,
    var: &'a tss_model::Variable,
    status: VariableStatus,
) -> Element<'a, Message> {
    let title = text("Mapping Status").size(14).color(GRAY_700);

    let status_content: Element<'a, Message> = match status {
        VariableStatus::Accepted => {
            if let Some((col, conf)) = domain.mapping.accepted(&var.name) {
                view_status_accepted(col, conf)
            } else {
                view_status_unmapped()
            }
        }
        VariableStatus::AutoGenerated => view_status_auto_generated(),
        VariableStatus::Suggested => {
            if let Some((col, conf)) = domain.mapping.suggestion(&var.name) {
                view_status_suggested(&var.name, col, conf)
            } else {
                view_status_unmapped()
            }
        }
        VariableStatus::NotCollected => {
            let reason = domain
                .mapping
                .not_collected_reason(&var.name)
                .unwrap_or("No reason provided");
            view_status_not_collected(reason)
        }
        VariableStatus::Omitted => view_status_omitted(),
        VariableStatus::Unmapped => view_status_unmapped(),
    };

    column![title, Space::new().height(SPACING_SM), status_content,].into()
}

fn view_status_accepted<'a>(col: &'a str, confidence: f32) -> Element<'a, Message> {
    let conf_pct = (confidence * 100.0) as u32;
    container(
        row![
            text("\u{f058}") // check-circle
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(16)
                .color(SUCCESS),
            Space::new().width(SPACING_SM),
            column![
                text("Mapped to:").size(12).color(GRAY_600),
                text(col).size(14).color(GRAY_900),
                text(format!("{}% confidence", conf_pct))
                    .size(11)
                    .color(GRAY_500),
            ],
        ]
        .align_y(Alignment::Center),
    )
    .padding(SPACING_MD)
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Color::from_rgb(0.9, 0.98, 0.92).into()),
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            color: SUCCESS,
            width: 1.0,
        },
        ..Default::default()
    })
    .into()
}

fn view_status_auto_generated<'a>() -> Element<'a, Message> {
    container(
        row![
            text("\u{f013}") // cog
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(16)
                .color(GRAY_600),
            Space::new().width(SPACING_SM),
            column![
                text("Auto-generated").size(14).color(GRAY_800),
                text("This variable is populated automatically by the system")
                    .size(12)
                    .color(GRAY_500),
            ],
        ]
        .align_y(Alignment::Center),
    )
    .padding(SPACING_MD)
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn view_status_suggested<'a>(
    var_name: &'a str,
    col: &'a str,
    confidence: f32,
) -> Element<'a, Message> {
    let conf_pct = (confidence * 100.0) as u32;
    let var_name_owned = var_name.to_string();

    container(column![
        row![
            text("\u{f0eb}") // lightbulb
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(16)
                .color(WARNING),
            Space::new().width(SPACING_SM),
            column![
                text("Suggested mapping:").size(12).color(GRAY_600),
                text(col).size(14).color(GRAY_900),
                text(format!("{}% confidence", conf_pct))
                    .size(11)
                    .color(GRAY_500),
            ],
        ]
        .align_y(Alignment::Center),
        Space::new().height(SPACING_SM),
        button(
            row![
                text("\u{f00c}") // check
                    .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                    .size(12),
                Space::new().width(SPACING_XS),
                text("Accept Suggestion").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::AcceptSuggestion(var_name_owned),
        )))
        .padding([8.0, 16.0])
        .style(button_primary),
    ])
    .padding(SPACING_MD)
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Color::from_rgb(1.0, 0.98, 0.9).into()),
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            color: WARNING,
            width: 1.0,
        },
        ..Default::default()
    })
    .into()
}

fn view_status_not_collected<'a>(reason: &'a str) -> Element<'a, Message> {
    container(
        row![
            text("\u{f05e}") // ban
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(16)
                .color(GRAY_500),
            Space::new().width(SPACING_SM),
            column![
                text("Not Collected").size(14).color(GRAY_800),
                text(reason).size(12).color(GRAY_500),
            ],
        ]
        .align_y(Alignment::Center),
    )
    .padding(SPACING_MD)
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn view_status_omitted<'a>() -> Element<'a, Message> {
    container(
        row![
            text("\u{f070}") // eye-slash
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(16)
                .color(GRAY_500),
            Space::new().width(SPACING_SM),
            column![
                text("Omitted").size(14).color(GRAY_800),
                text("This permissible variable will not be included in output")
                    .size(12)
                    .color(GRAY_500),
            ],
        ]
        .align_y(Alignment::Center),
    )
    .padding(SPACING_MD)
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn view_status_unmapped<'a>() -> Element<'a, Message> {
    container(
        row![
            text("\u{f111}") // circle
                .font(iced::Font::with_name("Font Awesome 6 Free Regular"))
                .size(16)
                .color(GRAY_400),
            Space::new().width(SPACING_SM),
            column![
                text("Not Mapped").size(14).color(GRAY_800),
                text("Select a source column below to map this variable")
                    .size(12)
                    .color(GRAY_500),
            ],
        ]
        .align_y(Alignment::Center),
    )
    .padding(SPACING_MD)
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

/// Action buttons for the variable.
fn view_mapping_actions<'a>(
    var: &'a tss_model::Variable,
    status: VariableStatus,
) -> Element<'a, Message> {
    let var_name = var.name.clone();
    let title = text("Actions").size(14).color(GRAY_700);

    let mut action_buttons: Vec<Element<'a, Message>> = Vec::new();

    // Clear mapping button (if mapped)
    if matches!(status, VariableStatus::Accepted) {
        let clear_btn = button(
            row![
                text("\u{f00d}") // times
                    .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                    .size(12),
                Space::new().width(SPACING_XS),
                text("Clear Mapping").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::ClearMapping(var_name.clone()),
        )))
        .padding([8.0, 16.0])
        .style(button_secondary);

        action_buttons.push(clear_btn.into());
    }

    // Mark Not Collected button (if not Required and not already)
    if var.core != Some(CoreDesignation::Required)
        && !matches!(
            status,
            VariableStatus::NotCollected | VariableStatus::AutoGenerated
        )
    {
        let not_collected_btn = button(
            row![
                text("\u{f05e}") // ban
                    .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                    .size(12),
                Space::new().width(SPACING_XS),
                text("Mark Not Collected").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::MarkNotCollected {
                variable: var_name.clone(),
            },
        )))
        .padding([8.0, 16.0])
        .style(button_secondary);

        action_buttons.push(not_collected_btn.into());
    }

    // Mark Omit button (if Permissible and not already)
    if (var.core.is_none() || var.core == Some(CoreDesignation::Permissible))
        && !matches!(
            status,
            VariableStatus::Omitted | VariableStatus::AutoGenerated
        )
    {
        let omit_btn = button(
            row![
                text("\u{f070}") // eye-slash
                    .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                    .size(12),
                Space::new().width(SPACING_XS),
                text("Omit from Output").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::MarkOmitted(var_name.clone()),
        )))
        .padding([8.0, 16.0])
        .style(button_secondary);

        action_buttons.push(omit_btn.into());
    }

    // Clear Omit button (if omitted)
    if matches!(status, VariableStatus::Omitted) {
        let clear_omit_btn = button(
            row![
                text("\u{f06e}") // eye
                    .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                    .size(12),
                Space::new().width(SPACING_XS),
                text("Include in Output").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::ClearOmitted(var_name),
        )))
        .padding([8.0, 16.0])
        .style(button_secondary);

        action_buttons.push(clear_omit_btn.into());
    }

    let actions_content: Element<'a, Message> = if action_buttons.is_empty() {
        text("No additional actions available")
            .size(12)
            .color(GRAY_500)
            .into()
    } else {
        let mut actions_col = column![].spacing(SPACING_SM);
        for btn in action_buttons {
            actions_col = actions_col.push(btn);
        }
        actions_col.into()
    };

    column![title, Space::new().height(SPACING_SM), actions_content,].into()
}

// =============================================================================
// EMPTY STATE
// =============================================================================

/// Empty state when no variable is selected.
fn view_no_selection<'a>() -> Element<'a, Message> {
    container(
        column![
            text("\u{f0a6}") // hand-pointer
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(48)
                .color(GRAY_400),
            Space::new().height(SPACING_MD),
            text("Select a Variable").size(16).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Click a variable from the list to view details and configure mapping")
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
