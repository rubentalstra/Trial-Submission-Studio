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
use iced::{Alignment, Border, Color, Element, Length};
use iced_fonts::lucide;

use crate::theme::{BORDER_RADIUS_MD, SPACING_MD, SPACING_SM, SPACING_XS, colors};

/// Enhanced domain card with progress and validation badges.
pub struct DomainCard<M> {
    code: String,
    name: String,
    row_count: usize,
    progress: f32,
    validation: Option<(usize, usize)>, // (warnings, errors)
    on_click: M,
    // Theme colors
    accent_color: Color,
    text_on_accent: Color,
    text_primary: Color,
    text_muted: Color,
    hover_bg: Color,
    card_bg: Color,
    border_color: Color,
    warning_color: Color,
    error_color: Color,
    // Progress bar colors
    progress_fill: Color,
    progress_complete: Color,
    progress_track: Color,
}

impl<M: Clone + 'static> DomainCard<M> {
    /// Create a new domain card.
    pub fn new(code: impl Into<String>, name: impl Into<String>, on_click: M) -> Self {
        let c = colors();
        Self {
            code: code.into(),
            name: name.into(),
            row_count: 0,
            progress: 0.0,
            validation: None,
            on_click,
            accent_color: c.accent_primary,
            text_on_accent: c.text_on_accent,
            text_primary: c.text_primary,
            text_muted: c.text_muted,
            hover_bg: c.background_secondary,
            card_bg: c.background_elevated,
            border_color: c.border_default,
            warning_color: c.status_warning,
            error_color: c.status_error,
            progress_fill: c.accent_primary,
            progress_complete: c.status_success,
            progress_track: c.border_default,
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
        let accent = self.accent_color;
        let text_on_accent = self.text_on_accent;
        let text_primary = self.text_primary;
        let text_muted = self.text_muted;
        let hover_bg = self.hover_bg;
        let card_bg = self.card_bg;
        let border_color = self.border_color;
        let warning_color = self.warning_color;
        let error_color = self.error_color;
        let progress_fill = self.progress_fill;
        let progress_complete = self.progress_complete;
        let progress_track = self.progress_track;

        // Domain badge
        let badge: Element<'static, M> = container(text(code).size(14).color(text_on_accent))
            .padding([4.0, 12.0])
            .style(move |_| container::Style {
                background: Some(accent.into()),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into();

        let name_text = text(name).size(14).color(text_primary);
        let row_text = text(format!("{} rows", row_count))
            .size(12)
            .color(text_muted);

        let top_row = row![
            badge,
            Space::new().width(SPACING_SM),
            name_text,
            Space::new().width(Length::Fill),
            row_text,
        ]
        .align_y(Alignment::Center);

        // Progress bar (inline)
        let progress_bar: Element<'static, M> = create_progress_bar(
            progress,
            progress_fill,
            progress_complete,
            progress_track,
            text_muted,
            6.0,
        );

        // Validation badges with themed colors
        let validation_badges: Element<'static, M> = match validation {
            Some((warnings, errors)) => {
                let mut badges = row![].spacing(SPACING_XS).align_y(Alignment::Center);

                if warnings > 0 {
                    badges = badges.push(validation_badge(
                        warnings,
                        warning_color,
                        text_on_accent,
                        true,
                    ));
                }

                if errors > 0 {
                    badges =
                        badges.push(validation_badge(errors, error_color, text_on_accent, false));
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
            .style(move |_theme, status| {
                let bg = match status {
                    iced::widget::button::Status::Hovered => hover_bg,
                    _ => card_bg,
                };
                iced::widget::button::Style {
                    background: Some(bg.into()),
                    text_color: text_primary,
                    border: Border {
                        radius: BORDER_RADIUS_MD.into(),
                        color: border_color,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}

/// Create an inline progress bar element.
fn create_progress_bar<M: 'static>(
    value: f32,
    fill_color: Color,
    complete_color: Color,
    track_color: Color,
    label_color: Color,
    height: f32,
) -> Element<'static, M> {
    let percentage = (value * 100.0).round() as u32;
    let actual_fill = if value >= 1.0 {
        complete_color
    } else {
        fill_color
    };

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

    let fill: Element<'static, M> = container(Space::new())
        .width(fill_width)
        .height(height)
        .style(move |_| container::Style {
            background: Some(actual_fill.into()),
            border: Border {
                radius: (height / 2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into();

    let empty: Element<'static, M> = Space::new().width(empty_width).height(height).into();

    let bar: Element<'static, M> = container(row![fill, empty].height(height))
        .width(Length::Fill)
        .height(height)
        .style(move |_| container::Style {
            background: Some(track_color.into()),
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
        text(format!("{}%", percentage)).size(12).color(label_color),
    ]
    .into()
}

/// Create a validation badge helper.
fn validation_badge<M: 'static>(
    count: usize,
    bg_color: Color,
    text_color: Color,
    is_warning: bool,
) -> Element<'static, M> {
    let icon = if is_warning {
        lucide::triangle_alert().size(12).color(text_color)
    } else {
        lucide::circle_x().size(12).color(text_color)
    };

    container(
        row![
            icon,
            Space::new().width(2.0),
            text(count.to_string()).size(11).color(text_color),
        ]
        .align_y(Alignment::Center),
    )
    .padding([2.0, 6.0])
    .style(move |_| container::Style {
        background: Some(bg_color.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
