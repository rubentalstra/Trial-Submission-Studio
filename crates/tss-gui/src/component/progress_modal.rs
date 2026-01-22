//! Progress modal component.
//!
//! A modal dialog showing progress of a long-running operation with optional
//! cancellation support.

use iced::widget::{Space, button, center, column, container, opaque, progress_bar, stack, text};
use iced::{Border, Element, Length, Shadow, Theme, Vector};

use crate::theme::{
    BORDER_RADIUS_LG, ClinicalColors, MODAL_WIDTH_SM, SPACING_LG, SPACING_MD, button_secondary,
    progress_bar_primary,
};

// =============================================================================
// PROGRESS MODAL
// =============================================================================

/// Creates a progress modal with optional cancel button.
///
/// Shows a progress bar, status message, and percentage. Use for export,
/// loading, or other long-running operations.
///
/// # Arguments
///
/// * `base` - The background content (entire app view)
/// * `title` - Modal title (e.g., "Exporting...")
/// * `message` - Current status message (e.g., "Processing DM domain...")
/// * `progress` - Progress value from 0.0 to 1.0
/// * `on_cancel` - Optional message for cancel button (None hides the button)
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::progress_modal;
///
/// let view = progress_modal(
///     base_content,
///     "Exporting Domains",
///     "Processing DM domain...",
///     0.45,
///     Some(Message::CancelExport),
/// );
/// ```
pub fn progress_modal<'a, M: Clone + 'a>(
    base: Element<'a, M>,
    title: &'a str,
    message: &'a str,
    progress: f32,
    on_cancel: Option<M>,
) -> Element<'a, M> {
    // Backdrop overlay
    let backdrop = container(column![])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().backdrop.into()),
            ..Default::default()
        });

    // Progress bar
    let progress_bar_widget = progress_bar(0.0..=1.0, progress)
        .girth(8.0)
        .style(progress_bar_primary);

    let title_owned = title.to_string();
    let message_owned = message.to_string();

    // Percentage text
    let percentage = text(format!("{}%", (progress * 100.0) as u32))
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_muted),
        });

    // Build content column
    let mut content = column![
        text(title_owned)
            .size(18)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        Space::new().height(SPACING_MD),
        text(message_owned)
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().height(SPACING_MD),
        progress_bar_widget,
        Space::new().height(8.0),
        container(percentage)
            .width(Length::Fill)
            .center_x(Length::Shrink),
    ]
    .spacing(0);

    // Add cancel button if provided
    if let Some(cancel_msg) = on_cancel {
        content = content.push(Space::new().height(SPACING_LG));
        content = content.push(
            container(
                button(text("Cancel"))
                    .on_press(cancel_msg)
                    .padding([10.0, 24.0])
                    .style(button_secondary),
            )
            .width(Length::Fill)
            .center_x(Length::Shrink),
        );
    }

    // Modal dialog box
    let dialog = container(content)
        .width(Length::Fixed(MODAL_WIDTH_SM))
        .padding(SPACING_LG)
        .style(|theme: &Theme| {
            let clinical = theme.clinical();
            container::Style {
                background: Some(clinical.background_elevated.into()),
                border: Border {
                    radius: BORDER_RADIUS_LG.into(),
                    width: 1.0,
                    color: clinical.border_default,
                },
                shadow: Shadow {
                    color: clinical.shadow_strong,
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            }
        });

    // Stack layers
    stack![base, opaque(backdrop), center(dialog),].into()
}

/// Creates a simple loading modal without progress indicator.
///
/// Shows a spinner-like message for indeterminate loading states.
///
/// # Arguments
///
/// * `base` - The background content
/// * `message` - Loading message (e.g., "Loading study...")
pub fn loading_modal<'a, M: 'a>(base: Element<'a, M>, message: &'a str) -> Element<'a, M> {
    let message_owned = message.to_string();

    // Backdrop overlay
    let backdrop = container(column![])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().backdrop.into()),
            ..Default::default()
        });

    // Indeterminate progress bar (full width, animated would require subscription)
    let progress_bar_widget = progress_bar(0.0..=1.0, 0.5)
        .girth(4.0)
        .style(progress_bar_primary);

    // Content
    let content = column![
        text(message_owned)
            .size(16)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().height(SPACING_MD),
        progress_bar_widget,
    ]
    .spacing(0);

    // Modal dialog box
    let dialog = container(content)
        .width(Length::Fixed(280.0))
        .padding(SPACING_LG)
        .style(|theme: &Theme| {
            let clinical = theme.clinical();
            container::Style {
                background: Some(clinical.background_elevated.into()),
                border: Border {
                    radius: BORDER_RADIUS_LG.into(),
                    width: 1.0,
                    color: clinical.border_default,
                },
                shadow: Shadow {
                    color: clinical.shadow_strong,
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            }
        });

    // Stack layers
    stack![base, opaque(backdrop), center(dialog),].into()
}
