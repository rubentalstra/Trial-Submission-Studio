//! Third-party licenses dialog view.
//!
//! Displays open source license acknowledgments.

use iced::widget::{Space, button, column, container, markdown, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_fonts::lucide;

use crate::message::{DialogMessage, Message, ThirdPartyMessage};
use crate::theme::{
    BORDER_RADIUS_LG, GRAY_100, GRAY_500, GRAY_700, GRAY_900, SPACING_LG, SPACING_MD, SPACING_SM,
    WHITE,
};

/// Third-party license information (embedded at build time).
const THIRD_PARTY_LICENSES: &str = include_str!("../../../../../THIRD_PARTY_LICENSES.md");

/// Pre-parsed markdown state for the third-party licenses dialog.
#[derive(Debug, Clone, Default)]
pub struct ThirdPartyState {
    markdown_items: Vec<markdown::Item>,
}

impl ThirdPartyState {
    /// Parse bundled licenses markdown for rendering.
    pub fn new() -> Self {
        Self {
            markdown_items: markdown::parse(THIRD_PARTY_LICENSES).collect(),
        }
    }

    /// Borrow the parsed markdown items.
    pub fn items(&self) -> &[markdown::Item] {
        &self.markdown_items
    }
}

/// Render the Third-party licenses dialog.
pub fn view_third_party_dialog(state: &'_ ThirdPartyState) -> Element<'_, Message> {
    let backdrop = container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
            ..Default::default()
        });

    let dialog_content = view_dialog_content(state);

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

/// Render the Third-party licenses dialog content for a standalone window (multi-window mode).
///
/// This is the content that appears in a separate dialog window.
pub fn view_third_party_dialog_content(state: &'_ ThirdPartyState) -> Element<'_, Message> {
    let content = view_dialog_content(state);

    // Wrap in a styled container for the window
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(GRAY_100.into()),
            ..Default::default()
        })
        .into()
}

/// Dialog content with header, scrollable content, and footer.
fn view_dialog_content(state: &'_ ThirdPartyState) -> Element<'_, Message> {
    let header = view_header();
    let content = view_licenses_content(state.items());
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
fn view_licenses_content<'a>(items: &'a [markdown::Item]) -> Element<'a, Message> {
    if items.is_empty() {
        return text("No third-party licenses found.")
            .size(13)
            .color(GRAY_500)
            .into();
    }

    let markdown_content: Element<'a, Message> =
        markdown::view(items, Theme::Light).map(|url| Message::OpenUrl(url.to_string()));

    let scroll = scrollable(
        container(markdown_content)
            .padding(SPACING_MD)
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill);

    container(scroll)
        .style(|_| container::Style {
            background: Some(GRAY_100.into()),
            ..Default::default()
        })
        .height(Length::Fill)
        .into()
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
