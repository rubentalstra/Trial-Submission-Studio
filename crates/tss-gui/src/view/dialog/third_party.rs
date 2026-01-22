//! Third-party licenses dialog view.
//!
//! Displays open source license acknowledgments.

use iced::widget::{Space, button, column, container, markdown, row, scrollable, text};
use iced::{Alignment, Element, Length, Theme};
use iced_fonts::lucide;

use crate::message::{DialogMessage, Message, ThirdPartyMessage};
use crate::theme::{ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM};

/// Third-party license information (embedded at build time).
const THIRD_PARTY_LICENSES: &str = include_str!("../../../../../THIRD_PARTY_LICENSES.md");

/// Pre-parsed markdown state for the third-party licenses dialog.
///
/// The markdown is parsed once when the dialog is opened and cached in state.
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

/// Render the Third-party licenses dialog content for a standalone window (multi-window mode).
///
/// This is the content that appears in a separate dialog window.
pub fn view_third_party_dialog_content<'a>(state: &'a ThirdPartyState) -> Element<'a, Message> {
    let content = view_dialog_content(state);

    // Wrap in a styled container for the window
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
            ..Default::default()
        })
        .into()
}

/// Dialog content with header, scrollable content, and footer.
fn view_dialog_content<'a>(state: &'a ThirdPartyState) -> Element<'a, Message> {
    let header = view_header();
    let content = view_licenses_content(state.items());
    let footer = view_footer();

    column![header, content, footer,].into()
}

/// Dialog header.
fn view_header<'a>() -> Element<'a, Message> {
    row![
        container(lucide::scale().size(18)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_secondary),
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text("Third-Party Licenses")
            .size(18)
            .style(|theme: &Theme| {
                text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                }
            }),
    ]
    .align_y(Alignment::Center)
    .padding([SPACING_MD, SPACING_LG])
    .into()
}

/// Scrollable licenses content.
///
/// The markdown items are pre-parsed and cached in state, so we only
/// build the widget tree here (no expensive parsing on every frame).
fn view_licenses_content<'a>(items: &'a [markdown::Item]) -> Element<'a, Message> {
    if items.is_empty() {
        return text("No third-party licenses found.")
            .size(13)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            })
            .into();
    }

    // Use appropriate theme for markdown rendering based on dark/light mode
    // Note: markdown::view requires a concrete Theme, so we use Theme::Dark/Light based on typical usage
    // The actual theme detection happens at runtime via the container styling
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
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
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
