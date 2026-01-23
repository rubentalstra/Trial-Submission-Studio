//! Detail panel component.
//!
//! A comprehensive detail panel that consolidates common detail view patterns:
//! header with title/subtitle, metadata sections, and action buttons.
//!
//! # Features
//!
//! - Header with title, subtitle, and optional badge
//! - Multiple content sections with optional dividers
//! - Action button area
//! - Automatic scrolling
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::panels::DetailPanel;
//!
//! // Full-featured detail panel
//! DetailPanel::new("STUDYID")
//!     .subtitle("Study Identifier")
//!     .badge("Required", Color::from_rgb(0.8, 0.2, 0.2))
//!     .section(metadata_card)
//!     .section_titled("Mapping Status", status_card)
//!     .actions(action_buttons)
//!     .view();
//!
//! // Simple detail panel
//! DetailPanel::new("Item Name")
//!     .subtitle("Item description")
//!     .section(content)
//!     .view();
//! ```

use iced::widget::{Column, Space, column, container, row, rule, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::theme::{ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS};

// =============================================================================
// DETAIL PANEL
// =============================================================================

/// A comprehensive detail panel component.
///
/// Provides a builder pattern for creating detail panels with header,
/// content sections, and actions.
pub struct DetailPanel<'a, M> {
    /// Panel title
    title: String,
    /// Optional subtitle
    subtitle: Option<String>,
    /// Optional badge: (text, background_color)
    badge: Option<(String, Color)>,
    /// Optional badge with icon: (icon, text, background_color)
    badge_with_icon: Option<(Element<'a, M>, String, Color)>,
    /// Content sections: (optional_title, content)
    sections: Vec<(Option<String>, Element<'a, M>)>,
    /// Actions area content
    actions: Option<Element<'a, M>>,
    /// Whether to show divider after header
    show_header_divider: bool,
    /// Whether to show dividers between sections
    show_section_dividers: bool,
    /// Whether to wrap in scrollable
    scrollable: bool,
}

impl<'a, M: 'a> DetailPanel<'a, M> {
    /// Create a new detail panel with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            badge: None,
            badge_with_icon: None,
            sections: Vec::new(),
            actions: None,
            show_header_divider: true,
            show_section_dividers: false,
            scrollable: true,
        }
    }

    /// Set the subtitle text.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Add a simple badge with text and background color.
    pub fn badge(mut self, text: impl Into<String>, color: Color) -> Self {
        self.badge = Some((text.into(), color));
        self
    }

    /// Add a badge with icon, text, and background color.
    pub fn badge_with_icon(
        mut self,
        icon: impl Into<Element<'a, M>>,
        text: impl Into<String>,
        color: Color,
    ) -> Self {
        self.badge_with_icon = Some((icon.into(), text.into(), color));
        self
    }

    /// Add a content section without a title.
    pub fn section(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.sections.push((None, content.into()));
        self
    }

    /// Add a content section with a title.
    pub fn section_titled(
        mut self,
        title: impl Into<String>,
        content: impl Into<Element<'a, M>>,
    ) -> Self {
        self.sections.push((Some(title.into()), content.into()));
        self
    }

    /// Set the actions area content.
    pub fn actions(mut self, actions: impl Into<Element<'a, M>>) -> Self {
        self.actions = Some(actions.into());
        self
    }

    /// Hide the divider after the header.
    pub fn no_header_divider(mut self) -> Self {
        self.show_header_divider = false;
        self
    }

    /// Show dividers between sections.
    pub fn with_section_dividers(mut self) -> Self {
        self.show_section_dividers = true;
        self
    }

    /// Disable scrolling (caller handles it).
    pub fn no_scroll(mut self) -> Self {
        self.scrollable = false;
        self
    }

    /// Build the detail panel element.
    pub fn view(self) -> Element<'a, M> {
        let Self {
            title,
            subtitle,
            badge,
            badge_with_icon,
            sections,
            actions,
            show_header_divider,
            show_section_dividers,
            scrollable: is_scrollable,
        } = self;

        let mut content: Column<'a, M> = column![];

        // Header
        content = content.push(build_header(title, subtitle, badge, badge_with_icon));

        // Header divider
        if show_header_divider {
            content = content.push(Space::new().height(SPACING_MD));
            content = content.push(rule::horizontal(1));
            content = content.push(Space::new().height(SPACING_MD));
        } else {
            content = content.push(Space::new().height(SPACING_MD));
        }

        // Sections
        let section_count = sections.len();
        for (idx, (section_title, section_content)) in sections.into_iter().enumerate() {
            // Section title if present
            if let Some(title_text) = section_title {
                content = content.push(
                    text(title_text)
                        .size(14)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_secondary),
                        })
                        .font(iced::Font {
                            weight: iced::font::Weight::Semibold,
                            ..Default::default()
                        }),
                );
                content = content.push(Space::new().height(SPACING_SM));
            }

            // Section content
            content = content.push(section_content);

            // Section divider (if not last section and dividers enabled)
            if show_section_dividers && idx < section_count - 1 {
                content = content.push(Space::new().height(SPACING_MD));
                content = content.push(rule::horizontal(1));
                content = content.push(Space::new().height(SPACING_MD));
            } else {
                content = content.push(Space::new().height(SPACING_LG));
            }
        }

        // Actions
        if let Some(actions_content) = actions {
            content = content.push(actions_content);
            content = content.push(Space::new().height(SPACING_MD));
        }

        // Wrap in scrollable if enabled
        if is_scrollable {
            scrollable(content).height(Length::Fill).into()
        } else {
            content.into()
        }
    }
}

/// Build the header element.
fn build_header<'a, M: 'a>(
    title: String,
    subtitle: Option<String>,
    badge: Option<(String, Color)>,
    badge_with_icon: Option<(Element<'a, M>, String, Color)>,
) -> Element<'a, M> {
    let title_element = text(title).size(20).style(|theme: &Theme| text::Style {
        color: Some(theme.extended_palette().background.base.text),
    });

    // Subtitle
    let subtitle_el: Element<'a, M> = if let Some(sub) = subtitle {
        column![
            Space::new().height(SPACING_XS),
            text(sub).size(14).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        ]
        .into()
    } else {
        Space::new().height(0.0).into()
    };

    // Badge (with icon takes priority over simple badge)
    let badge_el: Element<'a, M> = if let Some((icon, badge_text, bg_color)) = badge_with_icon {
        container(
            row![
                icon,
                Space::new().width(SPACING_XS),
                text(badge_text)
                    .size(11)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_on_accent),
                    }),
            ]
            .align_y(Alignment::Center),
        )
        .padding([4.0, 10.0])
        .style(move |_theme: &Theme| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else if let Some((badge_text, bg_color)) = badge {
        container(
            text(badge_text)
                .size(11)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_on_accent),
                }),
        )
        .padding([4.0, 10.0])
        .style(move |_theme: &Theme| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::new().width(0.0).into()
    };

    column![
        row![title_element, Space::new().width(Length::Fill), badge_el,].align_y(Alignment::Center),
        subtitle_el,
    ]
    .into()
}

// =============================================================================
// EMPTY DETAIL VIEW
// =============================================================================

/// An empty state for detail panels when nothing is selected.
///
/// # Example
///
/// ```ignore
/// use tss_gui::component::panels::EmptyDetailView;
///
/// EmptyDetailView::new("Select a variable")
///     .subtitle("Choose a variable from the list to view details")
///     .view();
/// ```
pub struct EmptyDetailView {
    message: String,
    subtitle: Option<String>,
}

impl EmptyDetailView {
    /// Create a new empty detail view.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            subtitle: None,
        }
    }

    /// Add a subtitle/description.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Build the empty detail view element.
    pub fn view<'a, M: 'a>(self) -> Element<'a, M> {
        let mut content = column![
            text(self.message)
                .size(16)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
        ]
        .align_x(Alignment::Center);

        if let Some(subtitle) = self.subtitle {
            content = content.push(Space::new().height(SPACING_SM));
            content = content.push(text(subtitle).size(13).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }));
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Shrink)
            .center_y(Length::Shrink)
            .into()
    }
}
