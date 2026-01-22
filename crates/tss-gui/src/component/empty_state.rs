//! Empty, loading, and error state components.
//!
//! Standardized feedback states for when there's no data to display,
//! an operation is in progress, or an error occurred.
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::{EmptyState, LoadingState, ErrorState};
//! use iced_fonts::lucide;
//!
//! // Empty state with action
//! EmptyState::new(lucide::folder().size(48), "No Study Loaded")
//!     .description("Open a study folder to get started")
//!     .action("Open Folder", Message::OpenFolder)
//!     .centered()
//!     .view()
//!
//! // Loading state
//! LoadingState::new("Building Preview")
//!     .description("Applying mappings and normalization rules...")
//!     .centered()
//!     .view()
//!
//! // Error state with retry
//! ErrorState::new("Preview Build Failed")
//!     .message(&error_text)
//!     .retry(Message::Retry)
//!     .centered()
//!     .view()
//! ```

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, button_primary,
};

// =============================================================================
// EMPTY STATE
// =============================================================================

/// Empty state with icon, title, description, and optional action.
///
/// Use when there's no data to display or user needs to take an action.
pub struct EmptyState<'a, M> {
    icon: Element<'a, M>,
    title: String,
    description: Option<String>,
    action: Option<(String, M)>,
    centered: bool,
    height: Option<f32>,
}

impl<'a, M: Clone + 'a> EmptyState<'a, M> {
    /// Create a new empty state with icon and title.
    pub fn new(icon: impl Into<Element<'a, M>>, title: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
            description: None,
            action: None,
            centered: false,
            height: None,
        }
    }

    /// Add a description below the title.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add an action button.
    pub fn action(mut self, label: impl Into<String>, message: M) -> Self {
        self.action = Some((label.into(), message));
        self
    }

    /// Center the content in a full-size container.
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    /// Set a fixed height (useful when not centered).
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }

    /// Build the element.
    pub fn view(self) -> Element<'a, M> {
        let title_text = text(self.title)
            .size(16)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            });

        let mut content = column![self.icon, Space::new().height(SPACING_MD),].push(title_text);

        if let Some(desc) = self.description {
            content =
                content
                    .push(Space::new().height(SPACING_SM))
                    .push(text(desc).size(13).style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted),
                    }));
        }

        if let Some((label, message)) = self.action {
            content = content.push(Space::new().height(SPACING_LG)).push(
                button(text(label).size(14).style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_on_accent),
                }))
                .on_press(message)
                .padding([10.0, 24.0])
                .style(button_primary),
            );
        }

        let content = content.align_x(Alignment::Center);

        if self.centered {
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Shrink)
                .center_y(Length::Shrink)
                .into()
        } else if let Some(h) = self.height {
            container(content)
                .width(Length::Fill)
                .height(Length::Fixed(h))
                .center_x(Length::Shrink)
                .center_y(Length::Shrink)
                .into()
        } else {
            container(content)
                .width(Length::Fill)
                .center_x(Length::Shrink)
                .into()
        }
    }
}

// =============================================================================
// LOADING STATE
// =============================================================================

/// Loading state with spinner and message.
///
/// Use when an async operation is in progress.
pub struct LoadingState {
    title: String,
    description: Option<String>,
    centered: bool,
}

impl LoadingState {
    /// Create a new loading state with title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            centered: false,
        }
    }

    /// Add a description below the title.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Center the content in a full-size container.
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    /// Build the element.
    pub fn view<'a, M: 'a>(self) -> Element<'a, M> {
        // For icons, we need to wrap in a container with style since lucide icons
        // use .color() which doesn't have a theme closure
        let icon = container(lucide::loader().size(40)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().primary.base.color),
            ..Default::default()
        });

        let title_text = text(self.title)
            .size(18)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            });

        let mut content =
            column![icon, Space::new().height(SPACING_LG), title_text,].align_x(Alignment::Center);

        if let Some(desc) = self.description {
            content =
                content
                    .push(Space::new().height(SPACING_SM))
                    .push(text(desc).size(13).style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted),
                    }));
        }

        if self.centered {
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Shrink)
                .center_y(Length::Shrink)
                .into()
        } else {
            container(content)
                .width(Length::Fill)
                .center_x(Length::Shrink)
                .into()
        }
    }
}

// =============================================================================
// ERROR STATE
// =============================================================================

/// Error state with message and optional retry action.
///
/// Use when an operation failed.
pub struct ErrorState<M> {
    title: String,
    message: Option<String>,
    retry: Option<M>,
    centered: bool,
}

impl<M: Clone> ErrorState<M> {
    /// Create a new error state with title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: None,
            retry: None,
            centered: false,
        }
    }

    /// Set the error message (shown in a container).
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Add a retry button.
    pub fn retry(mut self, message: M) -> Self {
        self.retry = Some(message);
        self
    }

    /// Center the content in a full-size container.
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    /// Build the element.
    pub fn view<'a>(self) -> Element<'a, M>
    where
        M: 'a,
    {
        // Error icon wrapped in container for theme-aware color
        let error_icon =
            container(lucide::circle_alert().size(48)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().danger.base.color),
                ..Default::default()
            });

        let title_text = text(self.title)
            .size(18)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            });

        let mut content = column![error_icon, Space::new().height(SPACING_LG), title_text,]
            .align_x(Alignment::Center)
            .max_width(400.0);

        if let Some(msg) = self.message {
            let msg_text = text(msg).size(12).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            });
            content = content.push(Space::new().height(SPACING_SM)).push(
                container(msg_text)
                    .padding(SPACING_MD)
                    .style(|theme: &Theme| container::Style {
                        background: Some(theme.clinical().background_secondary.into()),
                        border: Border {
                            radius: BORDER_RADIUS_SM.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            );
        }

        if let Some(retry_msg) = self.retry {
            // Retry icon wrapped in container for theme-aware color
            let retry_icon =
                container(lucide::refresh_cw().size(14)).style(|theme: &Theme| container::Style {
                    text_color: Some(theme.clinical().text_on_accent),
                    ..Default::default()
                });

            let retry_label = text("Retry").size(14).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_on_accent),
            });

            content = content.push(Space::new().height(SPACING_LG)).push(
                button(
                    row![retry_icon, Space::new().width(SPACING_SM), retry_label,]
                        .align_y(Alignment::Center),
                )
                .on_press(retry_msg)
                .padding([10.0, 24.0])
                .style(button_primary),
            );
        }

        if self.centered {
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Shrink)
                .center_y(Length::Shrink)
                .into()
        } else {
            container(content)
                .width(Length::Fill)
                .center_x(Length::Shrink)
                .into()
        }
    }
}

// =============================================================================
// NO FILTERED RESULTS
// =============================================================================

/// State when filters return no results.
///
/// Use when a search or filter yields zero matches.
pub struct NoFilteredResults<M> {
    filter_name: String,
    hint: Option<String>,
    clear_action: Option<M>,
    height: Option<f32>,
}

impl<M: Clone> NoFilteredResults<M> {
    /// Create a new no-results state.
    ///
    /// # Arguments
    /// * `filter_name` - What was filtered (e.g., "errors", "variables")
    pub fn new(filter_name: impl Into<String>) -> Self {
        Self {
            filter_name: filter_name.into(),
            hint: None,
            clear_action: None,
            height: None,
        }
    }

    /// Add a hint about what to do.
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Add a clear filters button.
    pub fn clear_action(mut self, message: M) -> Self {
        self.clear_action = Some(message);
        self
    }

    /// Set a fixed height.
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }

    /// Build the element.
    pub fn view<'a>(self) -> Element<'a, M>
    where
        M: 'a,
    {
        // Search icon wrapped in container for theme-aware color
        let search_icon =
            container(lucide::search().size(32)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_disabled),
                ..Default::default()
            });

        let no_results_text = text(format!("No {} found", self.filter_name))
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            });

        let mut content = column![
            search_icon,
            Space::new().height(SPACING_MD),
            no_results_text,
        ]
        .align_x(Alignment::Center);

        if let Some(hint) = self.hint {
            content =
                content
                    .push(Space::new().height(SPACING_SM))
                    .push(text(hint).size(12).style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted),
                    }));
        }

        if let Some(clear_msg) = self.clear_action {
            content = content.push(Space::new().height(SPACING_MD)).push(
                button(
                    text("Clear filters")
                        .size(12)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_secondary),
                        }),
                )
                .on_press(clear_msg)
                .padding([6.0, 12.0]),
            );
        }

        let height = self.height.map_or(Length::Fixed(200.0), Length::Fixed);

        container(content)
            .width(Length::Fill)
            .height(height)
            .center_x(Length::Shrink)
            .center_y(Length::Shrink)
            .into()
    }
}
