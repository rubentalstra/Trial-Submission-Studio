//! List panel component.
//!
//! A comprehensive list panel that consolidates `MasterPanelHeader`, `SearchFilterBar`,
//! and list rendering into a single builder-pattern component.
//!
//! # Features
//!
//! - Optional title with styling
//! - Search input with custom placeholder
//! - Multiple filter toggle buttons
//! - Stats display (e.g., "15/20 mapped")
//! - Empty state handling
//! - Sections for grouped content
//!
//! # Example
//!
//! ```ignore
//! use tss_gui::component::panels::ListPanel;
//!
//! // Full-featured list panel
//! ListPanel::new()
//!     .title("Variables")
//!     .search(&search_text, "Search variables...", |s| Msg::Search(s))
//!     .filter("Unmapped", unmapped, Msg::ToggleUnmapped)
//!     .filter("Required", required, Msg::ToggleRequired)
//!     .stats("15/20 mapped")
//!     .items(variable_items)
//!     .view();
//!
//! // Simple list panel with just items
//! ListPanel::new()
//!     .title("Domains")
//!     .items(domain_items)
//!     .view();
//!
//! // With empty state
//! ListPanel::new()
//!     .title("Variables")
//!     .search(&search, "Search...", |s| Msg::Search(s))
//!     .empty_state("No variables match your filters", Msg::ClearFilters)
//!     .view();
//! ```

use iced::widget::{
    Column, Space, button, column, container, row, rule, scrollable, text, text_input,
};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::theme::{
    ALPHA_LIGHT, BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_SM, SPACING_XS,
    button_secondary,
};

// =============================================================================
// FILTER TOGGLE (internal)
// =============================================================================

/// Internal filter toggle button.
fn filter_toggle<M: Clone + 'static>(
    label: String,
    active: bool,
    on_toggle: M,
) -> Element<'static, M> {
    button(text(label).size(11))
        .on_press(on_toggle)
        .padding([4.0, 8.0])
        .style(move |theme: &Theme, status| {
            if active {
                let accent_primary = theme.extended_palette().primary.base.color;
                let accent_light = Color {
                    a: ALPHA_LIGHT,
                    ..accent_primary
                };
                iced::widget::button::Style {
                    background: Some(accent_light.into()),
                    text_color: accent_primary,
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        color: accent_primary,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            } else {
                button_secondary(theme, status)
            }
        })
        .into()
}

// =============================================================================
// LIST PANEL
// =============================================================================

/// A comprehensive list panel component.
///
/// Provides a builder pattern for creating list panels with search, filters,
/// stats, and content sections.
pub struct ListPanel<'a, M> {
    /// Optional panel title
    title: Option<String>,
    /// Search configuration: (value, placeholder, handler)
    #[allow(clippy::type_complexity)]
    search: Option<(String, String, Box<dyn Fn(String) -> M>)>,
    /// Filter toggles: (label, active, on_toggle)
    filters: Vec<(String, bool, M)>,
    /// Stats text
    stats: Option<String>,
    /// Main content items
    items: Option<Element<'a, M>>,
    /// Sections with titles and content
    sections: Vec<(String, Option<usize>, Element<'a, M>)>,
    /// Empty state: (message, clear_action)
    empty_state: Option<(String, M)>,
    /// Whether to show divider after header
    show_divider: bool,
}

impl<'a, M: Clone + 'static> ListPanel<'a, M> {
    /// Create a new list panel.
    pub fn new() -> Self {
        Self {
            title: None,
            search: None,
            filters: Vec::new(),
            stats: None,
            items: None,
            sections: Vec::new(),
            empty_state: None,
            show_divider: true,
        }
    }

    /// Set the panel title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add search functionality.
    ///
    /// # Arguments
    ///
    /// * `value` - Current search text
    /// * `placeholder` - Placeholder text for the input
    /// * `on_change` - Message handler for search text changes
    pub fn search(
        mut self,
        value: &str,
        placeholder: impl Into<String>,
        on_change: impl Fn(String) -> M + 'static,
    ) -> Self {
        self.search = Some((value.to_string(), placeholder.into(), Box::new(on_change)));
        self
    }

    /// Add a filter toggle button.
    ///
    /// # Arguments
    ///
    /// * `label` - Button label
    /// * `active` - Whether the filter is currently active
    /// * `on_toggle` - Message to send when toggled
    pub fn filter(mut self, label: impl Into<String>, active: bool, on_toggle: M) -> Self {
        self.filters.push((label.into(), active, on_toggle));
        self
    }

    /// Add stats text (e.g., "15/20 mapped").
    pub fn stats(mut self, text: impl Into<String>) -> Self {
        self.stats = Some(text.into());
        self
    }

    /// Set the main content items.
    ///
    /// Use this for simple lists without sections.
    pub fn items(mut self, items: impl Into<Element<'a, M>>) -> Self {
        self.items = Some(items.into());
        self
    }

    /// Add a titled section with content.
    ///
    /// Use this for grouped lists with section headers.
    ///
    /// # Arguments
    ///
    /// * `title` - Section title
    /// * `content` - Section content
    pub fn section(mut self, title: impl Into<String>, content: impl Into<Element<'a, M>>) -> Self {
        self.sections.push((title.into(), None, content.into()));
        self
    }

    /// Add a titled section with count badge.
    ///
    /// # Arguments
    ///
    /// * `title` - Section title
    /// * `count` - Item count to display
    /// * `content` - Section content
    pub fn section_with_count(
        mut self,
        title: impl Into<String>,
        count: usize,
        content: impl Into<Element<'a, M>>,
    ) -> Self {
        self.sections
            .push((title.into(), Some(count), content.into()));
        self
    }

    /// Set the empty state display.
    ///
    /// Shown when there are no items and no sections.
    ///
    /// # Arguments
    ///
    /// * `message` - Message to display
    /// * `clear_action` - Message to send when "Clear filters" is clicked
    pub fn empty_state(mut self, message: impl Into<String>, clear_action: M) -> Self {
        self.empty_state = Some((message.into(), clear_action));
        self
    }

    /// Hide the divider after the header.
    pub fn no_divider(mut self) -> Self {
        self.show_divider = false;
        self
    }

    /// Build the list panel element.
    pub fn view(self) -> Element<'a, M> {
        // Destructure to avoid borrow issues
        let Self {
            title,
            search,
            filters,
            stats,
            items,
            sections,
            empty_state,
            show_divider,
        } = self;

        // Track what we have for the divider logic
        let has_title = title.is_some();
        let has_search = search.is_some();
        let has_filters = !filters.is_empty();
        let has_stats = stats.is_some();
        let has_items = items.is_some();
        let has_sections = !sections.is_empty();

        let mut content: Column<'a, M> = column![];

        // Title
        if let Some(title_text) = title {
            content = content.push(
                text(title_text)
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
        }

        // Search input
        if let Some((value, placeholder, on_change)) = search {
            let search_input = text_input(&placeholder, &value)
                .on_input(on_change)
                .padding([8.0, 12.0])
                .size(13)
                .width(Length::Fill);
            content = content.push(search_input);
            content = content.push(Space::new().height(SPACING_XS));
        }

        // Filter toggles
        if has_filters {
            let filter_buttons: Vec<Element<'static, M>> = filters
                .into_iter()
                .map(|(label, active, on_toggle)| filter_toggle(label, active, on_toggle))
                .collect();
            content = content.push(row(filter_buttons).spacing(SPACING_XS));
            content = content.push(Space::new().height(SPACING_SM));
        }

        // Stats
        if let Some(stats_text) = stats {
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
        if show_divider && (has_title || has_search || has_filters || has_stats) {
            content = content.push(rule::horizontal(1));
            content = content.push(Space::new().height(SPACING_SM));
        }

        // Content: empty state, items, or sections
        if !has_items && !has_sections {
            // Show empty state if configured
            if let Some((message, clear_action)) = empty_state {
                content = content.push(Self::build_empty_state(message, clear_action));
            }
        } else {
            // Add main items
            if let Some(items_content) = items {
                content = content.push(items_content);
            }

            // Add sections
            for (section_title, count, section_content) in sections {
                content = content.push(Space::new().height(SPACING_SM));
                content = content.push(Self::build_section(section_title, count, section_content));
            }
        }

        content.into()
    }

    /// Build a section element.
    fn build_section(
        title: String,
        count: Option<usize>,
        content: Element<'a, M>,
    ) -> Element<'a, M> {
        let count_el: Element<'a, M> = if let Some(count) = count {
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
            text(title).size(12).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
            count_el,
        ]
        .align_y(Alignment::Center);

        column![title_row, Space::new().height(SPACING_XS), content,].into()
    }

    /// Build the empty state element.
    fn build_empty_state(message: String, clear_action: M) -> Element<'a, M> {
        container(
            column![
                text(message).size(13).style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
                Space::new().height(SPACING_SM),
                button(text("Clear filters").size(12))
                    .on_press(clear_action)
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
}

impl<'a, M: Clone + 'static> Default for ListPanel<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// SCROLLABLE LIST PANEL
// =============================================================================

/// A list panel wrapped in a scrollable container.
///
/// Use this when the panel content may exceed the available height.
pub struct ScrollableListPanel<'a, M> {
    inner: ListPanel<'a, M>,
}

impl<'a, M: Clone + 'static> ScrollableListPanel<'a, M> {
    /// Create a new scrollable list panel.
    pub fn new() -> Self {
        Self {
            inner: ListPanel::new(),
        }
    }

    /// Set the panel title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.inner = self.inner.title(title);
        self
    }

    /// Add search functionality.
    pub fn search(
        mut self,
        value: &str,
        placeholder: impl Into<String>,
        on_change: impl Fn(String) -> M + 'static,
    ) -> Self {
        self.inner = self.inner.search(value, placeholder, on_change);
        self
    }

    /// Add a filter toggle button.
    pub fn filter(mut self, label: impl Into<String>, active: bool, on_toggle: M) -> Self {
        self.inner = self.inner.filter(label, active, on_toggle);
        self
    }

    /// Add stats text.
    pub fn stats(mut self, text: impl Into<String>) -> Self {
        self.inner = self.inner.stats(text);
        self
    }

    /// Set the main content items.
    pub fn items(mut self, items: impl Into<Element<'a, M>>) -> Self {
        self.inner = self.inner.items(items);
        self
    }

    /// Set the empty state display.
    pub fn empty_state(mut self, message: impl Into<String>, clear_action: M) -> Self {
        self.inner = self.inner.empty_state(message, clear_action);
        self
    }

    /// Build the scrollable list panel element.
    pub fn view(self) -> Element<'a, M> {
        scrollable(self.inner.view()).height(Length::Fill).into()
    }
}

impl<'a, M: Clone + 'static> Default for ScrollableListPanel<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}
