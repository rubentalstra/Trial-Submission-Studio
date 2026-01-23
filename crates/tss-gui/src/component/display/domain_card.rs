//! Domain card component for the home view.
//!
//! Enhanced domain card showing progress and validation status.
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::DomainCard;
//!
//! DomainCard::new("DM", "Demographics", Message::Click)
//!     .row_count(150)
//!     .progress(0.85)
//!     .validation(2, 1)  // 2 warnings, 1 error
//!     .view()
//! ```

use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::theme::{BORDER_RADIUS_MD, ClinicalColors, SPACING_MD, SPACING_SM, SPACING_XS};

/// Enhanced domain card with progress and validation badges.
pub struct DomainCard<M> {
    code: String,
    name: String,
    row_count: usize,
    progress: f32,
    validation: Option<(usize, usize)>, // (warnings, errors)
    on_click: M,
}

impl<M: Clone + 'static> DomainCard<M> {
    /// Create a new domain card.
    pub fn new(code: impl Into<String>, name: impl Into<String>, on_click: M) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            row_count: 0,
            progress: 0.0,
            validation: None,
            on_click,
        }
    }

    /// Set the row count to display.
    pub fn row_count(mut self, count: usize) -> Self {
        self.row_count = count;
        self
    }

    /// Set the mapping progress (0.0 to 1.0).
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Set validation counts (warnings, errors).
    pub fn validation(mut self, warnings: usize, errors: usize) -> Self {
        self.validation = Some((warnings, errors));
        self
    }

    /// Set optional validation - None means still loading.
    pub fn validation_opt(mut self, v: Option<(usize, usize)>) -> Self {
        self.validation = v;
        self
    }

    /// Build the domain card element.
    pub fn view(self) -> Element<'static, M> {
        let code = self.code;
        let name = self.name;
        let row_count = self.row_count;
        let progress = self.progress;
        let validation = self.validation;
        let on_click = self.on_click;

        // Domain badge
        let badge: Element<'static, M> =
            container(text(code).size(14).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_on_accent),
            }))
            .padding([4.0, 12.0])
            .style(|theme: &Theme| container::Style {
                background: Some(theme.extended_palette().primary.base.color.into()),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into();

        let name_text = text(name).size(14).style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });
        let row_text = text(format!("{} rows", row_count))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            });

        let top_row = row![
            badge,
            Space::new().width(SPACING_SM),
            name_text,
            Space::new().width(Length::Fill),
            row_text,
        ]
        .align_y(Alignment::Center);

        // Progress bar (inline)
        let progress_bar: Element<'static, M> = create_progress_bar(progress, 6.0);

        // Validation badges with themed colors
        let validation_badges: Element<'static, M> = match validation {
            Some((warnings, errors)) => {
                let mut badges = row![].spacing(SPACING_XS).align_y(Alignment::Center);

                if warnings > 0 {
                    badges = badges.push(validation_badge(warnings, true));
                }

                if errors > 0 {
                    badges = badges.push(validation_badge(errors, false));
                }

                badges.into()
            }
            None => Space::new().width(0.0).into(),
        };

        let bottom_row = row![
            container(progress_bar).width(Length::Fill),
            Space::new().width(SPACING_SM),
            validation_badges,
        ]
        .align_y(Alignment::Center);

        let content = iced::widget::column![top_row, Space::new().height(SPACING_SM), bottom_row,]
            .padding(SPACING_MD);

        button(content)
            .on_press(on_click)
            .padding(0.0)
            .width(Length::Fill)
            .style(|theme: &Theme, status| {
                let clinical = theme.clinical();
                let bg = match status {
                    iced::widget::button::Status::Hovered => clinical.background_secondary,
                    _ => clinical.background_elevated,
                };
                iced::widget::button::Style {
                    background: Some(bg.into()),
                    text_color: theme.extended_palette().background.base.text,
                    border: Border {
                        radius: BORDER_RADIUS_MD.into(),
                        color: clinical.border_default,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}

/// Create an inline progress bar element.
fn create_progress_bar<M: 'static>(value: f32, height: f32) -> Element<'static, M> {
    let percentage = (value * 100.0).round() as u32;

    let fill_width = if value > 0.0 {
        Length::FillPortion((value * 100.0).max(1.0) as u16)
    } else {
        Length::Fixed(0.0)
    };

    let empty_width = if value < 1.0 {
        Length::FillPortion(((1.0 - value) * 100.0).max(1.0) as u16)
    } else {
        Length::Fixed(0.0)
    };

    let is_complete = value >= 1.0;

    let fill: Element<'static, M> = container(Space::new())
        .width(fill_width)
        .height(height)
        .style(move |theme: &Theme| {
            let actual_fill = if is_complete {
                theme.extended_palette().success.base.color
            } else {
                theme.extended_palette().primary.base.color
            };
            container::Style {
                background: Some(actual_fill.into()),
                border: Border {
                    radius: (height / 2.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into();

    let empty: Element<'static, M> = Space::new().width(empty_width).height(height).into();

    let bar: Element<'static, M> = container(row![fill, empty].height(height))
        .width(Length::Fill)
        .height(height)
        .style(move |theme: &Theme| container::Style {
            background: Some(theme.clinical().border_default.into()),
            border: Border {
                radius: (height / 2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into();

    row![
        bar,
        Space::new().width(SPACING_SM),
        text(format!("{}%", percentage))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
    ]
    .into()
}

/// Create a validation badge helper.
fn validation_badge<M: 'static>(count: usize, is_warning: bool) -> Element<'static, M> {
    let icon = if is_warning {
        container(lucide::triangle_alert().size(12)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_on_accent),
            ..Default::default()
        })
    } else {
        container(lucide::circle_x().size(12)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_on_accent),
            ..Default::default()
        })
    };

    container(
        row![
            icon,
            Space::new().width(2.0),
            text(count.to_string())
                .size(11)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_on_accent),
                }),
        ]
        .align_y(Alignment::Center),
    )
    .padding([2.0, 6.0])
    .style(move |theme: &Theme| {
        let bg_color = if is_warning {
            theme.extended_palette().warning.base.color
        } else {
            theme.extended_palette().danger.base.color
        };
        container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}
