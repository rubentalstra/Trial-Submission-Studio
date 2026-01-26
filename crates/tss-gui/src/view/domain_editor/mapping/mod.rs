//! Mapping tab view.
//!
//! The mapping tab displays a master-detail interface for mapping
//! source columns to SDTM target variables.
//!
//! - **Left (Master)**: List of TARGET SDTM variables with status indicators
//! - **Right (Detail)**: Selected variable details with mapping controls

mod actions;
mod detail;
mod master;

use iced::widget::{container, text};
use iced::{Element, Theme};
use iced_fonts::lucide;

use crate::component::display::EmptyState;
use crate::component::layout::SplitView;
use crate::message::Message;
use crate::state::{AppState, ViewState};
use crate::theme::{ClinicalColors, MASTER_WIDTH};
use crate::util::matches_search_any;
use crate::view::domain_editor::detail_no_selection_default;

use tss_standards::CoreDesignation;
use tss_submit::VariableStatus;

use detail::view_variable_detail;
use master::{view_variable_list_content, view_variable_list_header};

// =============================================================================
// MAIN MAPPING TAB VIEW
// =============================================================================

/// Render the mapping tab content using master-detail layout.
pub fn view_mapping_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return EmptyState::new(
                container(lucide::circle_alert().size(48)).style(|theme: &Theme| {
                    container::Style {
                        text_color: Some(theme.clinical().text_muted),
                        ..Default::default()
                    }
                }),
                "Domain not found",
            )
            .centered()
            .view();
        }
    };

    // Mapping only applies to source domains
    let source = match domain.as_source() {
        Some(s) => s,
        None => {
            return EmptyState::new(
                container(lucide::info().size(48)).style(|theme: &Theme| container::Style {
                    text_color: Some(theme.clinical().text_muted),
                    ..Default::default()
                }),
                "Generated domains do not require mapping",
            )
            .centered()
            .view();
        }
    };

    let mapping_ui = match &state.view {
        ViewState::DomainEditor(editor) => &editor.mapping_ui,
        _ => return text("Invalid view state").into(),
    };

    let sdtm_domain = source.mapping.domain();

    // Apply filters
    let filtered_indices: Vec<usize> = sdtm_domain
        .variables
        .iter()
        .enumerate()
        .filter(|(_idx, var)| {
            // Search filter - check name and label
            let label = var.label.as_deref().unwrap_or("");
            if !matches_search_any(&[&var.name, label], &mapping_ui.search_filter) {
                return false;
            }

            // Unmapped filter
            if mapping_ui.filter_unmapped {
                let status = source.mapping.status(&var.name);
                if !matches!(status, VariableStatus::Unmapped | VariableStatus::Suggested) {
                    return false;
                }
            }

            // Required filter
            if mapping_ui.filter_required && var.core != Some(CoreDesignation::Required) {
                return false;
            }

            true
        })
        .map(|(idx, _)| idx)
        .collect();

    let master_header = view_variable_list_header(source, mapping_ui);
    let master_content = view_variable_list_content(source, &filtered_indices, mapping_ui);
    let detail = if let Some(selected_idx) = mapping_ui.selected_variable {
        if let Some(var) = sdtm_domain.variables.get(selected_idx) {
            view_variable_detail(state, source, var)
        } else {
            detail_no_selection_default(
                "Select a Variable",
                "Click a variable from the list to view details and configure mapping",
            )
        }
    } else {
        detail_no_selection_default(
            "Select a Variable",
            "Click a variable from the list to view details and configure mapping",
        )
    };

    SplitView::new(master_content, detail)
        .master_width(MASTER_WIDTH)
        .master_header(master_header)
        .view()
}
