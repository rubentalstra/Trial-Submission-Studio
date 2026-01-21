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

use crate::theme::{BORDER_RADIUS_SM, colors};

/// Core designation badge.
///
/// Returns a colored badge based on core designation:
/// - Required: Red ("Req")
/// - Expected: Amber ("Exp")
/// - Permissible: Gray ("Perm")
pub fn core_badge<'a, M: 'a>(designation: CoreDesignation) -> Element<'a, M> {
    let colors = colors();
    let (label, text_color, bg_color) = match designation {
        CoreDesignation::Required => ("Req", colors.status_error, colors.status_error_light),
        CoreDesignation::Expected => ("Exp", colors.status_warning, colors.status_warning_light),
        CoreDesignation::Permissible => ("Perm", colors.text_muted, colors.background_secondary),
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
    match designation {
        Some(CoreDesignation::Required) | Some(CoreDesignation::Expected) => {
            Some(core_badge(designation.unwrap()))
        }
        _ => None,
    }
}
