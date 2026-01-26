//! Validation tab view.
//!
//! The validation tab displays CDISC conformance issues found during
//! validation of the mapped and normalized data.
//!
//! - **Left (Master)**: Filterable list of validation issues
//! - **Right (Detail)**: Detailed view of the selected issue

mod detail;
mod helpers;
mod master;

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Length, Theme};
use iced_fonts::lucide;
use tss_submit::{Issue, Severity, ValidationReport};

use crate::component::display::EmptyState;
use crate::component::layout::SplitView;
use crate::message::domain_editor::ValidationMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, SeverityFilter, ViewState};
use crate::theme::{
    ClinicalColors, MASTER_WIDTH, SPACING_LG, SPACING_MD, SPACING_XS, button_secondary,
};
use crate::view::domain_editor::detail_no_selection_default;

use detail::view_issue_detail;
use master::{view_issues_list, view_master_header};

// =============================================================================
// MAIN VALIDATION TAB VIEW
// =============================================================================

/// Render the validation tab content.
pub fn view_validation_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return text("Domain not found")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                })
                .into();
        }
    };

    // Get validation UI state from view
    let validation_ui = match &state.view {
        ViewState::DomainEditor(editor) => &editor.validation_ui,
        _ => return text("Invalid view state").into(),
    };

    // Get validation cache from domain (persists across navigation)
    let report: &ValidationReport = match domain.validation_cache() {
        Some(r) => r,
        None => return view_no_validation_run(),
    };

    // If validation passed (no issues), show success state
    if report.is_empty() {
        return view_validation_passed();
    }

    // Filter issues by severity
    let filtered_issues: Vec<(usize, &Issue)> = report
        .issues
        .iter()
        .enumerate()
        .filter(|(_, issue)| match validation_ui.severity_filter {
            SeverityFilter::All => true,
            SeverityFilter::Errors => {
                matches!(issue.severity(), Severity::Error | Severity::Reject)
            }
            SeverityFilter::Warnings => matches!(issue.severity(), Severity::Warning),
            SeverityFilter::Info => matches!(issue.severity(), Severity::Info),
        })
        .collect();

    // Master panel header
    let master_header = view_master_header(report, validation_ui, filtered_issues.len());

    // Master panel content (list)
    let master_content = view_issues_list(&filtered_issues, validation_ui);

    // Detail panel
    let detail = if let Some(selected_idx) = validation_ui.selected_issue {
        if let Some(issue) = report.issues.get(selected_idx) {
            view_issue_detail(issue)
        } else {
            detail_no_selection_default(
                "Select an Issue",
                "Choose a validation issue from the list to view details",
            )
        }
    } else {
        detail_no_selection_default(
            "Select an Issue",
            "Choose a validation issue from the list to view details",
        )
    };

    SplitView::new(master_content, detail)
        .master_width(MASTER_WIDTH)
        .master_header(master_header)
        .view()
}

// =============================================================================
// EMPTY STATES
// =============================================================================

/// Empty state when no validation has been run.
fn view_no_validation_run<'a>() -> Element<'a, Message> {
    EmptyState::new(
        container(lucide::shield_check().size(48)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_disabled),
            ..Default::default()
        }),
        "No Validation Results",
    )
    .description("Click 'Re-validate' to check for CDISC conformance issues")
    .action(
        "Run Validation",
        Message::DomainEditor(DomainEditorMessage::Validation(
            ValidationMessage::RefreshValidation,
        )),
    )
    .centered()
    .view()
}

/// Success state when validation passed.
fn view_validation_passed<'a>() -> Element<'a, Message> {
    let refresh_btn = button(
        row![lucide::refresh_cw().size(14), text("Re-validate").size(13),]
            .spacing(SPACING_XS)
            .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Validation(
        ValidationMessage::RefreshValidation,
    )))
    .padding([10.0, 16.0])
    .style(button_secondary);

    container(
        column![
            container(lucide::circle_check().size(64)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().success.base.color),
                ..Default::default()
            }),
            Space::new().height(SPACING_MD),
            text("All Checks Passed")
                .size(20)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().success.base.color),
                })
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                }),
            Space::new().height(SPACING_XS),
            text("No CDISC conformance issues were found")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
            Space::new().height(SPACING_LG),
            refresh_btn,
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}
