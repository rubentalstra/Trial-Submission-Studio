//! Modal dialog overlay component.
//!
//! Provides modal dialogs with backdrop, title, content, and action buttons.

use iced::widget::{button, center, column, container, opaque, row, space, stack, text};
use iced::{Border, Element, Length, Shadow, Theme, Vector};
use iced_fonts::lucide;

use crate::theme::{
    BORDER_RADIUS_LG, ClinicalColors, MODAL_WIDTH_MD, SPACING_LG, SPACING_MD, SPACING_SM,
    button_ghost, button_primary, button_secondary,
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
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().backdrop.into()),
            ..Default::default()
        });

    let title_owned = title.to_string();

    // Header with title and close button
    let header = row![
        text(title_owned)
            .size(18)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        space::horizontal(),
        button(
            container(lucide::x().size(20)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_muted),
                ..Default::default()
            })
        )
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
#[allow(dead_code)]
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
