//! Master-detail layout component.
//!
//! A split pane layout with a master list on the left and detail view on the right.
//! Commonly used for the mapping view (variable list + detail panel) and export view.

use iced::widget::{column, container, row, rule, scrollable};
use iced::{Element, Length};

use crate::theme::{GRAY_200, SPACING_MD};

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
        // Master panel (fixed width, scrollable)
        container(scrollable(master).height(Length::Fill))
            .width(Length::Fixed(master_width))
            .height(Length::Fill)
            .padding(SPACING_MD),
        // Vertical divider
        rule::vertical(1).style(|_theme| rule::Style {
            color: GRAY_200,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }),
        // Detail panel (fill remaining, scrollable)
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
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::master_detail_with_header;
///
/// let layout = master_detail_with_header(
///     tab_bar,
///     variable_list,
///     variable_details,
///     280.0,
/// );
/// ```
pub fn master_detail_with_header<'a, M: 'a>(
    header: Element<'a, M>,
    master: Element<'a, M>,
    detail: Element<'a, M>,
    master_width: f32,
) -> Element<'a, M> {
    column![
        // Header bar
        container(header)
            .width(Length::Fill)
            .padding(iced::Padding::new(SPACING_MD).bottom(0.0)),
        // Horizontal divider
        rule::horizontal(1).style(|_theme| rule::Style {
            color: GRAY_200,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }),
        // Master-detail content
        master_detail(master, detail, master_width),
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
        // Detail panel (fill remaining, scrollable)
        container(scrollable(detail).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(SPACING_MD),
        // Vertical divider
        rule::vertical(1).style(|_theme| rule::Style {
            color: GRAY_200,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }),
        // Master panel (fixed width, scrollable)
        container(scrollable(master).height(Length::Fill))
            .width(Length::Fixed(master_width))
            .height(Length::Fill)
            .padding(SPACING_MD),
    ]
    .height(Length::Fill)
    .into()
}
