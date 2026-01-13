//! Third-party licenses dialog view.
//!
//! Displays open source license acknowledgments.

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length};
use iced_fonts::lucide;

use crate::message::{DialogMessage, Message, ThirdPartyMessage};
use crate::theme::{
    BORDER_RADIUS_LG, GRAY_100, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900, SPACING_LG,
    SPACING_MD, SPACING_SM, WHITE,
};

/// Third-party license information (embedded at build time).
const THIRD_PARTY_LICENSES: &str = include_str!("../../../../../THIRD_PARTY_LICENSES.md");

/// Render the Third-party licenses dialog.
pub fn view_third_party_dialog<'a>() -> Element<'a, Message> {
    let backdrop = container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
            ..Default::default()
        });

    let dialog_content = view_dialog_content();

    let dialog = container(dialog_content)
        .width(700)
        .height(550)
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

/// Dialog content with header, scrollable content, and footer.
fn view_dialog_content<'a>() -> Element<'a, Message> {
    let header = view_header();
    let content = view_licenses_content();
    let footer = view_footer();

    column![header, content, footer,].into()
}

/// Dialog header.
fn view_header<'a>() -> Element<'a, Message> {
    row![
        lucide::scale().size(18).color(GRAY_700),
        Space::new().width(SPACING_SM),
        text("Third-Party Licenses").size(18).color(GRAY_900),
    ]
    .align_y(Alignment::Center)
    .padding([SPACING_MD, SPACING_LG])
    .into()
}

/// Scrollable licenses content.
fn view_licenses_content<'a>() -> Element<'a, Message> {
    // Parse and render the licenses markdown
    // For simplicity, we'll render it as plain text with some formatting
    let content = parse_licenses_simple();

    container(
        scrollable(container(content).padding(SPACING_MD).width(Length::Fill)).height(Length::Fill),
    )
    .style(|_| container::Style {
        background: Some(GRAY_100.into()),
        ..Default::default()
    })
    .height(Length::Fill)
    .into()
}

/// Simple license parsing - renders as formatted text.
fn parse_licenses_simple<'a>() -> Element<'a, Message> {
    // Split into sections by "## " headers
    let sections: Vec<&str> = THIRD_PARTY_LICENSES
        .split("\n## ")
        .filter(|s| !s.trim().is_empty())
        .collect();

    let elements: Vec<Element<'a, Message>> = sections
        .into_iter()
        .take(50) // Limit to avoid performance issues
        .map(|section| {
            let lines: Vec<&str> = section.lines().collect();
            let title = lines.first().unwrap_or(&"Unknown");

            // Extract license type if present
            let license_line = lines.iter().find(|l| l.starts_with("License:"));
            let license_text = license_line
                .map(|l| l.trim_start_matches("License:").trim())
                .unwrap_or("Unknown");

            column![
                text(*title).size(13).color(GRAY_800),
                text(format!("License: {}", license_text))
                    .size(11)
                    .color(GRAY_600),
            ]
                .spacing(2)
                .into()
        })
        .collect();

    if elements.is_empty() {
        return text("No third-party licenses found.")
            .size(13)
            .color(GRAY_500)
            .into();
    }

    column(elements).spacing(SPACING_SM).into()
}

/// Dialog footer with close button.
fn view_footer<'a>() -> Element<'a, Message> {
    let close_btn = button(text("Close").size(13))
        .on_press(Message::Dialog(DialogMessage::ThirdParty(
            ThirdPartyMessage::Close,
        )))
        .padding([SPACING_SM, SPACING_LG]);

    row![Space::new().width(Length::Fill), close_btn,]
        .padding([SPACING_MD, SPACING_LG])
        .into()
}
