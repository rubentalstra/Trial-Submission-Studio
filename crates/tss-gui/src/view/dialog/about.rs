//! About dialog view.
//!
//! Displays application information, version, and links.

use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Alignment, Border, Color, Element, Length};
use iced_fonts::lucide;

use crate::message::{AboutMessage, DialogMessage, Message};
use crate::theme::{
    BORDER_RADIUS_LG, GRAY_500, GRAY_600, GRAY_700, GRAY_900, SPACING_LG, SPACING_MD, SPACING_SM,
    SPACING_XS, WHITE,
};

/// Embedded SVG logo bytes.
const LOGO_SVG: &[u8] = include_bytes!("../../../assets/icon.svg");

/// Application version from Cargo.toml.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Render the About dialog.
pub fn view_about_dialog<'a>() -> Element<'a, Message> {
    let backdrop = container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
            ..Default::default()
        });

    let dialog_content = view_dialog_content();

    let dialog = container(dialog_content)
        .width(400)
        .style(|_| container::Style {
            background: Some(WHITE.into()),
            border: Border {
                radius: BORDER_RADIUS_LG.into(),
                ..Default::default()
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        });

    let centered_dialog = container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Shrink)
        .center_y(Length::Shrink);

    iced::widget::stack![backdrop, centered_dialog].into()
}

/// Dialog content with app info and links.
fn view_dialog_content<'a>() -> Element<'a, Message> {
    // App logo from embedded SVG
    let logo_handle = svg::Handle::from_memory(LOGO_SVG);
    let logo = svg(logo_handle).width(72).height(72);

    // App name and version
    let title = text("Trial Submission Studio").size(20).color(GRAY_900);

    let version = text(format!("Version {}", VERSION))
        .size(14)
        .color(GRAY_600);

    let description = text("Transform clinical trial data into FDA-compliant CDISC formats")
        .size(13)
        .color(GRAY_700);

    // Divider
    let divider =
        container(Space::new().width(Length::Fill).height(1)).style(|_| container::Style {
            background: Some(GRAY_500.into()),
            ..Default::default()
        });

    // Links section
    let website_btn = button(
        row![
            lucide::globe().size(14),
            Space::new().width(SPACING_XS),
            text("Website").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Dialog(DialogMessage::About(
        AboutMessage::OpenWebsite,
    )))
    .padding([SPACING_XS, SPACING_SM]);

    let github_btn = button(
        row![
            lucide::github().size(14),
            Space::new().width(SPACING_XS),
            text("GitHub").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Dialog(DialogMessage::About(
        AboutMessage::OpenGitHub,
    )))
    .padding([SPACING_XS, SPACING_SM]);

    let links =
        row![website_btn, Space::new().width(SPACING_SM), github_btn,].align_y(Alignment::Center);

    // Copyright
    let copyright = text("© 2024-2025 Trial Submission Studio Contributors")
        .size(11)
        .color(GRAY_500);

    // Close button
    let close_btn = button(text("Close").size(14))
        .on_press(Message::Dialog(DialogMessage::About(AboutMessage::Close)))
        .padding([SPACING_SM, SPACING_LG]);

    column![
        Space::new().height(SPACING_MD),
        logo,
        Space::new().height(SPACING_SM),
        title,
        version,
        Space::new().height(SPACING_XS),
        description,
        Space::new().height(SPACING_MD),
        divider,
        Space::new().height(SPACING_MD),
        links,
        Space::new().height(SPACING_SM),
        copyright,
        Space::new().height(SPACING_LG),
        close_btn,
        Space::new().height(SPACING_MD),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}
