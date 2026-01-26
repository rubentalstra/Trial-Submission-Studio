//! Welcome view - displayed when no study is loaded.
//!
//! Shows app branding, workflow selector, and recent projects.

use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::panels::SectionCard;
use crate::message::{HomeMessage, Message};
use crate::state::{AppState, RecentProject, WorkflowMode};
use crate::theme::{
    ALPHA_LIGHT, BORDER_RADIUS_MD, BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD,
    SPACING_SM, SPACING_XL, SPACING_XS, button_primary,
};

/// Embedded SVG logo bytes.
const LOGO_SVG: &[u8] = include_bytes!("../../../assets/icon.svg");

/// Render the welcome view (no study loaded).
pub fn view_welcome(state: &'_ AppState, workflow_mode: WorkflowMode) -> Element<'_, Message> {
    let content = column![
        // Logo
        view_logo(),
        Space::new().height(SPACING_LG),
        // Title
        text("Trial Submission Studio")
            .size(28)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        Space::new().height(SPACING_XS),
        // Tagline
        text("Transform clinical data into FDA-compliant CDISC formats")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().height(SPACING_XL),
        // Workflow selector card
        view_workflow_selector(workflow_mode),
        Space::new().height(SPACING_XL),
        // Recent projects section
        view_recent_projects(state),
    ]
    .align_x(Alignment::Center)
    .max_width(500.0);

    // Center the content
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(SPACING_XL)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_elevated.into()),
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
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_muted),
        });

    // New Project button
    let new_project_button = button(
        row![
            container(lucide::folder_plus().size(16)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text("New Project").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::NewProject)
    .padding([SPACING_SM, SPACING_LG])
    .style(button_primary);

    // Build the card content
    let card_content = column![
        mode_buttons,
        Space::new().height(SPACING_SM),
        description,
        Space::new().height(SPACING_LG),
        container(new_project_button).center_x(Length::Fill),
    ]
    .width(Length::Fill);

    SectionCard::new("Select CDISC Standard", card_content)
        .icon(
            container(lucide::layers().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_secondary),
                ..Default::default()
            }),
        )
        .view()
}

/// Render a workflow mode button.
fn workflow_button<'a>(
    mode: WorkflowMode,
    current: WorkflowMode,
    enabled: bool,
) -> Element<'a, Message> {
    let is_selected = mode == current;
    let label = mode.display_name();

    let style_fn = move |theme: &Theme, status: button::Status| {
        let c = theme.clinical();
        let accent_primary = theme.extended_palette().primary.base.color;

        let disabled_bg = c.background_secondary;
        let disabled_text = c.text_disabled;
        let disabled_border = c.border_default;
        let selected_bg = Color {
            a: ALPHA_LIGHT,
            ..accent_primary
        };
        let selected_text = accent_primary;
        let selected_border = accent_primary;
        let hover_bg = c.background_secondary;
        let hover_text = theme.extended_palette().background.base.text;
        let hover_border = c.text_disabled;
        let default_bg = c.background_elevated;
        let default_text = c.text_secondary;
        let default_border = Color {
            a: 0.8,
            ..c.border_default
        };

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

/// Render the recent projects section.
fn view_recent_projects<'a>(state: &'a AppState) -> Element<'a, Message> {
    let recent_sorted = state.settings.general.recent_projects_sorted();

    if recent_sorted.is_empty() {
        return Space::new().height(0.0).into();
    }

    // Clear all button
    let clear_btn: Element<'_, Message> = button(
        row![
            container(lucide::trash().size(12)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_muted),
                ..Default::default()
            }),
            Space::new().width(4.0),
            text("Clear All")
                .size(11)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::ClearAllRecentProjects))
    .padding([4.0, 8.0])
    .style(|theme: &Theme, status| {
        let c = theme.clinical();
        let txt_color = match status {
            button::Status::Hovered => c.text_secondary,
            _ => c.text_muted,
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
        container(lucide::timer().size(14)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_muted),
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text("Recent Projects")
            .size(13)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().width(Length::Fill),
        clear_btn,
    ]
    .align_y(Alignment::Center);

    // Recent project items - show all up to max_recent
    let max_display = state.settings.general.max_recent;
    let mut items = column![header, Space::new().height(SPACING_SM),].width(Length::Fill);

    for project in recent_sorted.iter().take(max_display) {
        items = items.push(recent_project_item(project));
        items = items.push(Space::new().height(SPACING_XS));
    }

    items.into()
}

/// Render a recent project item with rich metadata.
fn recent_project_item<'a>(project: &'a RecentProject) -> Element<'a, Message> {
    let path_clone = project.path.clone();
    let path_for_remove = project.path.clone();
    let is_stale = !project.exists();
    let workflow_label = project.workflow_type.label().to_string();

    // Workflow badge
    let workflow_badge =
        container(
            text(workflow_label)
                .size(9)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().primary.base.color),
                }),
        )
        .padding([2.0, 6.0])
        .style(|theme: &Theme| {
            let accent_primary = theme.extended_palette().primary.base.color;
            let accent_light = Color {
                a: ALPHA_LIGHT,
                ..accent_primary
            };
            container::Style {
                background: Some(accent_light.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    // Stale indicator
    let stale_icon: Element<'_, Message> = if is_stale {
        container(
            row![
                container(lucide::triangle_alert().size(10)).style(|theme: &Theme| {
                    container::Style {
                        text_color: Some(theme.extended_palette().warning.base.color),
                        ..Default::default()
                    }
                }),
                Space::new().width(2.0),
                text("Missing").size(9).style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().warning.base.color),
                }),
            ]
            .align_y(Alignment::Center),
        )
        .padding([2.0, 4.0])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().status_warning_light.into()),
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
    let display_name = project.display_name.clone();
    let relative_time = project.relative_time();
    let top_row = row![
        text(display_name)
            .size(13)
            .style(move |theme: &Theme| text::Style {
                color: Some(if is_stale {
                    theme.clinical().text_muted
                } else {
                    theme.extended_palette().background.base.text
                }),
            }),
        Space::new().width(SPACING_XS),
        workflow_badge,
        Space::new().width(SPACING_XS),
        stale_icon,
        Space::new().width(Length::Fill),
        text(relative_time)
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
    ]
    .align_y(Alignment::Center);

    // Bottom row: stats and path
    let stats_string = project.stats_string();
    let path_display = project.path.display().to_string();
    let bottom_row = row![
        text(stats_string)
            .size(11)
            .style(move |theme: &Theme| text::Style {
                color: Some(if is_stale {
                    theme.clinical().text_disabled
                } else {
                    theme.clinical().text_muted
                }),
            }),
        Space::new().width(SPACING_SM),
        text(path_display)
            .size(10)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_disabled),
            }),
    ]
    .align_y(Alignment::Center);

    // Remove button
    let remove_btn =
        button(
            container(lucide::x().size(12)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_muted),
                ..Default::default()
            }),
        )
        .on_press(Message::Home(HomeMessage::RemoveFromRecentProjects(
            path_for_remove,
        )))
        .padding(4.0)
        .style(|theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered => theme.clinical().background_secondary,
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

    let content = row![
        container(lucide::file_archive().size(16)).style(move |theme: &Theme| container::Style {
            text_color: Some(if is_stale {
                theme.clinical().text_disabled
            } else {
                theme.clinical().text_muted
            }),
            ..Default::default()
        }),
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
            Some(Message::Home(HomeMessage::RecentProjectClicked(path_clone)))
        })
        .padding([SPACING_SM, SPACING_MD])
        .width(Length::Fill)
        .style(move |theme: &Theme, status| {
            let c = theme.clinical();
            let bg = if is_stale {
                c.background_secondary
            } else {
                match status {
                    button::Status::Hovered => c.background_secondary,
                    _ => c.background_elevated,
                }
            };
            let btn_border_color = if is_stale {
                Color {
                    a: 0.8,
                    ..c.border_default
                }
            } else {
                c.border_default
            };
            button::Style {
                background: Some(bg.into()),
                text_color: c.text_secondary,
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
