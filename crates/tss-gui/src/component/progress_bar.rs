//! Progress bar component.
//!
//! Simple horizontal progress bar with optional percentage label.
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::ProgressBar;
//!
//! ProgressBar::new(0.75)
//!     .height(6.0)
//!     .show_label(true)
//!     .view()
//! ```

use iced::widget::{Space, container, row, text};
use iced::{Border, Element, Length};

use crate::theme::{GRAY_200, GRAY_600, PRIMARY_500, SPACING_SM, SUCCESS};

/// A horizontal progress bar.
///
/// Shows progress as a filled bar with optional percentage text.
pub struct ProgressBar {
    value: f32,
    height: f32,
    show_label: bool,
}

impl ProgressBar {
    /// Create a new progress bar with the given value (0.0 to 1.0).
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            height: 6.0,
            show_label: false,
        }
    }

    /// Set the height of the progress bar track.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Show percentage label next to the bar.
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Build the progress bar element.
    pub fn view<M: 'static>(self) -> Element<'static, M> {
        let percentage = (self.value * 100.0).round() as u32;
        let height = self.height;

        // Choose color based on completion
        let fill_color = if self.value >= 1.0 {
            SUCCESS
        } else {
            PRIMARY_500
        };

        // Filled portion width as FillPortion for proper scaling
        let fill_width = if self.value > 0.0 {
            Length::FillPortion((self.value * 100.0).max(1.0) as u16)
        } else {
            Length::Fixed(0.0)
        };

        let empty_width = if self.value < 1.0 {
            Length::FillPortion(((1.0 - self.value) * 100.0).max(1.0) as u16)
        } else {
            Length::Fixed(0.0)
        };

        // Filled bar segment
        let fill: Element<'static, M> = container(Space::new())
            .width(fill_width)
            .height(height)
            .style(move |_theme| container::Style {
                background: Some(fill_color.into()),
                border: Border {
                    radius: (height / 2.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into();

        // Empty segment
        let empty: Element<'static, M> = Space::new().width(empty_width).height(height).into();

        // Combine fill and empty into track with background
        let bar: Element<'static, M> = container(row![fill, empty].height(height))
            .width(Length::Fill)
            .height(height)
            .style(move |_theme| container::Style {
                background: Some(GRAY_200.into()),
                border: Border {
                    radius: (height / 2.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into();

        if self.show_label {
            row![
                bar,
                Space::new().width(SPACING_SM),
                text(format!("{}%", percentage)).size(12).color(GRAY_600),
            ]
            .into()
        } else {
            bar
        }
    }
}
