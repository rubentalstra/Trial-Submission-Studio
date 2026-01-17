//! Welcome view - displayed when no study is loaded.
//!
//! Shows app branding, workflow selector, and recent studies.

use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Alignment, Border, Element, Length};
use iced_fonts::lucide;

use crate::component::SectionCard;
use crate::message::{HomeMessage, Message};
use crate::state::{AppState, RecentStudy, WorkflowMode};
use crate::theme::{
    BORDER_RADIUS_MD, BORDER_RADIUS_SM, GRAY_100, GRAY_200, GRAY_300, GRAY_400, GRAY_500, GRAY_600,
    GRAY_700, GRAY_800, GRAY_900, PRIMARY_100, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM,
    SPACING_XL, SPACING_XS, WARNING, WARNING_LIGHT, WHITE, button_primary,
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
        text("Trial Submission Studio").size(28).color(GRAY_900),
        Space::new().height(SPACING_XS),
        // Tagline
        text("Transform clinical data into FDA-compliant CDISC formats")
            .size(14)
            .color(GRAY_500),
        Space::new().height(SPACING_XL),
        // Workflow selector card
        view_workflow_selector(workflow_mode),
        Space::new().height(SPACING_XL),
        // Recent studies section
        view_recent_studies(state),
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
        .style(|_| container::Style {
            background: Some(WHITE.into()),
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
    let description = text(workflow_mode.description()).size(13).color(GRAY_600);

    // Open Study button
    let open_button = button(
        row![
            lucide::folder_open().size(16).color(WHITE),
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
        .icon(lucide::layers().size(14).color(GRAY_700))
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

    let style_fn = move |_theme: &iced::Theme, status: button::Status| {
        let (bg, text_color, border_color) = if !enabled {
            // Disabled state
            (GRAY_100, GRAY_400, GRAY_200)
        } else if is_selected {
            // Selected state
            (PRIMARY_100, PRIMARY_500, PRIMARY_500)
        } else {
            match status {
                button::Status::Hovered => (GRAY_100, GRAY_800, GRAY_400),
                _ => (WHITE, GRAY_700, GRAY_300),
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
fn view_recent_studies(state: &AppState) -> Element<'_, Message> {
    let recent_sorted = state.settings.general.recent_sorted();

    if recent_sorted.is_empty() {
        return Space::new().height(0.0).into();
    }

    // Clear all button
    let clear_btn: Element<'_, Message> = button(
        row![
            lucide::trash().size(12).color(GRAY_500),
            Space::new().width(4.0),
            text("Clear All").size(11).color(GRAY_500),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::ClearAllRecentStudies))
    .padding([4.0, 8.0])
    .style(|_theme: &iced::Theme, status| {
        let text_color = match status {
            button::Status::Hovered => GRAY_700,
            _ => GRAY_500,
        };
        button::Style {
            background: None,
            text_color,
            ..Default::default()
        }
    })
    .into();

    // Section header with clear button
    let header = row![
        lucide::timer().size(14).color(GRAY_600),
        Space::new().width(SPACING_SM),
        text("Recent Studies").size(13).color(GRAY_600),
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
fn recent_study_item(study: &RecentStudy) -> Element<'_, Message> {
    let path_clone = study.path.clone();
    let path_for_remove = study.path.clone();
    let is_stale = !study.exists();

    // Workflow badge
    let workflow_badge = container(text(study.workflow_type.label()).size(9).color(PRIMARY_500))
        .padding([2.0, 6.0])
        .style(|_theme| container::Style {
            background: Some(PRIMARY_100.into()),
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
                lucide::triangle_alert().size(10).color(WARNING),
                Space::new().width(2.0),
                text("Missing").size(9).color(WARNING),
            ]
            .align_y(Alignment::Center),
        )
        .padding([2.0, 4.0])
        .style(|_theme| container::Style {
            background: Some(WARNING_LIGHT.into()),
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
    let name_color = if is_stale { GRAY_500 } else { GRAY_800 };
    let top_row = row![
        text(&study.display_name).size(13).color(name_color),
        Space::new().width(SPACING_XS),
        workflow_badge,
        Space::new().width(SPACING_XS),
        stale_icon,
        Space::new().width(Length::Fill),
        text(study.relative_time()).size(11).color(GRAY_500),
    ]
    .align_y(Alignment::Center);

    // Bottom row: stats and path
    let stats_color = if is_stale { GRAY_400 } else { GRAY_600 };
    let bottom_row = row![
        text(study.stats_string()).size(11).color(stats_color),
        Space::new().width(SPACING_SM),
        text(study.path.display().to_string())
            .size(10)
            .color(GRAY_400),
    ]
    .align_y(Alignment::Center);

    // Remove button
    let remove_btn = button(lucide::x().size(12).color(GRAY_500))
        .on_press(Message::Home(HomeMessage::RemoveFromRecent(
            path_for_remove,
        )))
        .padding(4.0)
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => GRAY_200,
                _ => iced::Color::TRANSPARENT,
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
        lucide::folder()
            .size(16)
            .color(if is_stale { GRAY_400 } else { GRAY_500 }),
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
                GRAY_100
            } else {
                match status {
                    button::Status::Hovered => GRAY_100,
                    _ => WHITE,
                }
            };
            let border_color = if is_stale { GRAY_300 } else { GRAY_200 };
            button::Style {
                background: Some(bg.into()),
                text_color: GRAY_700,
                border: Border {
                    radius: BORDER_RADIUS_MD.into(),
                    color: border_color,
                    width: 1.0,
                },
                ..Default::default()
            }
        });

    main_btn.into()
}
