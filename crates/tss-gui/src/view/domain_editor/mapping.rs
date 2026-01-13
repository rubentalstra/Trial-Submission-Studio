//! Mapping tab view.
//!
//! The mapping tab displays a master-detail interface for mapping
//! source columns to CDISC variables.

use iced::widget::{Space, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length};

use crate::message::Message;
use crate::state::{AppState, ViewState};
use crate::theme::{
    GRAY_100, GRAY_200, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900, SPACING_LG,
    SPACING_MD, SPACING_SM, SUCCESS, WARNING, WHITE,
};

// =============================================================================
// MAIN MAPPING TAB VIEW
// =============================================================================

/// Render the mapping tab content.
pub fn view_mapping_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return text("Domain not found").size(14).color(GRAY_500).into();
        }
    };

    // Get UI state from view
    let _mapping_ui = match &state.view {
        ViewState::DomainEditor { mapping_ui, .. } => mapping_ui,
        _ => return text("Invalid view state").into(),
    };

    // Header
    let header = view_mapping_header(domain_code);

    // Summary stats
    let stats = view_mapping_stats(domain);

    // Variable list placeholder
    let variable_list = view_variable_list_placeholder(domain);

    column![
        header,
        Space::new().height(SPACING_MD),
        stats,
        Space::new().height(SPACING_LG),
        scrollable(variable_list).height(Length::Fill),
    ]
    .spacing(0)
    .into()
}

// =============================================================================
// HEADER
// =============================================================================

/// Mapping tab header.
fn view_mapping_header<'a>(domain_code: &str) -> Element<'a, Message> {
    let title = text("Variable Mapping").size(18).color(GRAY_900);

    let subtitle = text(format!(
        "Map source columns to {} SDTM variables",
        domain_code
    ))
    .size(13)
    .color(GRAY_600);

    column![title, Space::new().height(4.0), subtitle,].into()
}

// =============================================================================
// STATS
// =============================================================================

/// Mapping progress stats.
fn view_mapping_stats<'a>(domain: &'a crate::state::Domain) -> Element<'a, Message> {
    let row_count = domain.row_count();
    let is_complete = domain.is_mapping_complete();
    let is_touched = domain.is_touched();

    let status_text = if is_complete {
        "Mapping Complete"
    } else if is_touched {
        "Mapping In Progress"
    } else {
        "Not Started"
    };

    let _status_color = if is_complete {
        SUCCESS
    } else if is_touched {
        WARNING
    } else {
        GRAY_500
    };

    let stats_row = row![
        view_stat_badge("Rows", row_count.to_string()),
        Space::new().width(SPACING_MD),
        view_stat_badge("Status", status_text.to_string()),
    ]
    .align_y(Alignment::Center);

    container(stats_row)
        .padding(SPACING_MD)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(GRAY_100.into()),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// Single stat badge.
fn view_stat_badge<'a>(label: &'static str, value: String) -> Element<'a, Message> {
    column![
        text(label).size(11).color(GRAY_500),
        text(value).size(14).color(GRAY_800),
    ]
    .into()
}

// =============================================================================
// VARIABLE LIST (Placeholder)
// =============================================================================

/// Placeholder variable list showing source columns.
fn view_variable_list_placeholder<'a>(domain: &'a crate::state::Domain) -> Element<'a, Message> {
    let source_columns = domain.source.column_names();

    let header = text("Source Columns").size(14).color(GRAY_700);

    let mut items = column![].spacing(4.0);

    for (idx, col) in source_columns.iter().take(20).enumerate() {
        let item = container(
            row![
                text(format!("{}.", idx + 1)).size(12).color(GRAY_400),
                Space::new().width(SPACING_SM),
                text(col.to_string()).size(13).color(GRAY_800),
            ]
            .align_y(Alignment::Center),
        )
        .padding([8.0, 12.0])
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(WHITE.into()),
            border: Border {
                radius: 4.0.into(),
                color: GRAY_200,
                width: 1.0,
            },
            ..Default::default()
        });

        items = items.push(item);
    }

    if source_columns.len() > 20 {
        items = items.push(
            text(format!(
                "... and {} more columns",
                source_columns.len() - 20
            ))
            .size(12)
            .color(GRAY_400),
        );
    }

    column![
        header,
        Space::new().height(SPACING_SM),
        text("Full mapping interface coming soon - this shows available source columns")
            .size(12)
            .color(GRAY_500),
        Space::new().height(SPACING_MD),
        items,
    ]
    .into()
}
