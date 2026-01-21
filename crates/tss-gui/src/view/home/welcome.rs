//! Welcome view - displayed when no study is loaded.
//!
//! Shows app branding, workflow selector, and recent studies.

use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Alignment, Border, Color, Element, Length};
use iced_fonts::lucide;

use crate::component::SectionCard;
use crate::message::{HomeMessage, Message};
use crate::state::{AppState, RecentStudy, WorkflowMode};
use crate::theme::{
    BORDER_RADIUS_MD, BORDER_RADIUS_SM, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL, SPACING_XS,
    button_primary, colors,
};

/// Embedded SVG logo bytes.
const LOGO_SVG: &[u8] = include_bytes!("../../../assets/icon.svg");

/// Render the welcome view (no study loaded).
pub fn view_welcome(state: &'_ AppState, workflow_mode: WorkflowMode) -> Element<'_, Message> {
    let c = colors();

    let content = column![
        // Logo
        view_logo(),
        Space::new().height(SPACING_LG),
        // Title
        text("Trial Submission Studio")
            .size(28)
            .color(c.text_primary),
        Space::new().height(SPACING_XS),
        // Tagline
        text("Transform clinical data into FDA-compliant CDISC formats")
            .size(14)
            .color(c.text_muted),
        Space::new().height(SPACING_XL),
        // Workflow selector card
        view_workflow_selector(workflow_mode),
        Space::new().height(SPACING_XL),
        // Recent studies section
        view_recent_studies(state),
    ]
    .align_x(Alignment::Center)
    .max_width(500.0);

    let bg_elevated = c.background_elevated;
    // Center the content
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(SPACING_XL)
        .style(move |_| container::Style {
            background: Some(bg_elevated.into()),
            ..Default::default()
        })
        .into()
}

/// Render the app logo.
fn view_logo<'a>() -> Element<'a, Message> {
    let logo_handle = svg::Handle::from_memory(LOGO_SVG);
    svg(logo_handle).width(80).height(80).into()
}

/// Render the workflow selector card.
fn view_workflow_selector<'a>(workflow_mode: WorkflowMode) -> Element<'a, Message> {
    let c = colors();

    // Workflow mode buttons
    let mode_buttons = row![
        workflow_button(WorkflowMode::Sdtm, workflow_mode, true),
        Space::new().width(SPACING_SM),
        workflow_button(WorkflowMode::Adam, workflow_mode, false),
        Space::new().width(SPACING_SM),
        workflow_button(WorkflowMode::Send, workflow_mode, false),
    ]
    .align_y(Alignment::Center);

    // Mode description
    let description = text(workflow_mode.description())
        .size(13)
        .color(c.text_muted);

    // Open Study button
    let open_button = button(
        row![
            lucide::folder_open().size(16).color(c.text_on_accent),
            Space::new().width(SPACING_SM),
            text("Open Study Folder").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::OpenStudyClicked))
    .padding([SPACING_SM, SPACING_LG])
    .style(button_primary);

    // Build the card content
    let card_content = column![
        mode_buttons,
        Space::new().height(SPACING_SM),
        description,
        Space::new().height(SPACING_LG),
        container(open_button).center_x(Length::Fill),
    ]
    .width(Length::Fill);

    SectionCard::new("Select CDISC Standard", card_content)
        .icon(lucide::layers().size(14).color(c.text_secondary))
        .view()
}

/// Render a workflow mode button.
fn workflow_button<'a>(
    mode: WorkflowMode,
    current: WorkflowMode,
    enabled: bool,
) -> Element<'a, Message> {
    let c = colors();
    let is_selected = mode == current;
    let label = mode.display_name();

    // Resolve colors for use in the closure
    let disabled_bg = c.background_secondary;
    let disabled_text = c.text_disabled;
    let disabled_border = c.border_default;
    let selected_bg = Color {
        a: 0.15,
        ..c.accent_primary
    };
    let selected_text = c.accent_primary;
    let selected_border = c.accent_primary;
    let hover_bg = c.background_secondary;
    let hover_text = c.text_primary;
    let hover_border = c.text_disabled;
    let default_bg = c.background_elevated;
    let default_text = c.text_secondary;
    let default_border = Color {
        a: 0.8,
        ..c.border_default
    };

    let style_fn = move |_theme: &iced::Theme, status: button::Status| {
        let (bg, text_color, border_color) = if !enabled {
            // Disabled state
            (disabled_bg, disabled_text, disabled_border)
        } else if is_selected {
            // Selected state
            (selected_bg, selected_text, selected_border)
        } else {
            match status {
                button::Status::Hovered => (hover_bg, hover_text, hover_border),
                _ => (default_bg, default_text, default_border),
            }
        };

        button::Style {
            background: Some(bg.into()),
            text_color,
            border: Border {
                radius: BORDER_RADIUS_MD.into(),
                width: 1.0,
                color: border_color,
            },
            ..Default::default()
        }
    };

    let btn = button(text(label).size(13)).padding([SPACING_SM, SPACING_MD]);

    if enabled {
        // For now, only SDTM is functional - we could add a message for mode switching
        btn.style(style_fn).into()
    } else {
        // Disabled button (ADaM, SEND coming soon)
        btn.style(style_fn).into()
    }
}

/// Render the recent studies section.
fn view_recent_studies<'a>(state: &'a AppState) -> Element<'a, Message> {
    let c = colors();
    let recent_sorted = state.settings.general.recent_sorted();

    if recent_sorted.is_empty() {
        return Space::new().height(0.0).into();
    }

    let text_muted = c.text_muted;
    let text_secondary = c.text_secondary;

    // Clear all button
    let clear_btn: Element<'_, Message> = button(
        row![
            lucide::trash().size(12).color(text_muted),
            Space::new().width(4.0),
            text("Clear All").size(11).color(text_muted),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::ClearAllRecentStudies))
    .padding([4.0, 8.0])
    .style(move |_theme: &iced::Theme, status| {
        let txt_color = match status {
            button::Status::Hovered => text_secondary,
            _ => text_muted,
        };
        button::Style {
            background: None,
            text_color: txt_color,
            ..Default::default()
        }
    })
    .into();

    // Section header with clear button
    let header = row![
        lucide::timer().size(14).color(c.text_muted),
        Space::new().width(SPACING_SM),
        text("Recent Studies").size(13).color(c.text_muted),
        Space::new().width(Length::Fill),
        clear_btn,
    ]
    .align_y(Alignment::Center);

    // Recent study items - show all up to max_recent
    let max_display = state.settings.general.max_recent;
    let mut items = column![header, Space::new().height(SPACING_SM),].width(Length::Fill);

    for study in recent_sorted.iter().take(max_display) {
        items = items.push(recent_study_item(study));
        items = items.push(Space::new().height(SPACING_XS));
    }

    items.into()
}

/// Render a recent study item with rich metadata.
fn recent_study_item<'a>(study: &'a RecentStudy) -> Element<'a, Message> {
    let c = colors();
    let path_clone = study.path.clone();
    let path_for_remove = study.path.clone();
    let is_stale = !study.exists();

    // Resolve all needed colors
    let accent_primary = c.accent_primary;
    let accent_light = Color {
        a: 0.15,
        ..accent_primary
    };
    let status_warning = c.status_warning;
    let status_warning_light = c.status_warning_light;
    let text_muted = c.text_muted;
    let text_primary = c.text_primary;
    let text_disabled = c.text_disabled;
    let text_secondary = c.text_secondary;
    let bg_secondary = c.background_secondary;
    let bg_elevated = c.background_elevated;
    let border_default = c.border_default;

    // Workflow badge
    let workflow_badge = container(
        text(study.workflow_type.label())
            .size(9)
            .color(accent_primary),
    )
    .padding([2.0, 6.0])
    .style(move |_theme| container::Style {
        background: Some(accent_light.into()),
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Stale indicator
    let stale_icon: Element<'_, Message> = if is_stale {
        container(
            row![
                lucide::triangle_alert().size(10).color(status_warning),
                Space::new().width(2.0),
                text("Missing").size(9).color(status_warning),
            ]
            .align_y(Alignment::Center),
        )
        .padding([2.0, 4.0])
        .style(move |_theme| container::Style {
            background: Some(status_warning_light.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::new().width(0.0).into()
    };

    // Top row: name, badges, time
    let name_color = if is_stale { text_muted } else { text_primary };
    let top_row = row![
        text(&study.display_name).size(13).color(name_color),
        Space::new().width(SPACING_XS),
        workflow_badge,
        Space::new().width(SPACING_XS),
        stale_icon,
        Space::new().width(Length::Fill),
        text(study.relative_time()).size(11).color(text_muted),
    ]
    .align_y(Alignment::Center);

    // Bottom row: stats and path
    let stats_color = if is_stale { text_disabled } else { text_muted };
    let bottom_row = row![
        text(study.stats_string()).size(11).color(stats_color),
        Space::new().width(SPACING_SM),
        text(study.path.display().to_string())
            .size(10)
            .color(text_disabled),
    ]
    .align_y(Alignment::Center);

    // Remove button
    let remove_btn = button(lucide::x().size(12).color(text_muted))
        .on_press(Message::Home(HomeMessage::RemoveFromRecent(
            path_for_remove,
        )))
        .padding(4.0)
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered => bg_secondary,
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(bg.into()),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    let folder_color = if is_stale { text_disabled } else { text_muted };
    let content = row![
        lucide::folder().size(16).color(folder_color),
        Space::new().width(SPACING_SM),
        column![top_row, Space::new().height(2.0), bottom_row,].width(Length::Fill),
        Space::new().width(SPACING_SM),
        remove_btn,
    ]
    .align_y(Alignment::Center);

    let main_btn = button(content)
        .on_press_maybe(if is_stale {
            None
        } else {
            Some(Message::Home(HomeMessage::RecentStudyClicked(path_clone)))
        })
        .padding([SPACING_SM, SPACING_MD])
        .width(Length::Fill)
        .style(move |_theme, status| {
            let bg = if is_stale {
                bg_secondary
            } else {
                match status {
                    button::Status::Hovered => bg_secondary,
                    _ => bg_elevated,
                }
            };
            let btn_border_color = if is_stale {
                Color {
                    a: 0.8,
                    ..border_default
                }
            } else {
                border_default
            };
            button::Style {
                background: Some(bg.into()),
                text_color: text_secondary,
                border: Border {
                    radius: BORDER_RADIUS_MD.into(),
                    color: btn_border_color,
                    width: 1.0,
                },
                ..Default::default()
            }
        });

    main_btn.into()
}
