//! Core designation badge component.
//!
//! Badges for CDISC core designations (Required, Expected, Permissible).
//! Uses the semantic color system for accessibility mode support.
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::core_badge;
//! use tss_standards::CoreDesignation;
//!
//! core_badge(CoreDesignation::Required)
//! core_badge(CoreDesignation::Expected)
//! core_badge(CoreDesignation::Permissible)
//! ```

use iced::widget::{container, text};
use iced::{Border, Element};
use tss_standards::CoreDesignation;

use crate::theme::{BORDER_RADIUS_SM, SemanticColor, ThemeConfig};

/// Core designation badge.
///
/// Returns a colored badge based on core designation:
/// - Required: Red ("Req")
/// - Expected: Amber ("Exp")
/// - Permissible: Gray ("Perm")
pub fn core_badge<'a, M: 'a>(designation: CoreDesignation) -> Element<'a, M> {
    core_badge_themed(&ThemeConfig::default(), designation)
}

/// Core designation badge with specific theme config.
pub fn core_badge_themed<'a, M: 'a>(
    config: &ThemeConfig,
    designation: CoreDesignation,
) -> Element<'a, M> {
    let (label, text_color, bg_color) = match designation {
        CoreDesignation::Required => (
            "Req",
            config.resolve(SemanticColor::StatusError),
            config.resolve(SemanticColor::StatusErrorLight),
        ),
        CoreDesignation::Expected => (
            "Exp",
            config.resolve(SemanticColor::StatusWarning),
            config.resolve(SemanticColor::StatusWarningLight),
        ),
        CoreDesignation::Permissible => (
            "Perm",
            config.resolve(SemanticColor::TextMuted),
            config.resolve(SemanticColor::BackgroundSecondary),
        ),
    };

    container(text(label).size(10).color(text_color))
        .padding([2.0, 6.0])
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// Core designation badge from optional.
///
/// Returns None element if designation is None or Permissible.
/// Only shows badges for Required and Expected.
pub fn core_badge_if_important<'a, M: 'a>(
    designation: Option<CoreDesignation>,
) -> Option<Element<'a, M>> {
    core_badge_if_important_themed(&ThemeConfig::default(), designation)
}

/// Core designation badge from optional with specific theme config.
pub fn core_badge_if_important_themed<'a, M: 'a>(
    config: &ThemeConfig,
    designation: Option<CoreDesignation>,
) -> Option<Element<'a, M>> {
    match designation {
        Some(CoreDesignation::Required) | Some(CoreDesignation::Expected) => {
            Some(core_badge_themed(config, designation.unwrap()))
        }
        _ => None,
    }
}
