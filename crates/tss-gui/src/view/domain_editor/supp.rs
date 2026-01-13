//! SUPP (Supplemental Qualifiers) tab view.
//!
//! # Architecture
//!
//! The SUPP tab uses a clean state-based UX:
//!
//! - **Pending**: Editable fields + sample data + "Add to SUPP"/"Skip" buttons
//! - **Included (view)**: Read-only summary + "Edit"/"Remove" options
//! - **Included (edit)**: Editable fields + "Save"/"Cancel" buttons
//! - **Skipped**: Skip message + sample data + "Add to SUPP instead" button
//!
//! # Edit Draft Pattern
//!
//! For pending columns, edits go directly to `supp_config`.
//! For included columns in edit mode, edits go to `edit_draft` and are
//! committed only on "Save".

use iced::widget::{
    Space, button, column, container, pick_list, row, rule, scrollable, text, text_input,
};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;
use polars::prelude::AnyValue;

use crate::component::master_detail_with_pinned_header;
use crate::message::domain_editor::SuppMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{
    AppState, Domain, SuppAction, SuppColumnConfig, SuppEditDraft, SuppFilterMode, SuppOrigin,
    SuppUiState, ViewState,
};
use crate::theme::{
    BORDER_RADIUS_SM, ERROR, GRAY_100, GRAY_300, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800,
    GRAY_900, PRIMARY_100, PRIMARY_500, PRIMARY_600, PRIMARY_700, SPACING_LG, SPACING_MD,
    SPACING_SM, SPACING_XL, SPACING_XS, SUCCESS, WARNING, WHITE,
};

// =============================================================================
// CONSTANTS
// =============================================================================

const MASTER_WIDTH: f32 = 300.0;
const QNAM_MAX_LEN: usize = 8;
const QLABEL_MAX_LEN: usize = 40;

// =============================================================================
// MAIN SUPP TAB VIEW
// =============================================================================

/// Render the SUPP configuration tab content.
pub fn view_supp_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
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
    let supp_ui = match &state.view {
        ViewState::DomainEditor { supp_ui, .. } => supp_ui,
        _ => return text("Invalid view state").into(),
    };

    // Get unmapped columns
    let unmapped_columns = domain.unmapped_columns();

    // If no unmapped columns, show success state
    if unmapped_columns.is_empty() {
        return view_all_mapped_state(domain_code);
    }

    // Filter columns based on search and filter mode
    let filtered: Vec<String> = unmapped_columns
        .iter()
        .filter(|col| {
            // Search filter
            if !supp_ui.search_filter.is_empty()
                && !col
                    .to_lowercase()
                    .contains(&supp_ui.search_filter.to_lowercase())
            {
                return false;
            }

            // Action filter
            let config = domain.supp_config.get(*col);
            match supp_ui.filter_mode {
                SuppFilterMode::All => true,
                SuppFilterMode::Pending => config.map_or(true, |c| c.action == SuppAction::Pending),
                SuppFilterMode::Included => {
                    config.map_or(false, |c| c.action == SuppAction::Include)
                }
                SuppFilterMode::Skipped => config.map_or(false, |c| c.action == SuppAction::Skip),
            }
        })
        .cloned()
        .collect();

    // Build master header (pinned at top)
    let master_header = build_master_header_pinned(domain_code, supp_ui, filtered.len());

    // Build master content (scrollable column list)
    let master_content = build_master_content(&filtered, domain, supp_ui);

    // Build detail panel
    let detail = build_detail_panel(domain, supp_ui, domain_code);

    // Use master-detail layout with pinned header
    master_detail_with_pinned_header(master_header, master_content, detail, MASTER_WIDTH)
}

// =============================================================================
// MASTER PANEL: HEADER (PINNED)
// =============================================================================

/// Left panel header: title, search, filters, and stats (pinned at top).
fn build_master_header_pinned<'a>(
    domain_code: &'a str,
    ui: &'a SuppUiState,
    filtered_count: usize,
) -> Element<'a, Message> {
    let title = format!("SUPP{}", domain_code);

    // Title row
    let title_row = text(title).size(16).color(GRAY_900).font(iced::Font {
        weight: iced::font::Weight::Semibold,
        ..Default::default()
    });

    // Search box
    let search = text_input("Search columns...", &ui.search_filter)
        .on_input(|s| {
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::SearchChanged(s)))
        })
        .padding([8.0, 12.0])
        .size(13);

    // Filter buttons
    let filters = build_filter_buttons(ui.filter_mode);

    // Stats
    let stats = row![
        text(format!("{}", filtered_count)).size(12).color(GRAY_600),
        Space::new().width(4.0),
        text("columns").size(11).color(GRAY_500),
    ]
    .align_y(Alignment::Center);

    column![
        title_row,
        Space::new().height(SPACING_SM),
        search,
        Space::new().height(SPACING_SM),
        filters,
        Space::new().height(SPACING_SM),
        stats,
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
    ]
    .into()
}

// =============================================================================
// MASTER PANEL: CONTENT (SCROLLABLE)
// =============================================================================

/// Left panel content: scrollable list of columns.
fn build_master_content<'a>(
    filtered: &[String],
    domain: &'a Domain,
    ui: &'a SuppUiState,
) -> Element<'a, Message> {
    if filtered.is_empty() {
        return container(
            column![
                lucide::search_x().size(32).color(GRAY_400),
                Space::new().height(SPACING_SM),
                text("No columns match filter").size(13).color(GRAY_500),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(SPACING_LG)
        .center_x(Length::Shrink)
        .into();
    }

    // Build column items
    let mut items = column![].spacing(SPACING_XS);

    for col_name in filtered {
        let config = domain.supp_config.get(col_name);
        let action = config.map_or(SuppAction::Pending, |c| c.action);
        let is_selected = ui.selected_column.as_deref() == Some(col_name.as_str());
        let item = build_column_item(col_name.clone(), action, is_selected);
        items = items.push(item);
    }

    items.into()
}

fn build_filter_buttons(current: SuppFilterMode) -> Element<'static, Message> {
    let filters = [
        (SuppFilterMode::All, "All"),
        (SuppFilterMode::Pending, "Pending"),
        (SuppFilterMode::Included, "SUPP"),
        (SuppFilterMode::Skipped, "Skip"),
    ];

    let buttons: Vec<Element<'static, Message>> = filters
        .iter()
        .map(|(mode, label)| {
            let is_selected = current == *mode;
            let mode_val = *mode;
            let label_str = *label;

            button(
                text(label_str)
                    .size(11)
                    .color(if is_selected { PRIMARY_500 } else { GRAY_600 }),
            )
            .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
                SuppMessage::FilterModeChanged(mode_val),
            )))
            .padding([4.0, 8.0])
            .style(move |_: &Theme, _status| {
                if is_selected {
                    iced::widget::button::Style {
                        background: Some(PRIMARY_100.into()),
                        text_color: PRIMARY_500,
                        border: Border {
                            color: PRIMARY_500,
                            width: 1.0,
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        ..Default::default()
                    }
                } else {
                    iced::widget::button::Style {
                        background: Some(WHITE.into()),
                        text_color: GRAY_600,
                        border: Border {
                            color: GRAY_300,
                            width: 1.0,
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        ..Default::default()
                    }
                }
            })
            .into()
        })
        .collect();

    row(buttons).spacing(4.0).into()
}

fn build_column_item(
    col_name: String,
    action: SuppAction,
    is_selected: bool,
) -> Element<'static, Message> {
    let status_icon: Element<'static, Message> = match action {
        SuppAction::Pending => lucide::circle().size(10).color(GRAY_400).into(),
        SuppAction::Include => lucide::circle_check().size(10).color(SUCCESS).into(),
        SuppAction::Skip => lucide::circle_x().size(10).color(GRAY_400).into(),
    };

    let bg_color = if is_selected { PRIMARY_100 } else { WHITE };
    let text_color = if is_selected { PRIMARY_500 } else { GRAY_800 };
    let display_name = col_name.clone();

    button(
        row![
            status_icon,
            Space::new().width(SPACING_SM),
            text(display_name).size(13).color(text_color),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
        SuppMessage::ColumnSelected(col_name),
    )))
    .padding([8.0, 12.0])
    .width(Length::Fill)
    .style(move |_: &Theme, _status| iced::widget::button::Style {
        background: Some(bg_color.into()),
        text_color,
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

// =============================================================================
// DETAIL PANEL
// =============================================================================

fn build_detail_panel(
    domain: &Domain,
    ui: &SuppUiState,
    domain_code: &str,
) -> Element<'static, Message> {
    match &ui.selected_column {
        Some(col) => {
            let config = domain
                .supp_config
                .get(col)
                .cloned()
                .unwrap_or_else(|| SuppColumnConfig::from_column(col));

            let is_editing = ui.edit_draft.is_some();

            match config.action {
                SuppAction::Pending => build_pending_view(domain, col, &config, domain_code),
                SuppAction::Include if is_editing => {
                    build_edit_view(domain, col, ui.edit_draft.as_ref().unwrap(), domain_code)
                }
                SuppAction::Include => build_included_view(domain, col, &config, domain_code),
                SuppAction::Skip => build_skipped_view(domain, col, domain_code),
            }
        }
        None => build_no_selection_state(),
    }
}

fn build_no_selection_state() -> Element<'static, Message> {
    container(
        column![
            lucide::mouse_pointer_click().size(48).color(GRAY_400),
            Space::new().height(SPACING_LG),
            text("Select a Column").size(18).color(GRAY_700),
            Space::new().height(SPACING_SM),
            text("Click on a column in the list to configure its SUPP settings")
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
// PENDING VIEW - Editable fields + Add/Skip buttons
// =============================================================================

fn build_pending_view(
    domain: &Domain,
    col_name: &str,
    config: &SuppColumnConfig,
    domain_code: &str,
) -> Element<'static, Message> {
    let header = build_detail_header(col_name, domain_code);
    let sample_data = build_sample_data(domain, col_name);

    // Check QNAM conflict (only against included columns)
    let qnam_error = check_qnam_conflict(domain, col_name, &config.qnam);

    // Editable fields
    let fields = build_editable_fields(config.clone(), qnam_error);

    // Action buttons
    let actions = build_pending_actions(domain_code);

    scrollable(
        column![
            header,
            Space::new().height(SPACING_LG),
            sample_data,
            Space::new().height(SPACING_LG),
            fields,
            Space::new().height(SPACING_XL),
            actions,
        ]
        .padding(SPACING_LG)
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn build_pending_actions(domain_code: &str) -> Element<'static, Message> {
    let supp_name = format!("SUPP{}", domain_code);

    column![
        // Primary action - Add to SUPP
        button(
            row![
                lucide::plus().size(16).color(WHITE),
                Space::new().width(SPACING_SM),
                text(format!("Add to {}", supp_name)).size(14).color(WHITE),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
            SuppMessage::AddToSupp,
        )))
        .padding([12.0, 24.0])
        .width(Length::Fill)
        .style(|_: &Theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => PRIMARY_600,
                iced::widget::button::Status::Pressed => PRIMARY_700,
                _ => PRIMARY_500,
            };
            iced::widget::button::Style {
                background: Some(bg.into()),
                text_color: WHITE,
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }),
        Space::new().height(SPACING_MD),
        // Secondary action - Skip
        container(
            button(
                row![
                    lucide::x().size(14).color(GRAY_500),
                    Space::new().width(SPACING_XS),
                    text("Skip this column").size(13).color(GRAY_500),
                ]
                .align_y(Alignment::Center),
            )
            .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
                SuppMessage::Skip,
            )))
            .padding([8.0, 16.0])
            .style(|_: &Theme, status| {
                let text_color = match status {
                    iced::widget::button::Status::Hovered => GRAY_700,
                    _ => GRAY_500,
                };
                iced::widget::button::Style {
                    background: None,
                    text_color,
                    ..Default::default()
                }
            }),
        )
        .width(Length::Fill)
        .center_x(Length::Shrink),
    ]
    .into()
}

// =============================================================================
// INCLUDED VIEW (Read-only) - Summary + Edit/Remove buttons
// =============================================================================

fn build_included_view(
    domain: &Domain,
    col_name: &str,
    config: &SuppColumnConfig,
    domain_code: &str,
) -> Element<'static, Message> {
    let header = build_detail_header(col_name, domain_code);
    let sample_data = build_sample_data(domain, col_name);

    // Read-only summary
    let summary = build_readonly_summary(config);

    // Actions
    let actions = build_included_actions();

    scrollable(
        column![
            header,
            Space::new().height(SPACING_LG),
            sample_data,
            Space::new().height(SPACING_LG),
            summary,
            Space::new().height(SPACING_XL),
            actions,
        ]
        .padding(SPACING_LG)
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn build_readonly_summary(config: &SuppColumnConfig) -> Element<'static, Message> {
    let qnam = config.qnam.clone();
    let qlabel = config.qlabel.clone();
    let qorig = config.qorig.label().to_string();
    let qeval = config.qeval.clone().unwrap_or_else(|| "—".to_string());

    container(
        column![
            // Success indicator
            row![
                lucide::circle_check().size(16).color(SUCCESS),
                Space::new().width(SPACING_SM),
                text("Added to SUPP")
                    .size(13)
                    .color(SUCCESS)
                    .font(iced::Font {
                        weight: iced::font::Weight::Medium,
                        ..Default::default()
                    }),
            ]
            .align_y(Alignment::Center),
            Space::new().height(SPACING_MD),
            // Fields summary
            build_summary_row("QNAM", &qnam),
            Space::new().height(SPACING_SM),
            build_summary_row("QLABEL", &qlabel),
            Space::new().height(SPACING_SM),
            build_summary_row("QORIG", &qorig),
            Space::new().height(SPACING_SM),
            build_summary_row("QEVAL", &qeval),
        ]
        .padding(SPACING_MD),
    )
    .style(|_: &Theme| container::Style {
        background: Some(iced::Color::from_rgb(0.95, 0.99, 0.96).into()),
        border: Border {
            color: SUCCESS,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

fn build_summary_row(label: &str, value: &str) -> Element<'static, Message> {
    let label_str = label.to_string();
    let value_str = value.to_string();

    row![
        text(label_str)
            .size(12)
            .color(GRAY_600)
            .width(Length::Fixed(60.0)),
        text(value_str).size(13).color(GRAY_800).font(iced::Font {
            weight: iced::font::Weight::Medium,
            ..Default::default()
        }),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn build_included_actions() -> Element<'static, Message> {
    row![
        // Edit button
        button(
            row![
                lucide::pencil().size(14).color(PRIMARY_500),
                Space::new().width(SPACING_XS),
                text("Edit").size(13).color(PRIMARY_500),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
            SuppMessage::StartEdit,
        )))
        .padding([8.0, 16.0])
        .style(|_: &Theme, status| {
            let text_color = match status {
                iced::widget::button::Status::Hovered => PRIMARY_700,
                _ => PRIMARY_500,
            };
            iced::widget::button::Style {
                background: Some(PRIMARY_100.into()),
                text_color,
                border: Border {
                    color: PRIMARY_500,
                    width: 1.0,
                    radius: BORDER_RADIUS_SM.into(),
                },
                ..Default::default()
            }
        }),
        Space::new().width(SPACING_MD),
        // Remove button
        button(
            row![
                lucide::trash().size(14).color(GRAY_500),
                Space::new().width(SPACING_XS),
                text("Remove").size(13).color(GRAY_500),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
            SuppMessage::UndoAction,
        )))
        .padding([8.0, 16.0])
        .style(|_: &Theme, status| {
            let text_color = match status {
                iced::widget::button::Status::Hovered => GRAY_700,
                _ => GRAY_500,
            };
            iced::widget::button::Style {
                background: None,
                text_color,
                border: Border {
                    color: GRAY_300,
                    width: 1.0,
                    radius: BORDER_RADIUS_SM.into(),
                },
                ..Default::default()
            }
        }),
    ]
    .into()
}

// =============================================================================
// EDIT VIEW (for included columns) - Editable fields + Save/Cancel
// =============================================================================

fn build_edit_view(
    domain: &Domain,
    col_name: &str,
    draft: &SuppEditDraft,
    domain_code: &str,
) -> Element<'static, Message> {
    let header = build_detail_header(col_name, domain_code);
    let sample_data = build_sample_data(domain, col_name);

    // Create a temporary config from draft for display
    let temp_config = SuppColumnConfig {
        column: col_name.to_string(),
        qnam: draft.qnam.clone(),
        qlabel: draft.qlabel.clone(),
        qorig: draft.qorig,
        qeval: if draft.qeval.is_empty() {
            None
        } else {
            Some(draft.qeval.clone())
        },
        action: SuppAction::Include,
    };

    // Check QNAM conflict
    let qnam_error = check_qnam_conflict(domain, col_name, &draft.qnam);

    // Editable fields
    let fields = build_editable_fields(temp_config, qnam_error);

    // Edit mode info
    let edit_info = container(
        row![
            lucide::info().size(14).color(WARNING),
            Space::new().width(SPACING_SM),
            text("Editing — changes will be saved when you click Save")
                .size(12)
                .color(GRAY_600),
        ]
        .align_y(Alignment::Center),
    )
    .padding([SPACING_SM, SPACING_MD])
    .style(|_: &Theme| container::Style {
        background: Some(iced::Color::from_rgb(1.0, 0.98, 0.92).into()),
        border: Border {
            color: WARNING,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill);

    // Actions
    let actions = build_edit_actions();

    scrollable(
        column![
            header,
            Space::new().height(SPACING_MD),
            edit_info,
            Space::new().height(SPACING_LG),
            sample_data,
            Space::new().height(SPACING_LG),
            fields,
            Space::new().height(SPACING_XL),
            actions,
        ]
        .padding(SPACING_LG)
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn build_edit_actions() -> Element<'static, Message> {
    row![
        // Save button (primary)
        button(
            row![
                lucide::check().size(16).color(WHITE),
                Space::new().width(SPACING_SM),
                text("Save").size(14).color(WHITE),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
            SuppMessage::SaveEdit,
        )))
        .padding([10.0, 24.0])
        .style(|_: &Theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => PRIMARY_600,
                iced::widget::button::Status::Pressed => PRIMARY_700,
                _ => PRIMARY_500,
            };
            iced::widget::button::Style {
                background: Some(bg.into()),
                text_color: WHITE,
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }),
        Space::new().width(SPACING_MD),
        // Cancel button
        button(text("Cancel").size(14).color(GRAY_600))
            .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
                SuppMessage::CancelEdit,
            )))
            .padding([10.0, 24.0])
            .style(|_: &Theme, status| {
                let text_color = match status {
                    iced::widget::button::Status::Hovered => GRAY_800,
                    _ => GRAY_600,
                };
                iced::widget::button::Style {
                    background: None,
                    text_color,
                    border: Border {
                        color: GRAY_300,
                        width: 1.0,
                        radius: BORDER_RADIUS_SM.into(),
                    },
                    ..Default::default()
                }
            }),
    ]
    .into()
}

// =============================================================================
// SKIPPED VIEW - Skip message + sample data + Add instead button
// =============================================================================

fn build_skipped_view(
    domain: &Domain,
    col_name: &str,
    domain_code: &str,
) -> Element<'static, Message> {
    let header = build_detail_header(col_name, domain_code);
    let sample_data = build_sample_data(domain, col_name);

    // Skip message
    let skip_message = container(
        column![
            row![
                lucide::circle_minus().size(20).color(GRAY_500),
                Space::new().width(SPACING_SM),
                text("Skipped").size(16).color(GRAY_700).font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                }),
            ]
            .align_y(Alignment::Center),
            Space::new().height(SPACING_SM),
            text("This column will not be included in the output.")
                .size(13)
                .color(GRAY_500),
        ]
        .padding(SPACING_MD),
    )
    .style(|_: &Theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            color: GRAY_300,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill);

    // Action - Add to SUPP instead
    let supp_name = format!("SUPP{}", domain_code);
    let action = button(
        row![
            lucide::plus().size(16).color(PRIMARY_500),
            Space::new().width(SPACING_SM),
            text(format!("Add to {} instead", supp_name))
                .size(14)
                .color(PRIMARY_500),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
        SuppMessage::UndoAction,
    )))
    .padding([10.0, 20.0])
    .style(|_: &Theme, status| {
        let (bg, text_color) = match status {
            iced::widget::button::Status::Hovered => (PRIMARY_100, PRIMARY_700),
            _ => (WHITE, PRIMARY_500),
        };
        iced::widget::button::Style {
            background: Some(bg.into()),
            text_color,
            border: Border {
                color: PRIMARY_500,
                width: 1.0,
                radius: BORDER_RADIUS_SM.into(),
            },
            ..Default::default()
        }
    });

    scrollable(
        column![
            header,
            Space::new().height(SPACING_LG),
            sample_data,
            Space::new().height(SPACING_LG),
            skip_message,
            Space::new().height(SPACING_XL),
            action,
        ]
        .padding(SPACING_LG)
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

// =============================================================================
// SHARED COMPONENTS
// =============================================================================

fn build_detail_header(col_name: &str, domain_code: &str) -> Element<'static, Message> {
    let col_display = col_name.to_string();
    let target = format!("SUPP{}", domain_code);

    column![
        text("Configure SUPP Variable")
            .size(18)
            .color(GRAY_900)
            .font(iced::Font {
                weight: iced::font::Weight::Semibold,
                ..Default::default()
            }),
        Space::new().height(4.0),
        row![
            text("Source:").size(13).color(GRAY_500),
            Space::new().width(SPACING_XS),
            text(col_display).size(13).color(GRAY_800).font(iced::Font {
                weight: iced::font::Weight::Semibold,
                ..Default::default()
            }),
        ],
        Space::new().height(2.0),
        row![
            text("Target:").size(13).color(GRAY_500),
            Space::new().width(SPACING_XS),
            text(target).size(13).color(GRAY_600),
        ],
    ]
    .into()
}

fn build_sample_data(domain: &Domain, col_name: &str) -> Element<'static, Message> {
    let samples = get_sample_values(domain, col_name, 5);

    let sample_chips: Vec<Element<'static, Message>> = samples
        .into_iter()
        .map(|s| {
            container(text(s).size(11).color(GRAY_700))
                .padding([4.0, 8.0])
                .style(|_: &Theme| container::Style {
                    background: Some(GRAY_100.into()),
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
        text("No data available").size(12).color(GRAY_400).into()
    } else {
        row(sample_chips).spacing(SPACING_XS).wrap().into()
    };

    container(
        column![
            row![
                lucide::database().size(14).color(GRAY_500),
                Space::new().width(SPACING_SM),
                text("Sample Values").size(13).color(GRAY_600),
            ]
            .align_y(Alignment::Center),
            Space::new().height(SPACING_SM),
            sample_content,
        ]
        .width(Length::Fill),
    )
    .padding(SPACING_MD)
    .style(|_: &Theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn build_editable_fields(
    config: SuppColumnConfig,
    qnam_conflict_error: Option<String>,
) -> Element<'static, Message> {
    // QNAM field (required) - check empty first, then conflict
    let qnam_error = if config.qnam.trim().is_empty() {
        Some("QNAM is required".to_string())
    } else {
        qnam_conflict_error
    };
    let qnam_field = build_text_field(
        "QNAM *",
        "Qualifier name (max 8 chars)",
        config.qnam,
        QNAM_MAX_LEN,
        qnam_error,
        true,
        |v| {
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QnamChanged(
                v.to_uppercase(),
            )))
        },
    );

    // QLABEL field (required) - validate empty
    let qlabel_error = if config.qlabel.trim().is_empty() {
        Some("QLABEL is required".to_string())
    } else {
        None
    };
    let qlabel_field = build_text_field(
        "QLABEL *",
        "Describe what this value represents...",
        config.qlabel,
        QLABEL_MAX_LEN,
        qlabel_error,
        true,
        |v| Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QlabelChanged(v))),
    );

    // QORIG picker
    let qorig_field = build_origin_picker(config.qorig);

    // QEVAL field (optional)
    let qeval_field = build_text_field(
        "QEVAL",
        "Evaluator (e.g., INVESTIGATOR)",
        config.qeval.unwrap_or_default(),
        40,
        None,
        false,
        |v| Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QevalChanged(v))),
    );

    column![qnam_field, qlabel_field, qorig_field, qeval_field,]
        .spacing(SPACING_MD)
        .into()
}

fn build_text_field<F>(
    label: &'static str,
    placeholder: &'static str,
    value: String,
    max_len: usize,
    error: Option<String>,
    _is_required: bool,
    on_change: F,
) -> Element<'static, Message>
where
    F: 'static + Fn(String) -> Message,
{
    let char_count = value.len();
    let is_over = char_count > max_len;
    let has_error = error.is_some() || is_over;

    let error_msg: Element<'static, Message> = if let Some(err) = error {
        row![
            lucide::circle_alert().size(12).color(ERROR),
            Space::new().width(4.0),
            text(err).size(11).color(ERROR),
        ]
        .into()
    } else {
        Space::new().height(0.0).into()
    };

    let count_display = format!("{}/{}", char_count, max_len);

    column![
        row![
            text(label).size(12).color(GRAY_600),
            Space::new().width(Length::Fill),
            text(count_display)
                .size(11)
                .color(if is_over { ERROR } else { GRAY_400 }),
        ],
        Space::new().height(4.0),
        text_input(placeholder, &value)
            .on_input(on_change)
            .padding([10.0, 12.0])
            .size(14)
            .style(move |_: &Theme, _status| {
                let border_color = if has_error { ERROR } else { GRAY_300 };
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
            }),
        error_msg,
    ]
    .into()
}

fn build_origin_picker(current: SuppOrigin) -> Element<'static, Message> {
    column![
        text("QORIG").size(12).color(GRAY_600),
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

fn get_sample_values(domain: &Domain, col_name: &str, max: usize) -> Vec<String> {
    let mut samples = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Ok(col) = domain.source.data.column(col_name) {
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

fn check_qnam_conflict(domain: &Domain, current_col: &str, qnam: &str) -> Option<String> {
    if qnam.is_empty() {
        return None;
    }

    // Only check against columns already included in SUPP
    for (col, config) in &domain.supp_config {
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
// ALL MAPPED STATE
// =============================================================================

fn view_all_mapped_state(domain_code: &str) -> Element<'static, Message> {
    let msg1 = format!(
        "All source columns are mapped to {} variables.",
        domain_code
    );

    container(
        column![
            lucide::circle_check().size(48).color(SUCCESS),
            Space::new().height(SPACING_LG),
            text("All Columns Mapped").size(18).color(GRAY_700),
            Space::new().height(SPACING_SM),
            text(msg1).size(13).color(GRAY_500),
            Space::new().height(4.0),
            text("No SUPP configuration needed.")
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
// DISPLAY IMPLEMENTATIONS
// =============================================================================

impl std::fmt::Display for SuppOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}
