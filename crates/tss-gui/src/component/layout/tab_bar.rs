//! Tab bar navigation component.
//!
//! Horizontal tab navigation for switching between views or panels.

use iced::widget::{button, container, row, text};
use iced::{Border, Color, Element, Length, Theme};

use crate::theme::{ClinicalColors, TAB_PADDING_X, TAB_PADDING_Y};

// =============================================================================
// TAB DEFINITION
// =============================================================================

/// A tab item for the tab bar.
pub struct Tab<M> {
    /// Tab label text
    pub label: String,
    /// Message to send when tab is clicked
    pub message: M,
}

impl<M> Tab<M> {
    /// Create a new tab.
    pub fn new(label: impl Into<String>, message: M) -> Self {
        Self {
            label: label.into(),
            message,
        }
    }
}

// =============================================================================
// TAB BAR COMPONENT
// =============================================================================

/// Creates a horizontal tab bar.
///
/// Renders a row of tab buttons with the active tab highlighted.
///
/// # Arguments
///
/// * `tabs` - List of tabs to display
/// * `active_index` - Index of the currently active tab
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::{tab_bar, Tab};
///
/// let tabs = vec![
///     Tab::new("Mapping", Message::TabSelected(0)),
///     Tab::new("Transform", Message::TabSelected(1)),
///     Tab::new("Validation", Message::TabSelected(2)),
/// ];
///
/// let bar = tab_bar(tabs, state.active_tab);
/// ```
pub fn tab_bar<'a, M: Clone + 'a>(tabs: Vec<Tab<M>>, active_index: usize) -> Element<'a, M> {
    let mut tab_row = row![].spacing(0);

    for (index, tab) in tabs.into_iter().enumerate() {
        let is_active = index == active_index;
        let label = tab.label.clone();

        let tab_button = button(
            container(text(label).size(14).style(move |theme: &Theme| {
                let clinical = theme.clinical();
                let color = if is_active {
                    clinical.accent_pressed
                } else {
                    clinical.text_muted
                };
                text::Style { color: Some(color) }
            }))
            .padding([TAB_PADDING_Y, TAB_PADDING_X])
            .center_x(Length::Shrink),
        )
        .on_press(tab.message)
        .style(move |theme: &Theme, status| {
            let palette = theme.extended_palette();
            let clinical = theme.clinical();
            let accent_primary = palette.primary.base.color;
            // Create light tint of accent color for active tab background
            let accent_light = Color {
                a: 0.15,
                ..accent_primary
            };

            if is_active {
                button::Style {
                    background: Some(accent_light.into()),
                    text_color: clinical.accent_pressed,
                    border: Border {
                        color: accent_primary,
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            } else {
                let bg = match status {
                    button::Status::Hovered => Some(clinical.border_default.into()),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    text_color: clinical.text_muted,
                    border: Border::default(),
                    ..Default::default()
                }
            }
        });

        tab_row = tab_row.push(tab_button);
    }

    container(tab_row)
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let clinical = theme.clinical();
            container::Style {
                background: Some(clinical.background_secondary.into()),
                border: Border {
                    color: clinical.border_default,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}
