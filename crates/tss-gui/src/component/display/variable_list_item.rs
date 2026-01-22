//! Variable list item component.
//!
//! A selectable list item for displaying variables in master panels.
//! Used in mapping, normalization, and validation tabs.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::theme::{BORDER_RADIUS_SM, ClinicalColors, SPACING_SM};

// =============================================================================
// BADGE COLOR ENUM
// =============================================================================

/// Badge color specification - either a direct color or a theme-derived closure.
enum BadgeColor {
    Direct(Color),
    Themed(fn(&Theme) -> Color),
}

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
    trailing_badge: Option<(String, BadgeColor)>,
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

    /// Set a trailing badge with text and direct background color.
    pub fn trailing_badge(mut self, text: impl Into<String>, color: Color) -> Self {
        self.trailing_badge = Some((text.into(), BadgeColor::Direct(color)));
        self
    }

    /// Set a trailing badge with text and theme-derived background color.
    pub fn trailing_badge_themed(
        mut self,
        text: impl Into<String>,
        color_fn: fn(&Theme) -> Color,
    ) -> Self {
        self.trailing_badge = Some((text.into(), BadgeColor::Themed(color_fn)));
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
            match badge_color {
                BadgeColor::Direct(color) => {
                    container(text(badge_text).size(9).style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_on_accent),
                    }))
                    .padding([2.0, 4.0])
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(color.into()),
                        border: Border {
                            radius: 2.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .into()
                }
                BadgeColor::Themed(color_fn) => {
                    container(text(badge_text).size(9).style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_on_accent),
                    }))
                    .padding([2.0, 4.0])
                    .style(move |theme: &Theme| container::Style {
                        background: Some(color_fn(theme).into()),
                        border: Border {
                            radius: 2.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .into()
                }
            }
        } else {
            Space::new().width(0.0).into()
        };

        // Name and label text
        let name_text = text(name).size(13).style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });
        let label_text: Element<'a, M> = if let Some(lbl) = label {
            text(lbl)
                .size(11)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                })
                .into()
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
            .style(move |theme: &Theme, btn_status| {
                let palette = theme.extended_palette();
                let clinical = theme.clinical();

                let bg = if is_selected {
                    Some(clinical.accent_primary_light.into())
                } else {
                    match btn_status {
                        iced::widget::button::Status::Hovered => {
                            Some(clinical.background_secondary.into())
                        }
                        _ => None,
                    }
                };
                let border_color = if is_selected {
                    palette.primary.base.color
                } else {
                    clinical.border_default
                };
                iced::widget::button::Style {
                    background: bg,
                    text_color: palette.background.base.text,
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

        // Leading icon or nothing
        let leading: Element<'a, M> = self
            .leading_icon
            .unwrap_or_else(|| Space::new().width(0.0).into());

        button(
            row![
                leading,
                Space::new().width(SPACING_SM),
                text(display_text).size(13).style(move |theme: &Theme| {
                    let palette = theme.extended_palette();
                    let text_color = if is_selected {
                        palette.primary.base.color
                    } else {
                        palette.background.base.text
                    };
                    text::Style {
                        color: Some(text_color),
                    }
                }),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .on_press(on_click)
        .padding([8.0, 12.0])
        .width(Length::Fill)
        .style(move |theme: &Theme, _status| {
            let palette = theme.extended_palette();
            let clinical = theme.clinical();
            let bg_color = if is_selected {
                clinical.accent_primary_light
            } else {
                clinical.background_elevated
            };
            let text_color = if is_selected {
                palette.primary.base.color
            } else {
                palette.background.base.text
            };
            iced::widget::button::Style {
                background: Some(bg_color.into()),
                text_color,
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
    }
}
