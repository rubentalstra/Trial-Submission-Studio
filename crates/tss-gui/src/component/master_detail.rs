//! Master-detail layout component.
//!
//! A split pane layout with a master list on the left and detail view on the right.
//! Commonly used for the mapping view (variable list + detail panel) and export view.

use iced::widget::{column, container, row, rule, scrollable};
use iced::{Element, Length, Theme};

use crate::theme::{ClinicalColors, SPACING_MD};

// =============================================================================
// HELPER
// =============================================================================

/// Helper to create a divider rule style using theme.
fn divider_style(theme: &Theme) -> rule::Style {
    rule::Style {
        color: theme.clinical().border_default,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

// =============================================================================
// MASTER-DETAIL LAYOUT
// =============================================================================

/// Creates a master-detail split layout.
///
/// The master panel (left) has a fixed width, while the detail panel (right)
/// fills the remaining space. Both panels are scrollable.
///
/// # Arguments
///
/// * `master` - Content for the left panel (typically a list)
/// * `detail` - Content for the right panel (typically details of selected item)
/// * `master_width` - Width of the master panel in pixels
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::master_detail;
///
/// let layout = master_detail(
///     variable_list,
///     variable_details,
///     280.0,
/// );
/// ```
pub fn master_detail<'a, M: 'a>(
    master: Element<'a, M>,
    detail: Element<'a, M>,
    master_width: f32,
) -> Element<'a, M> {
    row![
        container(scrollable(master).height(Length::Fill))
            .width(Length::Fixed(master_width))
            .height(Length::Fill)
            .padding(SPACING_MD),
        rule::vertical(1).style(divider_style),
        container(scrollable(detail).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(SPACING_MD),
    ]
    .height(Length::Fill)
    .into()
}

/// Creates a master-detail layout with a header bar.
///
/// Adds a horizontal header above the master-detail split.
///
/// # Arguments
///
/// * `header` - Content for the header bar
/// * `master` - Content for the left panel
/// * `detail` - Content for the right panel
/// * `master_width` - Width of the master panel in pixels
pub fn master_detail_with_header<'a, M: 'a>(
    header: Element<'a, M>,
    master: Element<'a, M>,
    detail: Element<'a, M>,
    master_width: f32,
) -> Element<'a, M> {
    column![
        container(header)
            .width(Length::Fill)
            .padding(iced::Padding::new(SPACING_MD).bottom(0.0)),
        rule::horizontal(1).style(divider_style),
        master_detail(master, detail, master_width),
    ]
    .height(Length::Fill)
    .into()
}

/// Creates a master-detail layout with a pinned header on the master panel.
///
/// The master panel has a fixed header that doesn't scroll, while the content below scrolls.
/// This is useful for keeping search bars, filters, or titles visible.
///
/// # Arguments
///
/// * `master_header` - Pinned header content for the master panel (search, filters, etc.)
/// * `master_content` - Scrollable content for the master panel (list items)
/// * `detail` - Content for the right panel
/// * `master_width` - Width of the master panel in pixels
pub fn master_detail_with_pinned_header<'a, M: 'a>(
    master_header: Element<'a, M>,
    master_content: Element<'a, M>,
    detail: Element<'a, M>,
    master_width: f32,
) -> Element<'a, M> {
    row![
        container(
            column![
                master_header,
                scrollable(master_content).height(Length::Fill),
            ]
            .height(Length::Fill)
        )
        .width(Length::Fixed(master_width))
        .height(Length::Fill)
        .padding(SPACING_MD),
        rule::vertical(1).style(divider_style),
        container(detail)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(SPACING_MD),
    ]
    .height(Length::Fill)
    .into()
}

/// Creates a master-detail layout with master on the right.
///
/// Reverses the layout with detail on left and master on right.
/// Useful for right-to-left workflows or specific UI patterns.
///
/// # Arguments
///
/// * `detail` - Content for the left panel
/// * `master` - Content for the right panel
/// * `master_width` - Width of the master panel in pixels
pub fn detail_master<'a, M: 'a>(
    detail: Element<'a, M>,
    master: Element<'a, M>,
    master_width: f32,
) -> Element<'a, M> {
    row![
        container(scrollable(detail).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(SPACING_MD),
        rule::vertical(1).style(divider_style),
        container(scrollable(master).height(Length::Fill))
            .width(Length::Fixed(master_width))
            .height(Length::Fill)
            .padding(SPACING_MD),
    ]
    .height(Length::Fill)
    .into()
}
