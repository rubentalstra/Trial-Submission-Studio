//! Variable list item component.
//!
//! A selectable list item for displaying variables in master panels.
//! Used in mapping, normalization, and validation tabs.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::theme::{
    BORDER_RADIUS_SM, GRAY_100, GRAY_200, GRAY_500, GRAY_800, GRAY_900, PRIMARY_100, PRIMARY_500,
    SPACING_SM, WHITE,
};

// =============================================================================
// VARIABLE LIST ITEM
// =============================================================================

/// A selectable variable list item for master panels.
///
/// # Example
/// ```ignore
/// VariableListItem::new("STUDYID", Message::SelectVariable(0))
///     .label("Study Identifier")
///     .leading_icon(lucide::circle_check().size(12).color(SUCCESS))
///     .trailing_badge("Req", ERROR)
///     .selected(true)
///     .view()
/// ```
pub struct VariableListItem<'a, M> {
    name: String,
    label: Option<String>,
    on_click: M,
    selected: bool,
    leading_icon: Option<Element<'a, M>>,
    trailing_badge: Option<(String, Color)>,
}

impl<'a, M: Clone + 'a> VariableListItem<'a, M> {
    /// Create a new variable list item.
    pub fn new(name: impl Into<String>, on_click: M) -> Self {
        Self {
            name: name.into(),
            label: None,
            on_click,
            selected: false,
            leading_icon: None,
            trailing_badge: None,
        }
    }

    /// Set the label/description text.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the leading icon element.
    pub fn leading_icon(mut self, icon: impl Into<Element<'a, M>>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// Set a trailing badge with text and background color.
    pub fn trailing_badge(mut self, text: impl Into<String>, color: Color) -> Self {
        self.trailing_badge = Some((text.into(), color));
        self
    }

    /// Set whether this item is selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Build the list item element.
    pub fn view(self) -> Element<'a, M> {
        let is_selected = self.selected;
        let name = self.name;
        let label = self.label;
        let on_click = self.on_click;

        // Leading icon or spacer
        let leading: Element<'a, M> = self
            .leading_icon
            .unwrap_or_else(|| Space::new().width(0.0).into());

        // Trailing badge
        let trailing: Element<'a, M> = if let Some((badge_text, badge_color)) = self.trailing_badge
        {
            container(text(badge_text).size(9).color(WHITE))
                .padding([2.0, 4.0])
                .style(move |_theme: &Theme| container::Style {
                    background: Some(badge_color.into()),
                    border: Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        } else {
            Space::new().width(0.0).into()
        };

        // Name and label text
        let name_text = text(name).size(13).color(GRAY_900);
        let label_text: Element<'a, M> = if let Some(lbl) = label {
            text(lbl).size(11).color(GRAY_500).into()
        } else {
            Space::new().height(0.0).into()
        };

        let content = row![
            leading,
            Space::new().width(SPACING_SM),
            column![name_text, label_text,].width(Length::Fill),
            trailing,
        ]
        .align_y(Alignment::Center)
        .padding([SPACING_SM, SPACING_SM]);

        button(content)
            .on_press(on_click)
            .width(Length::Fill)
            .style(move |_theme: &Theme, btn_status| {
                let bg = if is_selected {
                    Some(PRIMARY_100.into())
                } else {
                    match btn_status {
                        iced::widget::button::Status::Hovered => Some(GRAY_100.into()),
                        _ => None,
                    }
                };
                let border_color = if is_selected { PRIMARY_500 } else { GRAY_200 };
                iced::widget::button::Style {
                    background: bg,
                    text_color: GRAY_900,
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        color: border_color,
                        width: if is_selected { 1.0 } else { 0.0 },
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}

// =============================================================================
// SIMPLE LIST ITEM (for columns in SUPP, etc.)
// =============================================================================

/// A simpler selectable list item without all the variable-specific features.
///
/// # Example
/// ```ignore
/// SimpleListItem::new("AGE_SOURCE", Message::SelectColumn("AGE_SOURCE".into()))
///     .leading_icon(lucide::circle().size(10).color(GRAY_400))
///     .selected(true)
///     .view()
/// ```
pub struct SimpleListItem<'a, M> {
    text: String,
    on_click: M,
    selected: bool,
    leading_icon: Option<Element<'a, M>>,
}

impl<'a, M: Clone + 'a> SimpleListItem<'a, M> {
    /// Create a new simple list item.
    pub fn new(text: impl Into<String>, on_click: M) -> Self {
        Self {
            text: text.into(),
            on_click,
            selected: false,
            leading_icon: None,
        }
    }

    /// Set the leading icon element.
    pub fn leading_icon(mut self, icon: impl Into<Element<'a, M>>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// Set whether this item is selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Build the list item element.
    pub fn view(self) -> Element<'a, M> {
        let is_selected = self.selected;
        let display_text = self.text;
        let on_click = self.on_click;

        let bg_color = if is_selected { PRIMARY_100 } else { WHITE };
        let text_color = if is_selected { PRIMARY_500 } else { GRAY_800 };

        // Leading icon or nothing
        let leading: Element<'a, M> = self
            .leading_icon
            .unwrap_or_else(|| Space::new().width(0.0).into());

        button(
            row![
                leading,
                Space::new().width(SPACING_SM),
                text(display_text).size(13).color(text_color),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .on_press(on_click)
        .padding([8.0, 12.0])
        .width(Length::Fill)
        .style(move |_: &Theme, _status| iced::widget::button::Style {
            background: Some(bg_color.into()),
            text_color,
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }
}
