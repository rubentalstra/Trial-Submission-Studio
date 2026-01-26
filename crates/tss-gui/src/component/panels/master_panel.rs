//! Master panel components.
//!
//! Components for building master panels in master-detail layouts.
//! Includes headers, list wrappers, and empty states.

use iced::widget::{Space, button, column, container, row, rule, text};
use iced::{Alignment, Element, Length, Theme};

use crate::theme::{ClinicalColors, SPACING_LG, SPACING_SM, SPACING_XS, button_secondary};

// =============================================================================
// MASTER PANEL HEADER
// =============================================================================

/// A complete master panel header with title, search, filters, and stats.
///
/// This combines all the common elements of a master panel header into
/// a single reusable component.
///
/// # Example
/// ```ignore
/// MasterPanelHeader::new("Variables")
///     .search(&search_text, "Search...", |s| Message::Search(s))
///     .filter("Unmapped", unmapped, Message::ToggleUnmapped)
///     .stats("15/20 mapped")
///     .view()
/// ```
pub struct MasterPanelHeader<M> {
    title: String,
    #[allow(clippy::type_complexity)]
    search: Option<(String, String, Box<dyn Fn(String) -> M>)>, // (value, placeholder, handler)
    filters: Vec<(String, bool, M)>,
    stats: Option<String>,
}

impl<M: Clone + 'static> MasterPanelHeader<M> {
    /// Create a new master panel header with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            search: None,
            filters: Vec::new(),
            stats: None,
        }
    }

    /// Add search functionality.
    pub fn search(
        mut self,
        value: &str,
        placeholder: impl Into<String>,
        on_change: impl Fn(String) -> M + 'static,
    ) -> Self {
        self.search = Some((value.to_string(), placeholder.into(), Box::new(on_change)));
        self
    }

    /// Add a filter toggle.
    pub fn filter(mut self, label: impl Into<String>, active: bool, on_toggle: M) -> Self {
        self.filters.push((label.into(), active, on_toggle));
        self
    }

    /// Add stats text.
    pub fn stats(mut self, text: impl Into<String>) -> Self {
        self.stats = Some(text.into());
        self
    }

    /// Build the master panel header element.
    pub fn view(self) -> Element<'static, M> {
        use super::FilterToggle;
        use iced::widget::text_input;

        let mut content = column![];

        // Title
        content = content.push(
            text(self.title)
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_secondary),
                })
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                }),
        );
        content = content.push(Space::new().height(SPACING_SM));

        // Search input
        if let Some((value, placeholder, on_change)) = self.search {
            let search_input = text_input(&placeholder, &value)
                .on_input(on_change)
                .padding([8.0, 12.0])
                .size(13)
                .width(Length::Fill);
            content = content.push(search_input);
            content = content.push(Space::new().height(SPACING_XS));
        }

        // Filter toggles
        if !self.filters.is_empty() {
            let filter_buttons: Vec<Element<'static, M>> = self
                .filters
                .into_iter()
                .map(|(label, active, on_toggle)| {
                    FilterToggle::new(label, active, on_toggle).view()
                })
                .collect();
            content = content.push(row(filter_buttons).spacing(SPACING_XS));
            content = content.push(Space::new().height(SPACING_SM));
        }

        // Stats
        if let Some(stats_text) = self.stats {
            // Split stats text into number and label parts
            let (num_part, label_part) = if let Some((n, r)) = stats_text.split_once(' ') {
                (n.to_string(), r.to_string())
            } else {
                (stats_text, String::new())
            };

            let stats_row = row![
                text(num_part).size(12).style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
                Space::new().width(4.0),
                text(label_part)
                    .size(11)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted),
                    }),
            ]
            .align_y(Alignment::Center);
            content = content.push(stats_row);
            content = content.push(Space::new().height(SPACING_SM));
        }

        // Divider
        content = content.push(rule::horizontal(1));
        content = content.push(Space::new().height(SPACING_SM));

        content.into()
    }
}

// =============================================================================
// MASTER PANEL EMPTY STATE
// =============================================================================

/// Empty state for master panel when no items match filters.
///
/// # Example
/// ```ignore
/// master_panel_empty("No variables match your filters", Message::ClearFilters)
/// ```
pub fn master_panel_empty<'a, M: Clone + 'a>(
    message: impl Into<String>,
    clear_message: M,
) -> Element<'a, M> {
    let msg = message.into();

    container(
        column![
            text(msg).size(13).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
            Space::new().height(SPACING_SM),
            button(text("Clear filters").size(12))
                .on_press(clear_message)
                .padding([6.0, 12.0])
                .style(button_secondary),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(SPACING_LG)
    .center_x(Length::Shrink)
    .into()
}

// =============================================================================
// MASTER PANEL SECTION
// =============================================================================

/// A titled section within a master panel.
///
/// # Example
/// ```ignore
/// MasterPanelSection::new("Auto-Generated")
///     .count(5)
///     .content(items_column)
///     .view()
/// ```
pub struct MasterPanelSection<'a, M> {
    title: String,
    count: Option<usize>,
    content: Element<'a, M>,
}

impl<'a, M: 'a> MasterPanelSection<'a, M> {
    /// Create a new master panel section.
    pub fn new(title: impl Into<String>, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            title: title.into(),
            count: None,
            content: content.into(),
        }
    }

    /// Add a count badge to the section title.
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Build the section element.
    pub fn view(self) -> Element<'a, M> {
        let count_el: Element<'a, M> = if let Some(count) = self.count {
            row![
                Space::new().width(SPACING_XS),
                text(format!("({})", count))
                    .size(11)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted),
                    }),
            ]
            .into()
        } else {
            Space::new().width(0.0).into()
        };

        let title_row = row![
            text(self.title)
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
            count_el,
        ]
        .align_y(Alignment::Center);

        column![title_row, Space::new().height(SPACING_XS), self.content,].into()
    }
}
