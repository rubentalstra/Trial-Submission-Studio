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
use iced::{Alignment, Border, Element, Length};
use iced_fonts::lucide;

use super::progress_bar::ProgressBar;
use crate::theme::{
    BORDER_RADIUS_MD, ERROR, GRAY_100, GRAY_200, GRAY_500, GRAY_700, GRAY_800, PRIMARY_500,
    SPACING_MD, SPACING_SM, SPACING_XS, WARNING, WHITE,
};

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
        // Clone owned data for the static lifetime
        let code = self.code;
        let name = self.name;
        let row_count = self.row_count;
        let progress = self.progress;
        let validation = self.validation;
        let on_click = self.on_click;

        // Top row: badge + name + row count
        let badge: Element<'static, M> = domain_badge_owned(code);
        let name_text = text(name).size(14).color(GRAY_800);
        let row_text = text(format!("{} rows", row_count)).size(12).color(GRAY_500);

        let top_row = row![
            badge,
            Space::new().width(SPACING_SM),
            name_text,
            Space::new().width(Length::Fill),
            row_text,
        ]
        .align_y(Alignment::Center);

        // Bottom row: progress bar + validation badges
        let progress_bar: Element<'static, M> = ProgressBar::new(progress)
            .height(6.0)
            .show_label(true)
            .view();

        // Validation badges (only show if count > 0)
        let validation_badges: Element<'static, M> = match validation {
            Some((warnings, errors)) => {
                let mut badges = row![].spacing(SPACING_XS).align_y(Alignment::Center);

                if warnings > 0 {
                    badges = badges.push(warning_badge(warnings));
                }

                if errors > 0 {
                    badges = badges.push(error_badge(errors));
                }

                badges.into()
            }
            None => {
                // Still loading - show nothing or subtle indicator
                Space::new().width(0.0).into()
            }
        };

        let bottom_row = row![
            container(progress_bar).width(Length::Fill),
            Space::new().width(SPACING_SM),
            validation_badges,
        ]
        .align_y(Alignment::Center);

        // Card content
        let content = iced::widget::column![top_row, Space::new().height(SPACING_SM), bottom_row,]
            .padding(SPACING_MD);

        // Wrap in clickable button with card styling
        button(content)
            .on_press(on_click)
            .padding(0.0)
            .width(Length::Fill)
            .style(|_theme, status| {
                let bg = match status {
                    iced::widget::button::Status::Hovered => GRAY_100,
                    _ => WHITE,
                };
                iced::widget::button::Style {
                    background: Some(bg.into()),
                    text_color: GRAY_700,
                    border: Border {
                        radius: BORDER_RADIUS_MD.into(),
                        color: GRAY_200,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}

/// Domain badge that takes ownership of the code string.
fn domain_badge_owned<M: 'static>(code: String) -> Element<'static, M> {
    container(text(code).size(14).color(WHITE))
        .padding([4.0, 12.0])
        .style(|_| container::Style {
            background: Some(PRIMARY_500.into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// Create a warning validation badge.
fn warning_badge<M: 'static>(count: usize) -> Element<'static, M> {
    container(
        row![
            lucide::triangle_alert().size(12).color(WHITE),
            Space::new().width(2.0),
            text(count.to_string()).size(11).color(WHITE),
        ]
        .align_y(Alignment::Center),
    )
    .padding([2.0, 6.0])
    .style(|_| container::Style {
        background: Some(WARNING.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

/// Create an error validation badge.
fn error_badge<M: 'static>(count: usize) -> Element<'static, M> {
    container(
        row![
            lucide::circle_x().size(12).color(WHITE),
            Space::new().width(2.0),
            text(count.to_string()).size(11).color(WHITE),
        ]
        .align_y(Alignment::Center),
    )
    .padding([2.0, 6.0])
    .style(|_| container::Style {
        background: Some(ERROR.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
