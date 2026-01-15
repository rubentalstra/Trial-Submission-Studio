//! Search and filter bar component.
//!
//! A reusable search input with filter toggle buttons and optional stats display.
//! Used across mapping, normalization, and SUPP tabs.

use iced::widget::{Space, button, column, row, text, text_input};
use iced::{Alignment, Border, Element, Length, Theme};

use crate::theme::{
    BORDER_RADIUS_SM, GRAY_500, GRAY_600, PRIMARY_100, PRIMARY_500, SPACING_SM, SPACING_XS,
    button_secondary,
};

// =============================================================================
// FILTER TOGGLE
// =============================================================================

/// A single filter toggle button with active/inactive state.
///
/// # Example
/// ```ignore
/// FilterToggle::new("Unmapped", is_active, Message::ToggleFilter)
///     .view()
/// ```
pub struct FilterToggle<M> {
    label: String,
    active: bool,
    on_toggle: M,
}

impl<M: Clone + 'static> FilterToggle<M> {
    /// Create a new filter toggle.
    pub fn new(label: impl Into<String>, active: bool, on_toggle: M) -> Self {
        Self {
            label: label.into(),
            active,
            on_toggle,
        }
    }

    /// Build the filter toggle element.
    pub fn view(self) -> Element<'static, M> {
        let active = self.active;
        let label = self.label;
        let on_toggle = self.on_toggle;

        button(text(label).size(11))
            .on_press(on_toggle)
            .padding([4.0, 8.0])
            .style(move |theme: &Theme, status| {
                if active {
                    iced::widget::button::Style {
                        background: Some(PRIMARY_100.into()),
                        text_color: PRIMARY_500,
                        border: Border {
                            radius: BORDER_RADIUS_SM.into(),
                            color: PRIMARY_500,
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
}

// =============================================================================
// SEARCH FILTER BAR
// =============================================================================

/// Search input with filter toggles and optional stats.
///
/// # Example
/// ```ignore
/// SearchFilterBar::new(&search_text, "Search variables...", |s| Message::SearchChanged(s))
///     .filter("Unmapped", filter_unmapped, Message::ToggleUnmapped(!filter_unmapped))
///     .filter("Required", filter_required, Message::ToggleRequired(!filter_required))
///     .stats("15/20 mapped")
///     .view()
/// ```
pub struct SearchFilterBar<M> {
    search_value: String,
    placeholder: String,
    on_search: Box<dyn Fn(String) -> M>,
    filters: Vec<(String, bool, M)>,
    stats_text: Option<String>,
}

impl<M: Clone + 'static> SearchFilterBar<M> {
    /// Create a new search filter bar.
    pub fn new(
        search_value: &str,
        placeholder: impl Into<String>,
        on_search: impl Fn(String) -> M + 'static,
    ) -> Self {
        Self {
            search_value: search_value.to_string(),
            placeholder: placeholder.into(),
            on_search: Box::new(on_search),
            filters: Vec::new(),
            stats_text: None,
        }
    }

    /// Add a filter toggle button.
    pub fn filter(mut self, label: impl Into<String>, active: bool, on_toggle: M) -> Self {
        self.filters.push((label.into(), active, on_toggle));
        self
    }

    /// Add stats text below the filters.
    pub fn stats(mut self, text: impl Into<String>) -> Self {
        self.stats_text = Some(text.into());
        self
    }

    /// Build the search filter bar element.
    pub fn view(self) -> Element<'static, M> {
        let search_value = self.search_value;
        let placeholder = self.placeholder;
        let on_search = self.on_search;

        // Search input
        let search_input = text_input(&placeholder, &search_value)
            .on_input(on_search)
            .padding([8.0, 12.0])
            .size(13)
            .width(Length::Fill);

        // Filter buttons
        let filter_buttons: Vec<Element<'static, M>> = self
            .filters
            .into_iter()
            .map(|(label, active, on_toggle)| FilterToggle::new(label, active, on_toggle).view())
            .collect();

        let filters_row = if filter_buttons.is_empty() {
            None
        } else {
            Some(row(filter_buttons).spacing(SPACING_XS))
        };

        // Stats text
        let stats_el: Option<Element<'static, M>> = self.stats_text.map(|stats_text| {
            // Split stats text into number and label parts
            let (num_part, label_part) = if let Some((n, r)) = stats_text.split_once(' ') {
                (n.to_string(), r.to_string())
            } else {
                (stats_text, String::new())
            };

            row![
                text(num_part).size(12).color(GRAY_600),
                Space::new().width(4.0),
                text(label_part).size(11).color(GRAY_500),
            ]
            .align_y(Alignment::Center)
            .into()
        });

        // Build the column
        let mut content = column![search_input,].spacing(SPACING_XS);

        if let Some(filters) = filters_row {
            content = content.push(filters);
        }

        if let Some(stats) = stats_el {
            content = content.push(Space::new().height(SPACING_SM));
            content = content.push(stats);
        }

        content.into()
    }
}
