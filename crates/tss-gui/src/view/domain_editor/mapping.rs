//! Mapping tab view.
//!
//! The mapping tab displays a master-detail interface for mapping
//! source columns to SDTM target variables.
//!
//! - **Left (Master)**: List of TARGET SDTM variables with status indicators
//! - **Right (Detail)**: Selected variable details with mapping controls

use iced::widget::{Space, button, column, container, row, rule, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::master_detail_with_pinned_header;
use crate::message::domain_editor::MappingMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, MappingUiState, NotCollectedEdit, ViewState};
use crate::theme::{
    BORDER_RADIUS_SM, ERROR, GRAY_100, GRAY_200, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800,
    GRAY_900, PRIMARY_100, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, SUCCESS,
    WARNING, WHITE, button_primary, button_secondary,
};

use tss_standards::CoreDesignation;
use tss_submit::VariableStatus;

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

    // Build master panel header (search, filters, stats - pinned at top)
    let master_header = view_variable_list_header(domain, mapping_ui);

    // Build master panel content (scrollable variable list)
    let master_content = view_variable_list_content(domain, &filtered_indices, mapping_ui);

    // Build detail panel (selected variable)
    let detail = if let Some(selected_idx) = mapping_ui.selected_variable {
        if let Some(var) = sdtm_domain.variables.get(selected_idx) {
            view_variable_detail(state, domain, var)
        } else {
            view_no_selection()
        }
    } else {
        view_no_selection()
    };

    // Use master-detail layout with pinned header
    master_detail_with_pinned_header(master_header, master_content, detail, MASTER_WIDTH)
}

// =============================================================================
// MASTER PANEL: VARIABLE LIST
// =============================================================================

/// Left panel header: search, filters, and stats (pinned at top).
fn view_variable_list_header<'a>(
    domain: &'a crate::state::DomainState,
    mapping_ui: &'a MappingUiState,
) -> Element<'a, Message> {
    // Search and filter controls
    let search_filters = view_search_and_filters(mapping_ui);

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

    column![
        search_filters,
        Space::new().height(SPACING_SM),
        stats,
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
    ]
    .into()
}

/// Left panel content: scrollable list of variables.
fn view_variable_list_content<'a>(
    domain: &'a crate::state::DomainState,
    filtered_indices: &[usize],
    mapping_ui: &'a MappingUiState,
) -> Element<'a, Message> {
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

    // Empty state or list
    if filtered_indices.is_empty() {
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
        items.into()
    }
}

/// Search box and filter toggle buttons.
fn view_search_and_filters<'a>(mapping_ui: &'a MappingUiState) -> Element<'a, Message> {
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
    var: &'a tss_standards::SdtmVariable,
    status: VariableStatus,
    is_selected: bool,
) -> Element<'a, Message> {
    // Status indicator using lucide icons
    let status_icon_el: Element<'a, Message> = match status {
        VariableStatus::Accepted => lucide::circle_check().size(12).color(SUCCESS).into(),
        VariableStatus::AutoGenerated => lucide::settings().size(12).color(GRAY_500).into(),
        VariableStatus::Suggested => lucide::lightbulb().size(12).color(WARNING).into(),
        VariableStatus::NotCollected => lucide::ban().size(12).color(GRAY_400).into(),
        VariableStatus::Omitted => lucide::eye_off().size(12).color(GRAY_400).into(),
        VariableStatus::Unmapped => lucide::circle().size(12).color(GRAY_400).into(),
    };

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
    state: &'a AppState,
    domain: &'a crate::state::DomainState,
    var: &'a tss_standards::SdtmVariable,
) -> Element<'a, Message> {
    let status = domain.mapping.status(&var.name);

    // Get mapping UI state for not_collected inline editing
    let not_collected_edit = match &state.view {
        ViewState::DomainEditor { mapping_ui, .. } => mapping_ui.not_collected_edit.as_ref(),
        _ => None,
    };

    // Header with variable name
    let header = view_detail_header(var);

    // Metadata section
    let metadata = view_variable_metadata(var);

    // Controlled Terminology section (if variable has a codelist)
    let ct_section: Element<'a, Message> = if var.codelist_code.is_some() {
        view_controlled_terminology(state, var)
    } else {
        Space::new().height(0.0).into()
    };

    // Current mapping status
    let mapping_status = view_mapping_status(domain, var, status);

    // Source column picker (only show if unmapped or suggested)
    let source_picker: Element<'a, Message> =
        if matches!(status, VariableStatus::Unmapped | VariableStatus::Suggested) {
            view_source_column_picker(domain, var)
        } else {
            Space::new().height(0.0).into()
        };

    // Check if we're in "not collected" entry mode for this variable
    let is_editing_not_collected = not_collected_edit
        .map(|e| e.variable == var.name)
        .unwrap_or(false);

    // Actions section - only show for non-Required variables
    // Required variables must be mapped, so no alternative actions apply
    let is_required = var.core == Some(CoreDesignation::Required);

    let actions: Element<'a, Message> = if is_editing_not_collected {
        view_not_collected_inline_edit(var, not_collected_edit.unwrap())
    } else if is_required && !matches!(status, VariableStatus::Accepted) {
        // Required but not yet mapped - no actions to show
        Space::new().height(0.0).into()
    } else if is_required {
        // Required and mapped - just show clear mapping option
        view_mapping_actions(domain, var, status)
    } else {
        // Expected or Permissible - show all applicable actions
        view_mapping_actions(domain, var, status)
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

/// Detail header with variable name.
fn view_detail_header<'a>(var: &'a tss_standards::SdtmVariable) -> Element<'a, Message> {
    let name = text(&var.name).size(20).color(GRAY_900);
    let label = text(var.label.as_deref().unwrap_or("No label"))
        .size(14)
        .color(GRAY_600);

    column![name, Space::new().height(SPACING_XS), label,].into()
}

/// Variable metadata display.
fn view_variable_metadata<'a>(var: &'a tss_standards::SdtmVariable) -> Element<'a, Message> {
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
        tss_standards::VariableType::Char => "Character",
        tss_standards::VariableType::Num => "Numeric",
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

// =============================================================================
// CONTROLLED TERMINOLOGY SECTION
// =============================================================================

/// Display controlled terminology information for a variable.
fn view_controlled_terminology<'a>(
    state: &'a AppState,
    var: &'a tss_standards::SdtmVariable,
) -> Element<'a, Message> {
    let ct_code = match &var.codelist_code {
        Some(code) => code,
        None => return Space::new().height(0.0).into(),
    };

    // Try to resolve the codelist from the terminology registry
    let resolved = state
        .terminology
        .as_ref()
        .and_then(|reg| reg.resolve(ct_code, None));

    let title_row = row![
        lucide::list().size(14).color(PRIMARY_500),
        Space::new().width(SPACING_SM),
        text("Controlled Terminology").size(14).color(GRAY_700),
    ]
    .align_y(Alignment::Center);

    let content: Element<'a, Message> = if let Some(resolved) = resolved {
        let codelist = resolved.codelist;

        // Codelist header info
        let codelist_name = text(&codelist.name).size(13).color(GRAY_800);
        let codelist_code_text = text(format!("({})", ct_code)).size(11).color(GRAY_500);

        let extensible_badge = if codelist.extensible {
            container(text("Extensible").size(10).color(GRAY_600))
                .padding([2.0, 6.0])
                .style(|_theme: &Theme| container::Style {
                    background: Some(GRAY_200.into()),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
        } else {
            container(text("Non-extensible").size(10).color(ERROR))
                .padding([2.0, 6.0])
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Color::from_rgb(1.0, 0.95, 0.95).into()),
                    border: Border {
                        radius: 4.0.into(),
                        color: ERROR,
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

        // Build terms list (limit to first 10 for space)
        let terms: Vec<_> = codelist.terms.values().collect();
        let show_count = terms.len().min(10);
        let has_more = terms.len() > 10;

        let mut terms_list = column![].spacing(2.0);

        // Header row for the terms table
        let terms_header = row![
            text("Value")
                .size(10)
                .color(GRAY_500)
                .width(Length::Fixed(350.0)),
            text("Meaning").size(10).color(GRAY_500),
        ]
        .padding(SPACING_SM);

        terms_list = terms_list.push(terms_header);
        terms_list = terms_list.push(rule::horizontal(1).style(|_theme| rule::Style {
            color: GRAY_200,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }));

        for term in terms.iter().take(show_count) {
            let meaning = term
                .preferred_term
                .as_deref()
                .unwrap_or(&term.submission_value);

            let term_row = row![
                text(&term.submission_value)
                    .size(12)
                    .color(PRIMARY_500)
                    .width(Length::Fixed(350.0)),
                text(meaning).size(12).color(GRAY_700),
            ]
            .padding(SPACING_SM)
            .align_y(Alignment::Center);

            terms_list = terms_list.push(term_row);
        }

        // Show "and X more..." if truncated
        if has_more {
            let more_text = text(format!("... and {} more values", terms.len() - show_count))
                .size(11)
                .color(GRAY_500);
            terms_list =
                terms_list.push(container(more_text).padding([4.0, 8.0]).width(Length::Fill));
        }

        // Wrap terms in a styled container
        let terms_container = container(terms_list)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(WHITE.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    color: GRAY_200,
                    width: 1.0,
                },
                ..Default::default()
            });

        column![
            header_row,
            Space::new().height(SPACING_SM),
            text("Allowed Values:").size(11).color(GRAY_600),
            Space::new().height(SPACING_XS),
            terms_container,
        ]
        .into()
    } else {
        // Codelist not found in registry
        row![
            lucide::triangle_alert().size(12).color(WARNING),
            Space::new().width(SPACING_SM),
            text(format!("Codelist {} not found in terminology", ct_code))
                .size(12)
                .color(GRAY_600),
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
            .style(|_theme: &Theme| container::Style {
                background: Some(GRAY_100.into()),
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

/// Current mapping status display.
fn view_mapping_status<'a>(
    domain: &'a crate::state::DomainState,
    var: &'a tss_standards::SdtmVariable,
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
            lucide::circle_check().size(16).color(SUCCESS),
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
            lucide::settings().size(16).color(GRAY_600),
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
            lucide::lightbulb().size(16).color(WARNING),
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
                lucide::check().size(12),
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
            lucide::ban().size(16).color(GRAY_500),
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
            lucide::eye_off().size(16).color(GRAY_500),
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
            lucide::circle().size(16).color(GRAY_400),
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

// =============================================================================
// SOURCE COLUMN SELECTION
// =============================================================================

/// A source column option with optional suggestion info for display in picker.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ColumnOption {
    name: String,
    is_suggested: bool,
    confidence: Option<u32>, // Percentage 0-100
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

/// Source column picker using a dropdown with suggestion highlighting.
fn view_source_column_picker<'a>(
    domain: &'a crate::state::DomainState,
    var: &'a tss_standards::SdtmVariable,
) -> Element<'a, Message> {
    use iced::widget::pick_list;

    // Get all source columns
    let source_columns = domain.source.column_names();

    // Get columns that are already mapped (to filter out)
    let mapped_columns: std::collections::BTreeSet<String> = domain
        .mapping
        .all_accepted()
        .values()
        .map(|(col, _)| col.clone())
        .collect();

    // Get suggestion for this variable (if any)
    let suggestion = domain.mapping.suggestion(&var.name);
    let suggested_col: Option<&str> = suggestion.as_ref().map(|(col, _)| col.as_ref());
    let suggested_conf = suggestion.as_ref().map(|(_, conf)| (*conf * 100.0) as u32);

    // Build column options with suggestion info
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

    // Sort: suggested first, then alphabetically
    column_options.sort_by(|a, b| match (a.is_suggested, b.is_suggested) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    if column_options.is_empty() {
        return container(
            row![
                lucide::info().size(14).color(GRAY_500),
                Space::new().width(SPACING_SM),
                text("All source columns are already mapped")
                    .size(13)
                    .color(GRAY_500),
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
        .into();
    }

    let var_name = var.name.clone();
    let column_count = column_options.len();
    let has_suggestion = column_options.iter().any(|c| c.is_suggested);

    // Create the pick_list dropdown
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
    .style(|_theme: &Theme, status| {
        let border_color = match status {
            pick_list::Status::Active => PRIMARY_500,
            pick_list::Status::Hovered => GRAY_400,
            pick_list::Status::Opened { .. } => PRIMARY_500,
        };
        pick_list::Style {
            text_color: GRAY_800,
            placeholder_color: GRAY_500,
            handle_color: GRAY_600,
            background: WHITE.into(),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                width: 1.0,
                color: border_color,
            },
        }
    });

    // Helper text
    let helper_text = if has_suggestion {
        format!(
            "{} columns available - best match shown first",
            column_count
        )
    } else {
        format!("{} columns available", column_count)
    };

    column![
        text("Map to Source Column").size(14).color(GRAY_700),
        Space::new().height(SPACING_SM),
        picker,
        Space::new().height(SPACING_XS),
        text(helper_text).size(11).color(GRAY_500),
    ]
    .into()
}

/// Action buttons for the variable.
fn view_mapping_actions<'a>(
    domain: &'a crate::state::DomainState,
    var: &'a tss_standards::SdtmVariable,
    status: VariableStatus,
) -> Element<'a, Message> {
    let var_name = var.name.clone();
    let title = text("Actions").size(14).color(GRAY_700);

    let mut action_buttons: Vec<Element<'a, Message>> = Vec::new();

    // Clear mapping button (if mapped)
    if matches!(status, VariableStatus::Accepted) {
        let clear_btn = button(
            row![
                lucide::x().size(12),
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

    // Mark Not Collected button (only for Expected variables that are not mapped)
    // User must clear mapping first before marking as Not Collected
    if var.core == Some(CoreDesignation::Expected)
        && !matches!(
            status,
            VariableStatus::Accepted | VariableStatus::NotCollected | VariableStatus::AutoGenerated
        )
    {
        let not_collected_btn = button(
            row![
                lucide::ban().size(12),
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

    // Edit reason + Revert buttons (if NotCollected)
    if matches!(status, VariableStatus::NotCollected) {
        let current_reason = domain
            .mapping
            .not_collected_reason(&var.name)
            .unwrap_or("")
            .to_string();
        let var_name_for_edit = var_name.clone();

        let edit_btn = button(
            row![
                lucide::pencil().size(12),
                Space::new().width(SPACING_XS),
                text("Edit Reason").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::EditNotCollectedReason {
                variable: var_name_for_edit,
                current_reason,
            },
        )))
        .padding([8.0, 16.0])
        .style(button_secondary);

        action_buttons.push(edit_btn.into());

        let revert_btn = button(
            row![
                lucide::undo().size(12),
                Space::new().width(SPACING_XS),
                text("Revert to Mapping").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::ClearNotCollected(var_name.clone()),
        )))
        .padding([8.0, 16.0])
        .style(button_secondary);

        action_buttons.push(revert_btn.into());
    }

    // Mark Omit button (only for Permissible variables that are not mapped)
    // User must clear mapping first before marking as Omitted
    if (var.core.is_none() || var.core == Some(CoreDesignation::Permissible))
        && !matches!(
            status,
            VariableStatus::Accepted | VariableStatus::Omitted | VariableStatus::AutoGenerated
        )
    {
        let omit_btn = button(
            row![
                lucide::eye_off().size(12),
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
                lucide::eye().size(12),
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

    // If no actions available, don't render the section at all
    if action_buttons.is_empty() {
        return Space::new().height(0.0).into();
    }

    let mut actions_col = column![].spacing(SPACING_SM);
    for btn in action_buttons {
        actions_col = actions_col.push(btn);
    }

    column![title, Space::new().height(SPACING_SM), actions_col,].into()
}

// =============================================================================
// NOT COLLECTED INLINE EDITING
// =============================================================================

/// Inline editor for "Not Collected" reason.
fn view_not_collected_inline_edit<'a>(
    var: &'a tss_standards::SdtmVariable,
    edit: &'a NotCollectedEdit,
) -> Element<'a, Message> {
    let var_name = var.name.clone();
    let reason = edit.reason.clone();
    let reason_for_save = reason.clone();

    let title = text("Not Collected Reason").size(14).color(GRAY_700);

    let char_count = reason.len();
    let max_len = 200;
    let is_over = char_count > max_len;
    let is_empty = reason.trim().is_empty();

    // Character counter
    let count_display = format!("{}/{}", char_count, max_len);

    // Reason input field
    let reason_input = text_input("Enter reason why data was not collected...", &reason)
        .on_input(|s| {
            Message::DomainEditor(DomainEditorMessage::Mapping(
                MappingMessage::NotCollectedReasonChanged(s),
            ))
        })
        .padding([10.0, 12.0])
        .size(14)
        .style(move |_: &Theme, _status| {
            let border_color = if is_over || is_empty { ERROR } else { GRAY_200 };
            iced::widget::text_input::Style {
                background: WHITE.into(),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: BORDER_RADIUS_SM.into(),
                },
                icon: GRAY_500,
                placeholder: GRAY_400,
                value: GRAY_900,
                selection: PRIMARY_100,
            }
        });

    // Error message
    let error_msg: Element<'a, Message> = if is_empty {
        text("Reason is required").size(11).color(ERROR).into()
    } else if is_over {
        text("Reason too long").size(11).color(ERROR).into()
    } else {
        Space::new().height(0.0).into()
    };

    // Save button (disabled if empty or over limit)
    let can_save = !is_empty && !is_over;
    let save_btn = button(
        row![
            lucide::check().size(12),
            Space::new().width(SPACING_XS),
            text("Save").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press_maybe(if can_save {
        Some(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::NotCollectedSave {
                variable: var_name.clone(),
                reason: reason_for_save,
            },
        )))
    } else {
        None
    })
    .padding([8.0, 16.0])
    .style(button_primary);

    // Cancel button
    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::NotCollectedCancel,
        )))
        .padding([8.0, 16.0])
        .style(button_secondary);

    column![
        title,
        Space::new().height(SPACING_SM),
        row![
            text("Reason *").size(12).color(GRAY_600),
            Space::new().width(Length::Fill),
            text(count_display)
                .size(11)
                .color(if is_over { ERROR } else { GRAY_400 }),
        ],
        Space::new().height(4.0),
        reason_input,
        error_msg,
        Space::new().height(SPACING_MD),
        row![save_btn, Space::new().width(SPACING_SM), cancel_btn,],
    ]
    .into()
}

// =============================================================================
// EMPTY STATE
// =============================================================================

/// Empty state when no variable is selected.
fn view_no_selection<'a>() -> Element<'a, Message> {
    container(
        column![
            lucide::mouse_pointer_click().size(48).color(GRAY_400),
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
