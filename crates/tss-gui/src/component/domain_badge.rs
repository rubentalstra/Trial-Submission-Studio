//! Domain badge component.
//!
//! Colored pill badges for domain codes (DM, AE, LB, etc.).
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::domain_badge;
//!
//! domain_badge("DM")
//! domain_badge_small("AE")
//! ```

use iced::widget::{container, text};
use iced::{Border, Element};

use crate::theme::{PRIMARY_500, WHITE};

/// Domain code badge (standard size).
///
/// Creates a pill-shaped badge with primary background for domain codes.
pub fn domain_badge<'a, M: 'a>(code: &'a str) -> Element<'a, M> {
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

/// Domain code badge (smaller size).
///
/// Smaller variant for compact contexts.
pub fn domain_badge_small<'a, M: 'a>(code: &'a str) -> Element<'a, M> {
    container(text(code).size(12).color(WHITE))
        .padding([2.0, 8.0])
        .style(|_| container::Style {
            background: Some(PRIMARY_500.into()),
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
