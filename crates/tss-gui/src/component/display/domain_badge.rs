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
use iced::{Border, Element, Theme};

use crate::theme::ClinicalColors;

/// Domain code badge (standard size).
///
/// Creates a pill-shaped badge with primary background for domain codes.
pub fn domain_badge<'a, M: 'a>(code: &'a str) -> Element<'a, M> {
    let code_owned = code.to_string();

    container(text(code_owned.clone()).size(14).style(|theme: &Theme| {
        let clinical = theme.clinical();
        text::Style {
            color: Some(clinical.text_on_accent),
        }
    }))
    .padding([4.0, 12.0])
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.primary.base.color.into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}

/// Domain code badge (smaller size).
///
/// Smaller variant for compact contexts.
pub fn domain_badge_small<'a, M: 'a>(code: &'a str) -> Element<'a, M> {
    let code_owned = code.to_string();

    container(text(code_owned.clone()).size(12).style(|theme: &Theme| {
        let clinical = theme.clinical();
        text::Style {
            color: Some(clinical.text_on_accent),
        }
    }))
    .padding([2.0, 8.0])
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.primary.base.color.into()),
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}
