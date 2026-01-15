//! Mapping tab view.
//!
//! The mapping tab displays a master-detail interface for mapping
//! source columns to SDTM target variables.
//!
//! - **Left (Master)**: List of TARGET SDTM variables with status indicators
//! - **Right (Detail)**: Selected variable details with mapping controls

use iced::widget::{
    Space, button, column, container, pick_list, row, rule, scrollable, text, text_input,
};
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::{
    ActionButton, ActionButtonList, DetailHeader, EmptyState, FilterToggle, MetadataCard,
    StatusCard, VariableListItem, master_detail_with_pinned_header,
};
use crate::message::domain_editor::MappingMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, DomainState, MappingUiState, NotCollectedEdit, ViewState};
use crate::theme::{
    BORDER_RADIUS_SM, ERROR, GRAY_100, GRAY_200, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800,
    GRAY_900, MASTER_WIDTH, PRIMARY_100, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM,
    SPACING_XS, SUCCESS, WARNING, WHITE, button_primary, button_secondary,
};

use tss_standards::CoreDesignation;
use tss_submit::VariableStatus;

// =============================================================================
// MAIN MAPPING TAB VIEW
// =============================================================================

/// Render the mapping tab content using master-detail layout.
pub fn view_mapping_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return EmptyState::new(
                lucide::circle_alert().size(48).color(GRAY_400),
                "Domain not found",
            )
            .centered()
            .view();
        }
    };

    let mapping_ui = match &state.view {
        ViewState::DomainEditor { mapping_ui, .. } => mapping_ui,
        _ => return text("Invalid view state").into(),
    };

    let sdtm_domain = domain.mapping.domain();

    // Apply filters
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

    let master_header = view_variable_list_header(domain, mapping_ui);
    let master_content = view_variable_list_content(domain, &filtered_indices, mapping_ui);
    let detail = if let Some(selected_idx) = mapping_ui.selected_variable {
        if let Some(var) = sdtm_domain.variables.get(selected_idx) {
            view_variable_detail(state, domain, var)
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

fn view_variable_list_header<'a>(
    domain: &'a DomainState,
    mapping_ui: &'a MappingUiState,
) -> Element<'a, Message> {
    let summary = domain.summary();

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

    // Filter toggles
    let filter_unmapped = mapping_ui.filter_unmapped;
    let filter_required = mapping_ui.filter_required;

    let unmapped_btn = FilterToggle::new(
        "Unmapped",
        filter_unmapped,
        Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::FilterUnmappedToggled(!filter_unmapped),
        )),
    )
    .view();

    let required_btn = FilterToggle::new(
        "Required",
        filter_required,
        Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::FilterRequiredToggled(!filter_required),
        )),
    )
    .view();

    // Stats
    let stats = row![
        text(format!("{}/{}", summary.mapped, summary.total_variables))
            .size(12)
            .color(GRAY_600),
        Space::new().width(4.0),
        text("mapped").size(11).color(GRAY_500),
    ]
    .align_y(Alignment::Center);

    column![
        search_input,
        Space::new().height(SPACING_XS),
        row![unmapped_btn, required_btn].spacing(SPACING_XS),
        Space::new().height(SPACING_SM),
        stats,
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
    ]
    .into()
}

fn view_variable_list_content<'a>(
    domain: &'a DomainState,
    filtered_indices: &[usize],
    mapping_ui: &'a MappingUiState,
) -> Element<'a, Message> {
    let sdtm_domain = domain.mapping.domain();

    if filtered_indices.is_empty() {
        return container(
            column![
                text("No variables match your filters")
                    .size(13)
                    .color(GRAY_500),
                Space::new().height(SPACING_SM),
                button(text("Clear filters").size(12))
                    .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
                        MappingMessage::SearchCleared
                    )))
                    .padding([6.0, 12.0])
                    .style(button_secondary),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(SPACING_LG)
        .center_x(Length::Shrink)
        .into();
    }

    let mut items = column![].spacing(SPACING_XS);
    for &idx in filtered_indices {
        if let Some(var) = sdtm_domain.variables.get(idx) {
            let status = domain.mapping.status(&var.name);
            let is_selected = mapping_ui.selected_variable == Some(idx);
            items = items.push(view_variable_item(idx, var, status, is_selected));
        }
    }
    items.into()
}

fn view_variable_item<'a>(
    index: usize,
    var: &'a tss_standards::SdtmVariable,
    status: VariableStatus,
    is_selected: bool,
) -> Element<'a, Message> {
    // Status icon
    let status_icon: Element<'a, Message> = match status {
        VariableStatus::Accepted => lucide::circle_check().size(12).color(SUCCESS).into(),
        VariableStatus::AutoGenerated => lucide::settings().size(12).color(GRAY_500).into(),
        VariableStatus::Suggested => lucide::lightbulb().size(12).color(WARNING).into(),
        VariableStatus::NotCollected => lucide::ban().size(12).color(GRAY_400).into(),
        VariableStatus::Omitted => lucide::eye_off().size(12).color(GRAY_400).into(),
        VariableStatus::Unmapped => lucide::circle().size(12).color(GRAY_400).into(),
    };

    // Core badge
    let (badge_text, badge_color) = match var.core {
        Some(CoreDesignation::Required) => (Some("Req"), Some(ERROR)),
        Some(CoreDesignation::Expected) => (Some("Exp"), Some(WARNING)),
        Some(CoreDesignation::Permissible) => (Some("Perm"), Some(GRAY_500)),
        None => (None, None),
    };

    let mut item = VariableListItem::new(
        &var.name,
        Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::VariableSelected(index),
        )),
    )
    .leading_icon(status_icon)
    .selected(is_selected);

    if let Some(label) = &var.label {
        item = item.label(label);
    }

    if let (Some(txt), Some(col)) = (badge_text, badge_color) {
        item = item.trailing_badge(txt, col);
    }

    item.view()
}

// =============================================================================
// DETAIL PANEL
// =============================================================================

fn view_variable_detail<'a>(
    state: &'a AppState,
    domain: &'a DomainState,
    var: &'a tss_standards::SdtmVariable,
) -> Element<'a, Message> {
    let status = domain.mapping.status(&var.name);
    let not_collected_edit = match &state.view {
        ViewState::DomainEditor { mapping_ui, .. } => mapping_ui.not_collected_edit.as_ref(),
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
    let mapping_status = view_mapping_status(domain, var, status);
    let source_picker: Element<'a, Message> =
        if matches!(status, VariableStatus::Unmapped | VariableStatus::Suggested) {
            view_source_column_picker(domain, var)
        } else {
            Space::new().height(0.0).into()
        };

    let is_editing_not_collected = not_collected_edit
        .map(|e| e.variable == var.name)
        .unwrap_or(false);
    let is_required = var.core == Some(CoreDesignation::Required);

    let actions: Element<'a, Message> = if is_editing_not_collected {
        view_not_collected_inline_edit(var, not_collected_edit.unwrap())
    } else if is_required && !matches!(status, VariableStatus::Accepted) {
        Space::new().height(0.0).into()
    } else {
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
    domain: &'a DomainState,
    var: &'a tss_standards::SdtmVariable,
    status: VariableStatus,
) -> Element<'a, Message> {
    let title = text("Mapping Status").size(14).color(GRAY_700);

    let status_content: Element<'a, Message> = match status {
        VariableStatus::Accepted => {
            if let Some((col, conf)) = domain.mapping.accepted(&var.name) {
                let conf_pct = (conf * 100.0) as u32;
                StatusCard::new(lucide::circle_check().size(16).color(SUCCESS))
                    .title("Mapped to:")
                    .value(col)
                    .description(format!("{}% confidence", conf_pct))
                    .background(Color::from_rgb(0.9, 0.98, 0.92))
                    .border_color(SUCCESS)
                    .view()
            } else {
                view_status_unmapped()
            }
        }
        VariableStatus::AutoGenerated => {
            StatusCard::new(lucide::settings().size(16).color(GRAY_600))
                .value("Auto-generated")
                .description("This variable is populated automatically by the system")
                .view()
        }
        VariableStatus::Suggested => {
            if let Some((col, conf)) = domain.mapping.suggestion(&var.name) {
                let conf_pct = (conf * 100.0) as u32;
                let var_name = var.name.clone();
                StatusCard::new(lucide::lightbulb().size(16).color(WARNING))
                    .title("Suggested mapping:")
                    .value(col)
                    .description(format!("{}% confidence", conf_pct))
                    .action(
                        "Accept Suggestion",
                        Message::DomainEditor(DomainEditorMessage::Mapping(
                            MappingMessage::AcceptSuggestion(var_name),
                        )),
                    )
                    .background(Color::from_rgb(1.0, 0.98, 0.9))
                    .border_color(WARNING)
                    .view()
            } else {
                view_status_unmapped()
            }
        }
        VariableStatus::NotCollected => {
            let reason = domain
                .mapping
                .not_collected_reason(&var.name)
                .unwrap_or("No reason provided");
            StatusCard::new(lucide::ban().size(16).color(GRAY_500))
                .value("Not Collected")
                .description(reason)
                .view()
        }
        VariableStatus::Omitted => StatusCard::new(lucide::eye_off().size(16).color(GRAY_500))
            .value("Omitted")
            .description("This permissible variable will not be included in output")
            .view(),
        VariableStatus::Unmapped => view_status_unmapped(),
    };

    column![title, Space::new().height(SPACING_SM), status_content].into()
}

fn view_status_unmapped<'a, M: Clone + 'a>() -> Element<'a, M> {
    StatusCard::new(lucide::circle().size(16).color(GRAY_400))
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
        lucide::list().size(14).color(PRIMARY_500),
        Space::new().width(SPACING_SM),
        text("Controlled Terminology").size(14).color(GRAY_700),
    ]
    .align_y(Alignment::Center);

    let content: Element<'a, Message> = if let Some(resolved) = resolved {
        let codelist = resolved.codelist;
        let codelist_name = text(&codelist.name).size(13).color(GRAY_800);
        let codelist_code_text = text(format!("({})", ct_code)).size(11).color(GRAY_500);

        let extensible_badge = if codelist.extensible {
            container(text("Extensible").size(10).color(GRAY_600))
                .padding([2.0, 6.0])
                .style(|_: &Theme| container::Style {
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
                .style(|_: &Theme| container::Style {
                    background: Some(Color::from_rgb(1.0, 0.95, 0.95).into()),
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

        let terms: Vec<_> = codelist.terms.values().collect();
        let show_count = terms.len().min(10);
        let has_more = terms.len() > 10;

        let mut terms_list = column![].spacing(2.0);
        let terms_header = row![
            text("Value")
                .size(10)
                .color(GRAY_500)
                .width(Length::Fixed(350.0)),
            text("Meaning").size(10).color(GRAY_500),
        ]
        .padding(SPACING_SM);

        terms_list = terms_list.push(terms_header);
        terms_list = terms_list.push(rule::horizontal(1).style(|_| rule::Style {
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

        if has_more {
            terms_list = terms_list.push(
                container(
                    text(format!("... and {} more values", terms.len() - show_count))
                        .size(11)
                        .color(GRAY_500),
                )
                .padding([4.0, 8.0])
                .width(Length::Fill),
            );
        }

        let terms_container = container(terms_list)
            .width(Length::Fill)
            .style(|_: &Theme| container::Style {
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
            .style(|_: &Theme| container::Style {
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
    domain: &'a DomainState,
    var: &'a tss_standards::SdtmVariable,
) -> Element<'a, Message> {
    let source_columns = domain.source.column_names();
    let mapped_columns: std::collections::BTreeSet<String> = domain
        .mapping
        .all_accepted()
        .values()
        .map(|(col, _)| col.clone())
        .collect();

    let suggestion = domain.mapping.suggestion(&var.name);
    let suggested_col: Option<&str> = suggestion.as_ref().map(|(col, _)| col.as_ref());
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
        .style(|_: &Theme| container::Style {
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
    .style(|_: &Theme, status| {
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

// =============================================================================
// ACTIONS
// =============================================================================

fn view_mapping_actions<'a>(
    domain: &'a DomainState,
    var: &'a tss_standards::SdtmVariable,
    status: VariableStatus,
) -> Element<'a, Message> {
    let var_name = var.name.clone();
    let mut actions = ActionButtonList::new().title("Actions");

    // Clear mapping (if mapped)
    if matches!(status, VariableStatus::Accepted) {
        actions = actions.button(ActionButton::secondary(
            lucide::x().size(12),
            "Clear Mapping",
            Message::DomainEditor(DomainEditorMessage::Mapping(MappingMessage::ClearMapping(
                var_name.clone(),
            ))),
        ));
    }

    // Mark Not Collected (Expected variables that are not mapped)
    if var.core == Some(CoreDesignation::Expected)
        && !matches!(
            status,
            VariableStatus::Accepted | VariableStatus::NotCollected | VariableStatus::AutoGenerated
        )
    {
        actions = actions.button(ActionButton::secondary(
            lucide::ban().size(12),
            "Mark Not Collected",
            Message::DomainEditor(DomainEditorMessage::Mapping(
                MappingMessage::MarkNotCollected {
                    variable: var_name.clone(),
                },
            )),
        ));
    }

    // Edit reason + Revert (if NotCollected)
    if matches!(status, VariableStatus::NotCollected) {
        let current_reason = domain
            .mapping
            .not_collected_reason(&var.name)
            .unwrap_or("")
            .to_string();
        actions = actions.button(ActionButton::secondary(
            lucide::pencil().size(12),
            "Edit Reason",
            Message::DomainEditor(DomainEditorMessage::Mapping(
                MappingMessage::EditNotCollectedReason {
                    variable: var_name.clone(),
                    current_reason,
                },
            )),
        ));
        actions = actions.button(ActionButton::secondary(
            lucide::undo().size(12),
            "Revert to Mapping",
            Message::DomainEditor(DomainEditorMessage::Mapping(
                MappingMessage::ClearNotCollected(var_name.clone()),
            )),
        ));
    }

    // Mark Omit (Permissible variables)
    if (var.core.is_none() || var.core == Some(CoreDesignation::Permissible))
        && !matches!(
            status,
            VariableStatus::Accepted | VariableStatus::Omitted | VariableStatus::AutoGenerated
        )
    {
        actions = actions.button(ActionButton::secondary(
            lucide::eye_off().size(12),
            "Omit from Output",
            Message::DomainEditor(DomainEditorMessage::Mapping(MappingMessage::MarkOmitted(
                var_name.clone(),
            ))),
        ));
    }

    // Clear Omit (if omitted)
    if matches!(status, VariableStatus::Omitted) {
        actions = actions.button(ActionButton::secondary(
            lucide::eye().size(12),
            "Include in Output",
            Message::DomainEditor(DomainEditorMessage::Mapping(MappingMessage::ClearOmitted(
                var_name,
            ))),
        ));
    }

    actions.view_or_empty()
}

// =============================================================================
// NOT COLLECTED INLINE EDIT
// =============================================================================

fn view_not_collected_inline_edit<'a>(
    var: &'a tss_standards::SdtmVariable,
    edit: &'a NotCollectedEdit,
) -> Element<'a, Message> {
    let var_name = var.name.clone();
    let reason = edit.reason.clone();
    let reason_for_save = reason.clone();

    let char_count = reason.len();
    let max_len = 200;
    let is_over = char_count > max_len;
    let is_empty = reason.trim().is_empty();
    let can_save = !is_empty && !is_over;

    let reason_input = text_input("Enter reason why data was not collected...", &reason)
        .on_input(|s| {
            Message::DomainEditor(DomainEditorMessage::Mapping(
                MappingMessage::NotCollectedReasonChanged(s),
            ))
        })
        .padding([10.0, 12.0])
        .size(14)
        .style(move |_: &Theme, _| {
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

    let error_msg: Element<'a, Message> = if is_empty {
        text("Reason is required").size(11).color(ERROR).into()
    } else if is_over {
        text("Reason too long").size(11).color(ERROR).into()
    } else {
        Space::new().height(0.0).into()
    };

    let save_btn = button(
        row![
            lucide::check().size(12),
            Space::new().width(SPACING_XS),
            text("Save").size(13)
        ]
        .align_y(Alignment::Center),
    )
    .on_press_maybe(if can_save {
        Some(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::NotCollectedSave {
                variable: var_name,
                reason: reason_for_save,
            },
        )))
    } else {
        None
    })
    .padding([8.0, 16.0])
    .style(button_primary);

    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::DomainEditor(DomainEditorMessage::Mapping(
            MappingMessage::NotCollectedCancel,
        )))
        .padding([8.0, 16.0])
        .style(button_secondary);

    column![
        text("Not Collected Reason").size(14).color(GRAY_700),
        Space::new().height(SPACING_SM),
        row![
            text("Reason *").size(12).color(GRAY_600),
            Space::new().width(Length::Fill),
            text(format!("{}/{}", char_count, max_len))
                .size(11)
                .color(if is_over { ERROR } else { GRAY_400 }),
        ],
        Space::new().height(4.0),
        reason_input,
        error_msg,
        Space::new().height(SPACING_MD),
        row![save_btn, Space::new().width(SPACING_SM), cancel_btn],
    ]
    .into()
}

// =============================================================================
// EMPTY STATE
// =============================================================================

fn view_no_selection<'a>() -> Element<'a, Message> {
    EmptyState::new(
        lucide::mouse_pointer_click().size(48).color(GRAY_400),
        "Select a Variable",
    )
    .description("Click a variable from the list to view details and configure mapping")
    .centered()
    .view()
}
