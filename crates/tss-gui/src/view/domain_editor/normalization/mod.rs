//! Normalization tab view.
//!
//! The normalization tab displays data normalization rules that are automatically
//! inferred from SDTM variable metadata. Users can see what transformations
//! will be applied to each variable during export.
//!
//! - **Left (Master)**: List of variables with their normalization types
//! - **Right (Detail)**: Detailed view of the selected rule

mod detail;
mod helpers;
mod master;

use iced::widget::{container, text};
use iced::{Element, Theme};
use iced_fonts::lucide;

use crate::component::display::EmptyState;
use crate::component::layout::SplitView;
use crate::message::Message;
use crate::state::{AppState, ViewState};
use crate::theme::{ClinicalColors, MASTER_WIDTH};
use crate::view::domain_editor::detail_no_selection;

use detail::view_rule_detail;
use master::{view_rules_header, view_rules_list};

// =============================================================================
// MAIN VIEW
// =============================================================================

/// Render the normalization tab content using master-detail layout.
pub fn view_normalization_tab<'a>(
    state: &'a AppState,
    domain_code: &'a str,
) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return EmptyState::new(
                container(lucide::circle_alert().size(48)).style(|theme: &Theme| {
                    container::Style {
                        text_color: Some(theme.clinical().text_disabled),
                        ..Default::default()
                    }
                }),
                "Domain not found",
            )
            .centered()
            .view();
        }
    };

    // Normalization only applies to source domains
    let source = match domain.as_source() {
        Some(s) => s,
        None => {
            return EmptyState::new(
                container(lucide::info().size(48)).style(|theme: &Theme| container::Style {
                    text_color: Some(theme.clinical().text_muted),
                    ..Default::default()
                }),
                "Generated domains do not require normalization",
            )
            .centered()
            .view();
        }
    };

    let normalization_ui = match &state.view {
        ViewState::DomainEditor(editor) => &editor.normalization_ui,
        _ => return text("Invalid view state").into(),
    };

    let normalization = &source.normalization;
    let sdtm_domain = source.mapping.domain();

    let master_header = view_rules_header(normalization.rules.len(), &normalization.rules);
    let master_content = view_rules_list(source, &normalization.rules, normalization_ui);
    let detail = if let Some(selected_idx) = normalization_ui.selected_rule {
        if let Some(rule) = normalization.rules.get(selected_idx) {
            view_rule_detail(source, rule, sdtm_domain, state.terminology.as_ref())
        } else {
            detail_no_selection(
                lucide::wand_sparkles().size(48),
                "Select a Rule",
                "Click a variable from the list to view its normalization details",
            )
        }
    } else {
        detail_no_selection(
            lucide::wand_sparkles().size(48),
            "Select a Rule",
            "Click a variable from the list to view its normalization details",
        )
    };

    SplitView::new(master_content, detail)
        .master_width(MASTER_WIDTH)
        .master_header(master_header)
        .view()
}
