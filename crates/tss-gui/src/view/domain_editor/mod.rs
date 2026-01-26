//! Domain editor view with tabbed interface.
//!
//! The domain editor provides a multi-tab interface for:
//! - **Mapping**: Map source columns to CDISC variables
//! - **Normalization**: Configure data normalization rules
//! - **Validation**: Review CDISC conformance issues
//! - **Preview**: Preview transformed output data
//! - **SUPP**: Configure supplemental qualifiers

pub mod mapping;
pub mod normalization;
pub mod preview;
pub mod supp;
pub mod validation;

use iced::widget::{column, container, text};
use iced::{Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::display::EmptyState;
use crate::component::layout::{PageHeader, Tab, tab_bar};
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, DomainState, EditorTab};
use crate::theme::ClinicalColors;

// Re-export tab view functions
pub use mapping::view_mapping_tab;
pub use normalization::view_normalization_tab;
pub use preview::view_preview_tab;
pub use supp::view_supp_tab;
pub use validation::view_validation_tab;

// =============================================================================
// MAIN DOMAIN EDITOR VIEW
// =============================================================================

/// Render the domain editor view.
///
/// Shows a header with domain info, tab bar, and the active tab content.
pub fn view_domain_editor<'a>(
    state: &'a AppState,
    domain_code: &'a str,
    tab: EditorTab,
) -> Element<'a, Message> {
    // Get domain data
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return container(text("Domain not found").size(16).style(|theme: &Theme| {
                text::Style {
                    color: Some(theme.clinical().text_muted),
                }
            }))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Shrink)
            .center_y(Length::Shrink)
            .into();
        }
    };

    // Header with domain info and back button
    let header = view_editor_header(domain_code, domain);

    // Tab bar
    let tabs = view_tab_bar(tab);

    // Tab content
    let content = match tab {
        EditorTab::Mapping => view_mapping_tab(state, domain_code),
        EditorTab::Normalization => view_normalization_tab(state, domain_code),
        EditorTab::Validation => view_validation_tab(state, domain_code),
        EditorTab::Preview => view_preview_tab(state, domain_code),
        EditorTab::Supp => view_supp_tab(state, domain_code),
    };

    // Main layout - content fills remaining space
    // Note: Don't wrap in scrollable here - tabs like Mapping use master_detail
    // which manages its own scrolling
    column![
        header,
        tabs,
        container(content).width(Length::Fill).height(Length::Fill),
    ]
    .spacing(0)
    .into()
}

// =============================================================================
// HEADER
// =============================================================================

/// Domain editor header with domain info and back button.
fn view_editor_header<'a>(domain_code: &'a str, domain: &'a DomainState) -> Element<'a, Message> {
    let display_name = domain.display_name(domain_code);
    let summary = domain.summary();

    let progress_pct = if summary.total_variables > 0 {
        (summary.mapped * 100) / summary.total_variables
    } else {
        0
    };

    PageHeader::new(display_name)
        .back(Message::DomainEditor(DomainEditorMessage::BackClicked))
        .badge_themed(domain_code, |theme: &Theme| {
            theme.extended_palette().primary.base.color
        })
        .meta("Rows", domain.row_count().to_string())
        .meta(
            "Progress",
            format!(
                "{}/{} ({}%)",
                summary.mapped, summary.total_variables, progress_pct
            ),
        )
        .view()
}

// =============================================================================
// TAB BAR
// =============================================================================

/// Tab bar for switching between editor tabs.
fn view_tab_bar<'a>(current_tab: EditorTab) -> Element<'a, Message> {
    let tabs: Vec<Tab<Message>> = EditorTab::ALL
        .iter()
        .map(|tab| {
            Tab::new(
                tab.name(),
                Message::DomainEditor(DomainEditorMessage::TabSelected(*tab)),
            )
        })
        .collect();

    // Find the active tab index
    let active_index = EditorTab::ALL
        .iter()
        .position(|tab| *tab == current_tab)
        .unwrap_or(0);

    container(tab_bar(tabs, active_index))
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let clinical = theme.clinical();
            container::Style {
                background: Some(clinical.background_elevated.into()),
                border: Border {
                    width: 1.0,
                    radius: 0.0.into(),
                    color: clinical.border_default,
                },
                ..Default::default()
            }
        })
        .into()
}

// =============================================================================
// SHARED HELPERS
// =============================================================================

/// Create a "no selection" empty state for the detail panel.
///
/// Used by all tabs when no item is selected in the master list.
/// Displays a centered icon, title, and description.
///
/// # Arguments
///
/// * `icon` - Lucide icon to display (e.g., `lucide::mouse_pointer_click`)
/// * `title` - Title text (e.g., "Select a Variable")
/// * `description` - Description text explaining what to do
pub fn detail_no_selection<'a>(
    icon: impl Into<Element<'a, Message>>,
    title: &'a str,
    description: &'a str,
) -> Element<'a, Message> {
    EmptyState::new(
        container(icon).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_disabled),
            ..Default::default()
        }),
        title,
    )
    .description(description)
    .centered()
    .view()
}

/// Create a "no selection" empty state with the default click icon.
///
/// Convenience function using the standard `mouse_pointer_click` icon at size 48.
pub fn detail_no_selection_default<'a>(
    title: &'a str,
    description: &'a str,
) -> Element<'a, Message> {
    detail_no_selection(lucide::mouse_pointer_click().size(48), title, description)
}
