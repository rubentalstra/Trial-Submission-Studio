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

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length};
use iced_fonts::lucide;

use crate::component::{Tab, tab_bar};
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, DomainState, EditorTab};
use crate::theme::{
    GRAY_100, GRAY_200, GRAY_500, GRAY_900, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, WHITE,
    button_secondary,
};

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
            return container(text("Domain not found").size(16).color(GRAY_500))
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

    // Back button
    let back_button = button(
        row![lucide::chevron_left().size(12), text("Back").size(14),]
            .spacing(SPACING_SM)
            .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::BackClicked))
    .padding([8.0, 16.0])
    .style(button_secondary);

    // Domain badge
    let domain_badge = container(text(domain_code).size(14).color(WHITE))
        .padding([4.0, 12.0])
        .style(move |_theme| container::Style {
            background: Some(PRIMARY_500.into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    // Domain name
    let domain_name = text(display_name).size(20).color(GRAY_900);

    // Row count
    let row_count = text(format!("{} rows", domain.row_count()))
        .size(13)
        .color(GRAY_500);

    // Progress indicator
    let summary = domain.summary();
    let progress_text = text(format!(
        "{}/{} mapped ({}%)",
        summary.mapped,
        summary.total_variables,
        if summary.total_variables > 0 {
            (summary.mapped * 100) / summary.total_variables
        } else {
            0
        }
    ))
    .size(12)
    .color(GRAY_500);

    let header_row = row![
        back_button,
        Space::new().width(SPACING_MD),
        domain_badge,
        Space::new().width(SPACING_SM),
        domain_name,
        Space::new().width(Length::Fill),
        row_count,
        Space::new().width(SPACING_MD),
        progress_text,
    ]
    .align_y(Alignment::Center);

    container(header_row)
        .width(Length::Fill)
        .padding([SPACING_MD, SPACING_LG])
        .style(|_theme| container::Style {
            background: Some(GRAY_100.into()),
            border: Border {
                width: 0.0,
                radius: 0.0.into(),
                color: GRAY_200,
            },
            ..Default::default()
        })
        .into()
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
        .style(|_theme| container::Style {
            background: Some(WHITE.into()),
            border: Border {
                width: 1.0,
                radius: 0.0.into(),
                color: GRAY_200,
            },
            ..Default::default()
        })
        .into()
}
