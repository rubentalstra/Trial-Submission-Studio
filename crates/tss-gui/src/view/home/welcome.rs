//! Welcome view - displayed when no study is loaded.
//!
//! Shows app branding, workflow selector, and recent studies.

use std::path::Path;

use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Alignment, Border, Element, Length};
use iced_fonts::lucide;

use crate::component::SectionCard;
use crate::message::{HomeMessage, Message};
use crate::state::{AppState, WorkflowMode};
use crate::theme::{
    BORDER_RADIUS_MD, GRAY_100, GRAY_200, GRAY_300, GRAY_400, GRAY_500, GRAY_600, GRAY_700,
    GRAY_800, GRAY_900, PRIMARY_100, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL,
    SPACING_XS, WHITE, button_primary,
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
    let recent = &state.settings.general.recent_studies;

    if recent.is_empty() {
        return Space::new().height(0.0).into();
    }

    // Section header
    let header = row![
        lucide::timer().size(14).color(GRAY_600),
        Space::new().width(SPACING_SM),
        text("Recent Studies").size(13).color(GRAY_600),
    ]
    .align_y(Alignment::Center);

    // Recent study items
    let mut items = column![header, Space::new().height(SPACING_SM),].width(Length::Fill);

    for path in recent.iter().take(5) {
        items = items.push(recent_study_item(path));
        items = items.push(Space::new().height(SPACING_XS));
    }

    items.into()
}

/// Render a recent study item.
fn recent_study_item(path: &Path) -> Element<'_, Message> {
    let display_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    let path_display = path.display().to_string();
    let path_clone = path.to_path_buf();

    button(
        row![
            lucide::folder().size(14).color(GRAY_500),
            Space::new().width(SPACING_SM),
            column![
                text(display_name).size(13).color(GRAY_800),
                text(path_display).size(11).color(GRAY_500),
            ]
            .spacing(2.0),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::RecentStudyClicked(path_clone)))
    .padding([SPACING_SM, SPACING_MD])
    .width(Length::Fill)
    .style(|_theme, status| {
        let bg = match status {
            button::Status::Hovered => GRAY_100,
            _ => WHITE,
        };
        button::Style {
            background: Some(bg.into()),
            text_color: GRAY_700,
            border: Border {
                radius: BORDER_RADIUS_MD.into(),
                color: GRAY_200,
                width: 1.0,
            },
            ..Default::default()
        }
    })
    .into()
}
