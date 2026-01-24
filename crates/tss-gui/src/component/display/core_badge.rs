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
use iced::{Border, Element, Theme};
use tss_standards::CoreDesignation;

use crate::theme::{BORDER_RADIUS_SM, ClinicalColors};

/// Core designation badge.
///
/// Returns a colored badge based on core designation:
/// - Required: Red ("Req")
/// - Expected: Amber ("Exp")
/// - Permissible: Gray ("Perm")
pub fn core_badge<'a, M: 'a>(designation: CoreDesignation) -> Element<'a, M> {
    let label = match designation {
        CoreDesignation::Required => "Req",
        CoreDesignation::Expected => "Exp",
        CoreDesignation::Permissible => "Perm",
    };

    container(text(label).size(10).style(move |theme: &Theme| {
        let palette = theme.extended_palette();
        let clinical = theme.clinical();
        let text_color = match designation {
            CoreDesignation::Required => palette.danger.base.color,
            CoreDesignation::Expected => palette.warning.base.color,
            CoreDesignation::Permissible => clinical.text_muted,
        };
        text::Style {
            color: Some(text_color),
        }
    }))
    .padding([2.0, 6.0])
    .style(move |theme: &Theme| {
        let clinical = theme.clinical();
        let bg_color = match designation {
            CoreDesignation::Required => clinical.status_error_light,
            CoreDesignation::Expected => clinical.status_warning_light,
            CoreDesignation::Permissible => clinical.background_secondary,
        };
        container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        }
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
        Some(d @ CoreDesignation::Required) | Some(d @ CoreDesignation::Expected) => {
            Some(core_badge(d))
        }
        _ => None,
    }
}
