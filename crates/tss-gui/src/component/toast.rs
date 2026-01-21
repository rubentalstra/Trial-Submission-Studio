//! Toast notification component.
//!
//! Shows a temporary notification message that auto-dismisses after a timeout.
//! Uses the semantic color system for accessibility mode support.

use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Element, Length};
use iced_fonts::lucide;

use crate::message::Message;
use crate::theme::{SPACING_MD, SPACING_SM, SPACING_XS, colors};

/// Toast notification state.
#[derive(Debug, Clone)]
pub struct ToastState {
    /// The message to display.
    pub message: String,
    /// Toast type determines the icon and styling.
    pub toast_type: ToastType,
    /// Whether the toast has an action (e.g., "View changelog").
    pub action: Option<ToastAction>,
}

/// Type of toast notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    /// Success notification (green check icon).
    Success,
    /// Information notification (blue info icon).
    Info,
    /// Warning notification (yellow warning icon).
    Warning,
    /// Error notification (red X icon).
    Error,
}

impl ToastType {
    /// Get the semantic color for this toast type.
    pub fn color(&self) -> iced::Color {
        let c = colors();
        match self {
            ToastType::Success => c.status_success,
            ToastType::Info => c.status_info,
            ToastType::Warning => c.status_warning,
            ToastType::Error => c.status_error,
        }
    }
}

/// Optional action for the toast.
#[derive(Debug, Clone)]
pub struct ToastAction {
    /// Label for the action button.
    pub label: String,
    /// Message to send when action is clicked.
    pub on_click: ToastActionType,
}

/// Types of toast actions.
#[derive(Debug, Clone)]
pub enum ToastActionType {
    /// Open the update dialog to view changelog.
    ViewChangelog,
    /// Open a URL in the browser.
    OpenUrl(String),
}

/// Toast message for handling toast events.
#[derive(Debug, Clone)]
pub enum ToastMessage {
    /// Dismiss the toast.
    Dismiss,
    /// Perform the toast action.
    Action,
    /// Show a new toast (used internally).
    Show(ToastState),
}

impl ToastState {
    /// Creates a new success toast for a completed update.
    pub fn update_success(version: &str) -> Self {
        Self {
            message: format!("Updated to v{version}"),
            toast_type: ToastType::Success,
            action: Some(ToastAction {
                label: "View changelog".to_string(),
                on_click: ToastActionType::ViewChangelog,
            }),
        }
    }
}

/// Renders a toast notification.
///
/// The toast appears at the bottom-right of the screen and can be dismissed.
pub fn view_toast(state: &ToastState) -> Element<'_, Message> {
    let c = colors();
    let icon_color = state.toast_type.color();
    let text_color = c.text_secondary;
    let bg_color = c.background_secondary;
    let border_color = c.border_default;
    let shadow_color = c.shadow;

    let icon = match state.toast_type {
        ToastType::Success => lucide::circle_check().size(18).color(icon_color),
        ToastType::Info => lucide::info().size(18).color(icon_color),
        ToastType::Warning => lucide::triangle_alert().size(18).color(icon_color),
        ToastType::Error => lucide::circle_x().size(18).color(icon_color),
    };

    let message_text = text(&state.message).size(14).color(text_color);

    let mut content = row![icon, Space::new().width(SPACING_SM), message_text,]
        .align_y(Alignment::Center)
        .spacing(SPACING_XS);

    // Add action button if present
    if let Some(action) = &state.action {
        let action_btn = button(text(&action.label).size(12))
            .on_press(Message::Toast(ToastMessage::Action))
            .padding([SPACING_XS, SPACING_SM]);

        content = content
            .push(Space::new().width(SPACING_MD))
            .push(action_btn);
    }

    // Add dismiss button
    let dismiss_btn = button(lucide::x().size(14))
        .on_press(Message::Toast(ToastMessage::Dismiss))
        .padding(SPACING_XS);

    content = content
        .push(Space::new().width(SPACING_SM))
        .push(dismiss_btn);

    container(content)
        .padding([SPACING_SM, SPACING_MD])
        .width(Length::Shrink)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            border: iced::Border {
                color: border_color,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow {
                color: shadow_color,
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        })
        .into()
}
