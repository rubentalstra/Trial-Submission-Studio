//! Modal dialog overlay component.
//!
//! Provides modal dialogs with backdrop, title, content, and action buttons.

use iced::widget::{button, center, column, container, opaque, row, space, stack, text};
use iced::{Border, Element, Length, Shadow, Vector};
use iced_fonts::lucide;

use crate::theme::{
    BACKDROP, BORDER_RADIUS_LG, GRAY_200, GRAY_500, GRAY_900, MODAL_WIDTH_MD, SHADOW_STRONG,
    SPACING_LG, SPACING_MD, SPACING_SM, WHITE, button_ghost, button_primary, button_secondary,
};

// =============================================================================
// MODAL DIALOG
// =============================================================================

/// Creates a modal dialog overlay.
///
/// The modal appears centered on top of the base content with a semi-transparent
/// backdrop. Clicking the backdrop does NOT close the modal - use the close button.
///
/// # Arguments
///
/// * `base` - The background content (entire app view)
/// * `title` - Modal title text
/// * `content` - Modal body content
/// * `on_close` - Message to send when close button is clicked
/// * `actions` - List of action buttons for the footer
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::modal;
///
/// let view = modal(
///     base_content,
///     "Confirm Action",
///     text("Are you sure?").into(),
///     Message::CloseModal,
///     vec![
///         cancel_button,
///         confirm_button,
///     ],
/// );
/// ```
pub fn modal<'a, M: Clone + 'a>(
    base: Element<'a, M>,
    title: &'a str,
    content: Element<'a, M>,
    on_close: M,
    actions: Vec<Element<'a, M>>,
) -> Element<'a, M> {
    // Backdrop overlay
    let backdrop = container(column![])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(BACKDROP.into()),
            ..Default::default()
        });

    // Header with title and close button
    let header = row![
        text(title).size(18).color(GRAY_900),
        space::horizontal(),
        button(lucide::x().size(20).color(GRAY_500))
            .on_press(on_close)
            .padding([4.0, 8.0])
            .style(button_ghost),
    ]
    .align_y(iced::Alignment::Center);

    // Action buttons row
    let action_row = {
        let mut r = row![space::horizontal()].spacing(SPACING_SM);
        for action in actions {
            r = r.push(action);
        }
        r
    };

    // Modal dialog box
    let dialog = container(
        column![
            header,
            container(content).padding([SPACING_MD, 0.0]),
            action_row,
        ]
        .spacing(SPACING_MD),
    )
    .width(Length::Fixed(MODAL_WIDTH_MD))
    .padding(SPACING_LG)
    .style(|_theme| container::Style {
        background: Some(WHITE.into()),
        border: Border {
            radius: BORDER_RADIUS_LG.into(),
            width: 1.0,
            color: GRAY_200,
        },
        shadow: Shadow {
            color: SHADOW_STRONG,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    });

    // Stack layers: base -> backdrop -> dialog
    stack![base, opaque(backdrop), center(dialog),].into()
}

/// Creates a simple confirmation modal.
///
/// A pre-built modal with a message and confirm/cancel buttons.
///
/// # Arguments
///
/// * `base` - The background content
/// * `title` - Modal title
/// * `message` - Confirmation message
/// * `confirm_label` - Label for the confirm button
/// * `on_confirm` - Message when confirmed
/// * `on_cancel` - Message when cancelled
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::confirm_modal;
///
/// let view = confirm_modal(
///     base_content,
///     "Delete Variable",
///     "Are you sure you want to remove this mapping?",
///     "Delete",
///     Message::ConfirmDelete,
///     Message::CancelDelete,
/// );
/// ```
pub fn confirm_modal<'a, M: Clone + 'a>(
    base: Element<'a, M>,
    title: &'a str,
    message: &'a str,
    confirm_label: &'a str,
    on_confirm: M,
    on_cancel: M,
) -> Element<'a, M> {
    let content = text(message).into();

    let cancel_btn: Element<'a, M> = button(text("Cancel"))
        .on_press(on_cancel.clone())
        .padding([10.0, 20.0])
        .style(button_secondary)
        .into();

    let confirm_btn: Element<'a, M> = button(text(confirm_label))
        .on_press(on_confirm)
        .padding([10.0, 20.0])
        .style(button_primary)
        .into();

    modal(
        base,
        title,
        content,
        on_cancel,
        vec![cancel_btn, confirm_btn],
    )
}

/// Creates an info/alert modal with a single OK button.
///
/// # Arguments
///
/// * `base` - The background content
/// * `title` - Modal title
/// * `message` - Alert message
/// * `on_close` - Message when OK is clicked
pub fn alert_modal<'a, M: Clone + 'a>(
    base: Element<'a, M>,
    title: &'a str,
    message: &'a str,
    on_close: M,
) -> Element<'a, M> {
    let content = text(message).into();

    let ok_btn: Element<'a, M> = button(text("OK"))
        .on_press(on_close.clone())
        .padding([10.0, 20.0])
        .style(button_primary)
        .into();

    modal(base, title, content, on_close, vec![ok_btn])
}
