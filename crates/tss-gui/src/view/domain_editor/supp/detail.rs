//! Detail panel components for the SUPP tab.
//!
//! Contains the right-side detail views for pending, included, edit, and skipped states.

use iced::widget::{Space, button, column, container, row, rule, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::display::MetadataCard;
use crate::message::domain_editor::SuppMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{SourceDomainState, SuppAction, SuppColumnConfig, SuppEditDraft, SuppUiState};
use crate::theme::{
    ALPHA_LIGHT, BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
};
use crate::view::domain_editor::detail_no_selection_default;

use super::helpers::{
    build_detail_header, build_editable_fields, build_sample_data, check_qnam_conflict,
};

// =============================================================================
// DETAIL PANEL
// =============================================================================

pub(super) fn build_detail_panel(
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
        None => detail_no_selection_default(
            "Select a Column",
            "Click on a column in the list to configure its SUPP settings",
        ),
    }
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
