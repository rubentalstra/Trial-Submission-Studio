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
//! domain_badge_themed(&theme_config, "DM")
//! ```

use iced::widget::{container, text};
use iced::{Border, Element};

use crate::theme::{PRIMARY_500, SemanticColor, ThemeConfig, WHITE};

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

/// Domain code badge with theme support (standard size).
///
/// Creates a pill-shaped badge with primary accent background.
pub fn domain_badge_themed<'a, M: 'a>(config: &ThemeConfig, code: &'a str) -> Element<'a, M> {
    let bg_color = config.resolve(SemanticColor::AccentPrimary);
    let text_color = config.resolve(SemanticColor::TextOnAccent);

    container(text(code).size(14).color(text_color))
        .padding([4.0, 12.0])
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

/// Domain code badge with theme support (smaller size).
///
/// Smaller themed variant for compact contexts.
pub fn domain_badge_small_themed<'a, M: 'a>(config: &ThemeConfig, code: &'a str) -> Element<'a, M> {
    let bg_color = config.resolve(SemanticColor::AccentPrimary);
    let text_color = config.resolve(SemanticColor::TextOnAccent);

    container(text(code).size(12).color(text_color))
        .padding([2.0, 8.0])
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
