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
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_fonts::lucide;
use polars::prelude::AnyValue;

use crate::component::display::{EmptyState, MetadataCard, NoFilteredResults};
use crate::component::inputs::TextField;
use crate::component::layout::SplitView;
use crate::component::panels::{DetailHeader, FilterToggle};
use crate::message::domain_editor::SuppMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{
    AppState, SourceDomainState, SuppAction, SuppColumnConfig, SuppEditDraft, SuppFilterMode,
    SuppOrigin, SuppUiState, ViewState,
};
use crate::theme::{
    ALPHA_LIGHT, BORDER_RADIUS_SM, ClinicalColors, MASTER_WIDTH, MAX_CHARS_SHORT_LABEL,
    MAX_CHARS_VARIABLE_NAME, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
};

// =============================================================================
// MAIN SUPP TAB VIEW
// =============================================================================

/// Render the SUPP configuration tab content.
pub fn view_supp_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return container(text("Domain not found").size(14).style(|theme: &Theme| {
                text::Style {
                    color: Some(theme.clinical().text_muted),
                }
            }))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Shrink)
            .center_y(Length::Shrink)
            .into();
        }
    };

    // SUPP configuration only applies to source domains
    let source = match domain.as_source() {
        Some(s) => s,
        None => {
            return container(
                text("Generated domains do not have SUPP columns")
                    .size(14)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted),
                    }),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Shrink)
            .center_y(Length::Shrink)
            .into();
        }
    };

    // Get UI state
    let supp_ui = match &state.view {
        ViewState::DomainEditor(editor) => &editor.supp_ui,
        _ => return text("Invalid view state").into(),
    };

    // Get unmapped columns
    let unmapped_columns = source.unmapped_columns();

    // If no unmapped columns, show success state
    if unmapped_columns.is_empty() {
        return view_all_mapped_state(domain_code);
    }

    // Filter columns based on search and filter mode
    let filtered: Vec<String> = unmapped_columns
        .iter()
        .filter(|col: &&String| {
            // Search filter
            if !supp_ui.search_filter.is_empty()
                && !col
                    .to_lowercase()
                    .contains(&supp_ui.search_filter.to_lowercase())
            {
                return false;
            }

            // Action filter
            let supp_config = source.supp_config.get(*col);
            match supp_ui.filter_mode {
                SuppFilterMode::All => true,
                SuppFilterMode::Pending => {
                    supp_config.is_none_or(|c| c.action == SuppAction::Pending)
                }
                SuppFilterMode::Included => {
                    supp_config.is_some_and(|c| c.action == SuppAction::Include)
                }
                SuppFilterMode::Skipped => {
                    supp_config.is_some_and(|c| c.action == SuppAction::Skip)
                }
            }
        })
        .cloned()
        .collect();

    // Build master header (pinned at top)
    let master_header = build_master_header_pinned(supp_ui, filtered.len());

    // Build master content (scrollable column list)
    let master_content = build_master_content(&filtered, source, supp_ui);

    // Build detail panel
    let detail = build_detail_panel(source, supp_ui, domain_code);

    // Use split view layout with pinned header
    SplitView::new(master_content, detail)
        .master_width(MASTER_WIDTH)
        .master_header(master_header)
        .view()
}

// =============================================================================
// MASTER PANEL: HEADER (PINNED)
// =============================================================================

/// Left panel header: search, filters, and stats (pinned at top).
fn build_master_header_pinned<'a>(
    ui: &'a SuppUiState,
    filtered_count: usize,
) -> Element<'a, Message> {
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
        text(format!("{}", filtered_count))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
        Space::new().width(4.0),
        text("columns").size(11).style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_muted),
        }),
    ]
    .align_y(Alignment::Center);

    column![
        search,
        Space::new().height(SPACING_XS),
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
    domain: &'a SourceDomainState,
    ui: &'a SuppUiState,
) -> Element<'a, Message> {
    if filtered.is_empty() {
        return NoFilteredResults::new("No columns match filter")
            .hint("Try adjusting your search or filter")
            .height(120.0)
            .view();
    }

    // Build column items
    let mut items = column![].spacing(SPACING_XS);

    for col_name in filtered {
        let supp_config = domain.supp_config.get(col_name);
        let action = supp_config.map_or(SuppAction::Pending, |c| c.action);
        let is_selected = ui.selected_column.as_deref() == Some(col_name.as_str());
        let item = build_column_item(col_name.clone(), action, is_selected);
        items = items.push(item);
    }

    items.into()
}

fn build_filter_buttons(current: SuppFilterMode) -> Element<'static, Message> {
    row![
        FilterToggle::new(
            "All",
            current == SuppFilterMode::All,
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::FilterModeChanged(
                SuppFilterMode::All
            )))
        )
        .view(),
        FilterToggle::new(
            "Pending",
            current == SuppFilterMode::Pending,
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::FilterModeChanged(
                SuppFilterMode::Pending
            )))
        )
        .view(),
        FilterToggle::new(
            "SUPP",
            current == SuppFilterMode::Included,
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::FilterModeChanged(
                SuppFilterMode::Included
            )))
        )
        .view(),
        FilterToggle::new(
            "Skip",
            current == SuppFilterMode::Skipped,
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::FilterModeChanged(
                SuppFilterMode::Skipped
            )))
        )
        .view(),
    ]
    .spacing(SPACING_XS)
    .into()
}

fn build_column_item(
    col_name: String,
    action: SuppAction,
    is_selected: bool,
) -> Element<'static, Message> {
    // Use static colors for status icons
    let status_icon: Element<'static, Message> = match action {
        SuppAction::Pending => lucide::circle()
            .size(10)
            .color(Color::from_rgb(0.65, 0.65, 0.70))
            .into(),
        SuppAction::Include => lucide::circle_check()
            .size(10)
            .color(Color::from_rgb(0.20, 0.78, 0.35))
            .into(),
        SuppAction::Skip => lucide::circle_x()
            .size(10)
            .color(Color::from_rgb(0.65, 0.65, 0.70))
            .into(),
    };

    let display_name = col_name.clone();

    button(
        row![
            status_icon,
            Space::new().width(SPACING_SM),
            text(display_name)
                .size(13)
                .style(move |theme: &Theme| text::Style {
                    color: Some(if is_selected {
                        theme.extended_palette().primary.base.color
                    } else {
                        theme.extended_palette().background.base.text
                    }),
                }),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
        SuppMessage::ColumnSelected(col_name),
    )))
    .padding([8.0, 12.0])
    .width(Length::Fill)
    .style(move |theme: &Theme, _status| {
        let accent_primary = theme.extended_palette().primary.base.color;
        let accent_light = Color {
            a: ALPHA_LIGHT,
            ..accent_primary
        };
        let bg_elevated = theme.clinical().background_elevated;

        let bg_color = if is_selected {
            accent_light
        } else {
            bg_elevated
        };

        iced::widget::button::Style {
            background: Some(bg_color.into()),
            text_color: if is_selected {
                accent_primary
            } else {
                theme.extended_palette().background.base.text
            },
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

fn build_detail_panel(
    source: &SourceDomainState,
    ui: &SuppUiState,
    domain_code: &str,
) -> Element<'static, Message> {
    match &ui.selected_column {
        Some(col) => {
            let config = source
                .supp_config
                .get(col)
                .cloned()
                .unwrap_or_else(|| SuppColumnConfig::from_column(col));

            match (&config.action, ui.edit_draft.as_ref()) {
                (SuppAction::Pending, _) => build_pending_view(source, col, &config, domain_code),
                (SuppAction::Include, Some(draft)) => {
                    build_edit_view(source, col, draft, domain_code)
                }
                (SuppAction::Include, None) => {
                    build_included_view(source, col, &config, domain_code)
                }
                (SuppAction::Skip, _) => build_skipped_view(source, col, domain_code),
            }
        }
        None => build_no_selection_state(),
    }
}

fn build_no_selection_state() -> Element<'static, Message> {
    EmptyState::new(
        container(lucide::mouse_pointer_click().size(48)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_disabled),
            ..Default::default()
        }),
        "Select a Column",
    )
    .description("Click on a column in the list to configure its SUPP settings")
    .centered()
    .view()
}

// =============================================================================
// PENDING VIEW - Editable fields + Add/Skip buttons
// =============================================================================

fn build_pending_view(
    source: &SourceDomainState,
    col_name: &str,
    config: &SuppColumnConfig,
    domain_code: &str,
) -> Element<'static, Message> {
    let header = build_detail_header(col_name, domain_code);
    let sample_data = build_sample_data(source, col_name);

    // Check QNAM conflict (only against included columns)
    let qnam_error = check_qnam_conflict(source, col_name, &config.qnam);

    // Editable fields
    let fields = build_editable_fields(config, qnam_error);

    // Action buttons
    let actions = build_pending_actions(domain_code);

    // Consistent layout matching mapping/normalization/validation
    scrollable(column![
        header,
        Space::new().height(SPACING_MD),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        sample_data,
        Space::new().height(SPACING_LG),
        fields,
        Space::new().height(SPACING_LG),
        actions,
        Space::new().height(SPACING_MD),
    ])
    .height(Length::Fill)
    .into()
}

fn build_pending_actions(domain_code: &str) -> Element<'static, Message> {
    let supp_name = format!("SUPP{}", domain_code);

    column![
        // Primary action - Add to SUPP
        button(
            row![
                container(lucide::plus().size(16)).style(|theme: &Theme| container::Style {
                    text_color: Some(theme.clinical().text_on_accent),
                    ..Default::default()
                }),
                Space::new().width(SPACING_SM),
                text(format!("Add to {}", supp_name))
                    .size(14)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_on_accent),
                    }),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
            SuppMessage::AddToSupp,
        )))
        .padding([12.0, 24.0])
        .width(Length::Fill)
        .style(|theme: &Theme, status| {
            let accent_primary = theme.extended_palette().primary.base.color;
            let accent_hover = theme.clinical().accent_hover;
            let accent_pressed = theme.clinical().accent_pressed;
            let bg = match status {
                iced::widget::button::Status::Hovered => accent_hover,
                iced::widget::button::Status::Pressed => accent_pressed,
                _ => accent_primary,
            };
            iced::widget::button::Style {
                background: Some(bg.into()),
                text_color: theme.clinical().text_on_accent,
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
                    container(lucide::x().size(14)).style(|theme: &Theme| container::Style {
                        text_color: Some(theme.clinical().text_muted),
                        ..Default::default()
                    }),
                    Space::new().width(SPACING_XS),
                    text("Skip this column")
                        .size(13)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_muted),
                        }),
                ]
                .align_y(Alignment::Center),
            )
            .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
                SuppMessage::Skip,
            )))
            .padding([8.0, 16.0])
            .style(|theme: &Theme, status| {
                let tc = match status {
                    iced::widget::button::Status::Hovered => theme.clinical().text_secondary,
                    _ => theme.clinical().text_muted,
                };
                iced::widget::button::Style {
                    background: None,
                    text_color: tc,
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
    source: &SourceDomainState,
    col_name: &str,
    config: &SuppColumnConfig,
    domain_code: &str,
) -> Element<'static, Message> {
    let header = build_detail_header(col_name, domain_code);
    let sample_data = build_sample_data(source, col_name);

    // Read-only summary
    let summary = build_readonly_summary(config);

    // Actions
    let actions = build_included_actions();

    // Consistent layout matching mapping/normalization/validation
    scrollable(column![
        header,
        Space::new().height(SPACING_MD),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        sample_data,
        Space::new().height(SPACING_LG),
        summary,
        Space::new().height(SPACING_LG),
        actions,
        Space::new().height(SPACING_MD),
    ])
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
                container(lucide::circle_check().size(16)).style(|theme: &Theme| {
                    container::Style {
                        text_color: Some(theme.extended_palette().success.base.color),
                        ..Default::default()
                    }
                }),
                Space::new().width(SPACING_SM),
                text("Added to SUPP")
                    .size(13)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.extended_palette().success.base.color),
                    })
                    .font(iced::Font {
                        weight: iced::font::Weight::Medium,
                        ..Default::default()
                    }),
            ]
            .align_y(Alignment::Center),
            Space::new().height(SPACING_MD),
            // Fields summary using MetadataCard
            MetadataCard::new()
                .row("QNAM", qnam)
                .row("QLABEL", qlabel)
                .row("QORIG", qorig)
                .row("QEVAL", qeval)
                .view(),
        ]
        .padding(SPACING_MD),
    )
    .style(|theme: &Theme| container::Style {
        background: Some(theme.clinical().status_success_light.into()),
        border: Border {
            color: theme.extended_palette().success.base.color,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

fn build_included_actions() -> Element<'static, Message> {
    row![
        // Edit button
        button(
            row![
                container(lucide::pencil().size(14)).style(|theme: &Theme| container::Style {
                    text_color: Some(theme.extended_palette().primary.base.color),
                    ..Default::default()
                }),
                Space::new().width(SPACING_XS),
                text("Edit").size(13).style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().primary.base.color),
                }),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
            SuppMessage::StartEdit,
        )))
        .padding([8.0, 16.0])
        .style(|theme: &Theme, status| {
            let accent_primary = theme.extended_palette().primary.base.color;
            let accent_hover = theme.clinical().accent_hover;
            let accent_light = Color {
                a: 0.15,
                ..accent_primary
            };
            let text_color = match status {
                iced::widget::button::Status::Hovered => accent_hover,
                _ => accent_primary,
            };
            iced::widget::button::Style {
                background: Some(accent_light.into()),
                text_color,
                border: Border {
                    color: accent_primary,
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
                container(lucide::trash().size(14)).style(|theme: &Theme| container::Style {
                    text_color: Some(theme.clinical().text_muted),
                    ..Default::default()
                }),
                Space::new().width(SPACING_XS),
                text("Remove").size(13).style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
            SuppMessage::UndoAction,
        )))
        .padding([8.0, 16.0])
        .style(|theme: &Theme, status| {
            let text_color = match status {
                iced::widget::button::Status::Hovered => theme.clinical().text_secondary,
                _ => theme.clinical().text_muted,
            };
            iced::widget::button::Style {
                background: None,
                text_color,
                border: Border {
                    color: theme.clinical().border_default,
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
    source: &SourceDomainState,
    col_name: &str,
    draft: &SuppEditDraft,
    domain_code: &str,
) -> Element<'static, Message> {
    let header = build_detail_header(col_name, domain_code);
    let sample_data = build_sample_data(source, col_name);

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
    let qnam_error = check_qnam_conflict(source, col_name, &draft.qnam);

    // Editable fields
    let fields = build_editable_fields(&temp_config, qnam_error);

    // Edit mode info
    let edit_info = container(
        row![
            container(lucide::info().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().warning.base.color),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text("Editing — changes will be saved when you click Save")
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_secondary),
                }),
        ]
        .align_y(Alignment::Center),
    )
    .padding([SPACING_SM, SPACING_MD])
    .style(|theme: &Theme| container::Style {
        background: Some(theme.clinical().status_warning_light.into()),
        border: Border {
            color: theme.extended_palette().warning.base.color,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill);

    // Actions
    let actions = build_edit_actions();

    // Consistent layout matching mapping/normalization/validation
    scrollable(column![
        header,
        Space::new().height(SPACING_MD),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        edit_info,
        Space::new().height(SPACING_LG),
        sample_data,
        Space::new().height(SPACING_LG),
        fields,
        Space::new().height(SPACING_LG),
        actions,
        Space::new().height(SPACING_MD),
    ])
    .height(Length::Fill)
    .into()
}

fn build_edit_actions() -> Element<'static, Message> {
    row![
        // Save button (primary)
        button(
            row![
                container(lucide::check().size(16)).style(|theme: &Theme| container::Style {
                    text_color: Some(theme.clinical().text_on_accent),
                    ..Default::default()
                }),
                Space::new().width(SPACING_SM),
                text("Save").size(14).style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_on_accent),
                }),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
            SuppMessage::SaveEdit,
        )))
        .padding([10.0, 24.0])
        .style(|theme: &Theme, status| {
            let accent_primary = theme.extended_palette().primary.base.color;
            let accent_hover = theme.clinical().accent_hover;
            let accent_pressed = theme.clinical().accent_pressed;
            let bg = match status {
                iced::widget::button::Status::Hovered => accent_hover,
                iced::widget::button::Status::Pressed => accent_pressed,
                _ => accent_primary,
            };
            iced::widget::button::Style {
                background: Some(bg.into()),
                text_color: theme.clinical().text_on_accent,
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }),
        Space::new().width(SPACING_MD),
        // Cancel button
        button(text("Cancel").size(14).style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        }),)
        .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
            SuppMessage::CancelEdit,
        )))
        .padding([10.0, 24.0])
        .style(|theme: &Theme, status| {
            let text_color = match status {
                iced::widget::button::Status::Hovered => {
                    theme.extended_palette().background.base.text
                }
                _ => theme.clinical().text_secondary,
            };
            iced::widget::button::Style {
                background: None,
                text_color,
                border: Border {
                    color: theme.clinical().border_default,
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
    source: &SourceDomainState,
    col_name: &str,
    domain_code: &str,
) -> Element<'static, Message> {
    let header = build_detail_header(col_name, domain_code);
    let sample_data = build_sample_data(source, col_name);

    // Skip message
    let skip_message = container(
        column![
            row![
                container(lucide::circle_minus().size(20)).style(|theme: &Theme| {
                    container::Style {
                        text_color: Some(theme.clinical().text_muted),
                        ..Default::default()
                    }
                }),
                Space::new().width(SPACING_SM),
                text("Skipped")
                    .size(16)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_secondary),
                    })
                    .font(iced::Font {
                        weight: iced::font::Weight::Semibold,
                        ..Default::default()
                    }),
            ]
            .align_y(Alignment::Center),
            Space::new().height(SPACING_SM),
            text("This column will not be included in the output.")
                .size(13)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
        ]
        .padding(SPACING_MD),
    )
    .style(|theme: &Theme| container::Style {
        background: Some(theme.clinical().background_secondary.into()),
        border: Border {
            color: theme.clinical().border_default,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill);

    // Action - Undo skip (returns to pending state)
    let action = button(
        row![
            container(lucide::rotate_ccw().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().primary.base.color),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text("Undo Skip")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().primary.base.color),
                }),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
        SuppMessage::UndoAction,
    )))
    .padding([10.0, 20.0])
    .style(|theme: &Theme, status| {
        let accent_primary = theme.extended_palette().primary.base.color;
        let accent_hover = theme.clinical().accent_hover;
        let accent_light = Color {
            a: ALPHA_LIGHT,
            ..accent_primary
        };
        let bg_elevated = theme.clinical().background_elevated;
        let (bg, text_color) = match status {
            iced::widget::button::Status::Hovered => (accent_light, accent_hover),
            _ => (bg_elevated, accent_primary),
        };
        iced::widget::button::Style {
            background: Some(bg.into()),
            text_color,
            border: Border {
                color: accent_primary,
                width: 1.0,
                radius: BORDER_RADIUS_SM.into(),
            },
            ..Default::default()
        }
    });

    // Consistent layout matching mapping/normalization/validation
    scrollable(column![
        header,
        Space::new().height(SPACING_MD),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        sample_data,
        Space::new().height(SPACING_LG),
        skip_message,
        Space::new().height(SPACING_LG),
        action,
        Space::new().height(SPACING_MD),
    ])
    .height(Length::Fill)
    .into()
}

// =============================================================================
// SHARED COMPONENTS
// =============================================================================

fn build_detail_header(col_name: &str, domain_code: &str) -> Element<'static, Message> {
    let col_display = col_name.to_string();
    let target = format!("SUPP{}", domain_code);

    DetailHeader::new("Configure SUPP Variable")
        .subtitle(format!("Source: {} → {}", col_display, target))
        .view()
}

fn build_sample_data(source: &SourceDomainState, col_name: &str) -> Element<'static, Message> {
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

fn build_editable_fields(
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

fn build_origin_picker(current: SuppOrigin) -> Element<'static, Message> {
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

fn get_sample_values(source: &SourceDomainState, col_name: &str, max: usize) -> Vec<String> {
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

fn check_qnam_conflict(
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
// ALL MAPPED STATE
// =============================================================================

fn view_all_mapped_state(domain_code: &str) -> Element<'static, Message> {
    let description = format!(
        "All source columns are mapped to {} variables. No SUPP configuration needed.",
        domain_code
    );

    EmptyState::new(
        container(lucide::circle_check().size(48)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().success.base.color),
            ..Default::default()
        }),
        "All Columns Mapped",
    )
    .description(description)
    .centered()
    .view()
}

// =============================================================================
// DISPLAY IMPLEMENTATIONS
// =============================================================================

impl std::fmt::Display for SuppOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}
