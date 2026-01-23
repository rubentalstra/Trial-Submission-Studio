//! Split view layout component.
//!
//! A flexible split-pane layout that consolidates all master-detail variants into
//! a single builder-pattern component.
//!
//! # Features
//!
//! - Configurable panel widths
//! - Optional pinned headers for either panel
//! - Reversible layout (master left or right)
//! - Automatic scrolling options
//! - Consistent divider styling
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::layout::SplitView;
//!
//! // Simple split view
//! SplitView::new(variable_list, variable_details)
//!     .master_width(280.0)
//!     .view();
//!
//! // With pinned header on master panel
//! SplitView::new(variable_list, variable_details)
//!     .master_width(300.0)
//!     .master_header(search_filter_bar)
//!     .view();
//!
//! // Reversed layout (detail on left, master on right)
//! SplitView::new(item_list, item_details)
//!     .master_width(250.0)
//!     .reversed()
//!     .view();
//! ```

use iced::widget::{column, container, row, rule, scrollable};
use iced::{Element, Length, Theme};

use crate::theme::{ClinicalColors, SPACING_MD};

/// Split view panel configuration.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PanelScroll {
    /// Panel content is scrollable
    #[default]
    Scroll,
    /// Panel content is not scrollable (caller handles scrolling)
    None,
}

/// A flexible split-pane layout component.
///
/// Provides a builder pattern for creating master-detail style layouts with
/// various configurations for headers, widths, and scrolling behavior.
pub struct SplitView<'a, M> {
    /// Content for the master panel (typically a list)
    master: Element<'a, M>,
    /// Content for the detail panel (typically item details)
    detail: Element<'a, M>,
    /// Width of the master panel in pixels
    master_width: f32,
    /// Optional pinned header for the master panel
    master_header: Option<Element<'a, M>>,
    /// Optional pinned header for the detail panel
    detail_header: Option<Element<'a, M>>,
    /// Whether to reverse the layout (detail left, master right)
    reversed: bool,
    /// Scrolling behavior for master panel
    master_scroll: PanelScroll,
    /// Scrolling behavior for detail panel
    detail_scroll: PanelScroll,
    /// Optional page-level header above the split
    page_header: Option<Element<'a, M>>,
}

impl<'a, M: 'a> SplitView<'a, M> {
    /// Create a new split view with master and detail content.
    ///
    /// # Arguments
    ///
    /// * `master` - Content for the master panel (typically a list)
    /// * `detail` - Content for the detail panel (typically item details)
    pub fn new(master: impl Into<Element<'a, M>>, detail: impl Into<Element<'a, M>>) -> Self {
        Self {
            master: master.into(),
            detail: detail.into(),
            master_width: 280.0,
            master_header: None,
            detail_header: None,
            reversed: false,
            master_scroll: PanelScroll::Scroll,
            detail_scroll: PanelScroll::None,
            page_header: None,
        }
    }

    /// Set the width of the master panel in pixels.
    ///
    /// Default is 280.0 pixels.
    pub fn master_width(mut self, width: f32) -> Self {
        self.master_width = width;
        self
    }

    /// Add a pinned header to the master panel.
    ///
    /// The header will stay fixed at the top while the content scrolls below.
    /// This is useful for search bars, filters, or section titles.
    pub fn master_header(mut self, header: impl Into<Element<'a, M>>) -> Self {
        self.master_header = Some(header.into());
        self
    }

    /// Add a pinned header to the detail panel.
    ///
    /// The header will stay fixed at the top while the content scrolls below.
    pub fn detail_header(mut self, header: impl Into<Element<'a, M>>) -> Self {
        self.detail_header = Some(header.into());
        self
    }

    /// Reverse the layout (detail on left, master on right).
    ///
    /// By default, master is on the left and detail is on the right.
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }

    /// Set whether the master panel should scroll its content.
    ///
    /// Default is `PanelScroll::Scroll`.
    pub fn master_scroll(mut self, scroll: PanelScroll) -> Self {
        self.master_scroll = scroll;
        self
    }

    /// Set whether the detail panel should scroll its content.
    ///
    /// Default is `PanelScroll::None` (caller handles scrolling).
    pub fn detail_scroll(mut self, scroll: PanelScroll) -> Self {
        self.detail_scroll = scroll;
        self
    }

    /// Add a page-level header above the entire split view.
    ///
    /// This is useful for breadcrumbs, page titles, or action bars that
    /// span the full width above the split.
    pub fn page_header(mut self, header: impl Into<Element<'a, M>>) -> Self {
        self.page_header = Some(header.into());
        self
    }

    /// Build the split view element.
    pub fn view(self) -> Element<'a, M> {
        // Destructure self to avoid borrow issues
        let Self {
            master,
            detail,
            master_width,
            master_header,
            detail_header,
            reversed,
            master_scroll,
            detail_scroll,
            page_header,
        } = self;

        let divider_style = |theme: &Theme| rule::Style {
            color: theme.clinical().border_default,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        };

        // Build master panel content
        let master_content = build_panel_content(master, master_header, master_scroll);

        // Build detail panel content
        let detail_content = build_panel_content(detail, detail_header, detail_scroll);

        // Create the panels
        let master_panel = container(master_content)
            .width(Length::Fixed(master_width))
            .height(Length::Fill)
            .padding(SPACING_MD);

        let detail_panel = container(detail_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(SPACING_MD);

        // Build the split row (respecting reversed flag)
        let split_row: Element<'a, M> = if reversed {
            row![
                detail_panel,
                rule::vertical(1).style(divider_style),
                master_panel,
            ]
            .height(Length::Fill)
            .into()
        } else {
            row![
                master_panel,
                rule::vertical(1).style(divider_style),
                detail_panel,
            ]
            .height(Length::Fill)
            .into()
        };

        // Add page header if present
        if let Some(header) = page_header {
            column![
                container(header)
                    .width(Length::Fill)
                    .padding(iced::Padding::new(SPACING_MD).bottom(0.0)),
                rule::horizontal(1).style(divider_style),
                split_row,
            ]
            .height(Length::Fill)
            .into()
        } else {
            split_row
        }
    }
}

/// Build panel content with optional header and scrolling.
fn build_panel_content<'a, M: 'a>(
    content: Element<'a, M>,
    header: Option<Element<'a, M>>,
    scroll: PanelScroll,
) -> Element<'a, M> {
    let scrolled_content: Element<'a, M> = match scroll {
        PanelScroll::Scroll => scrollable(content).height(Length::Fill).into(),
        PanelScroll::None => content,
    };

    if let Some(header) = header {
        column![header, scrolled_content,]
            .height(Length::Fill)
            .into()
    } else {
        scrolled_content
    }
}

// =============================================================================
// CONVENIENCE FUNCTIONS (for backwards compatibility)
// =============================================================================

/// Creates a simple master-detail split layout.
///
/// This is a convenience function that wraps `SplitView` for simple use cases.
///
/// # Arguments
///
/// * `master` - Content for the left panel (typically a list)
/// * `detail` - Content for the right panel (typically details of selected item)
/// * `master_width` - Width of the master panel in pixels
pub fn split_view<'a, M: 'a>(
    master: impl Into<Element<'a, M>>,
    detail: impl Into<Element<'a, M>>,
    master_width: f32,
) -> Element<'a, M> {
    SplitView::new(master, detail)
        .master_width(master_width)
        .view()
}

/// Creates a split view with a pinned header on the master panel.
///
/// # Arguments
///
/// * `master_header` - Pinned header content for the master panel
/// * `master_content` - Scrollable content for the master panel
/// * `detail` - Content for the detail panel
/// * `master_width` - Width of the master panel in pixels
pub fn split_view_with_header<'a, M: 'a>(
    master_header: impl Into<Element<'a, M>>,
    master_content: impl Into<Element<'a, M>>,
    detail: impl Into<Element<'a, M>>,
    master_width: f32,
) -> Element<'a, M> {
    SplitView::new(master_content, detail)
        .master_width(master_width)
        .master_header(master_header)
        .view()
}
