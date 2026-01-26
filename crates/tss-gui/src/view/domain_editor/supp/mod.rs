//! SUPP (Supplemental Qualifiers) tab view.
//!
//! # Architecture
//!
//! The SUPP tab uses a clean state-based UX:
//!
//! - **Pending**: Editable fields + sample data + "Add to SUPP"/"Skip" buttons
//! - **Included (view)**: Read-only summary + "Edit"/"Remove" options
//! - **Included (edit)**: Editable fields + "Save"/"Cancel" buttons
//! - **Skipped**: Skip message + sample data + "Add to SUPP instead" button
//!
//! # Edit Draft Pattern
//!
//! For pending columns, edits go directly to `supp_config`.
//! For included columns in edit mode, edits go to `edit_draft` and are
//! committed only on "Save".

mod detail;
mod helpers;
mod master;

use iced::widget::{container, text};
use iced::{Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::display::EmptyState;
use crate::component::layout::SplitView;
use crate::message::Message;
use crate::state::{AppState, SuppAction, SuppFilterMode, ViewState};
use crate::theme::{ClinicalColors, MASTER_WIDTH};
use crate::util::matches_search;

use detail::build_detail_panel;
use master::{build_master_content, build_master_header_pinned};

// =============================================================================
// MAIN SUPP TAB VIEW
// =============================================================================

/// Render the SUPP configuration tab content.
pub fn view_supp_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return container(text("Domain not found").size(14).style(|theme: &Theme| {
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

    // SUPP configuration only applies to source domains
    let source = match domain.as_source() {
        Some(s) => s,
        None => {
            return container(
                text("Generated domains do not have SUPP columns")
                    .size(14)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted),
                    }),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Shrink)
            .center_y(Length::Shrink)
            .into();
        }
    };

    // Get UI state
    let supp_ui = match &state.view {
        ViewState::DomainEditor(editor) => &editor.supp_ui,
        _ => return text("Invalid view state").into(),
    };

    // Get unmapped columns
    let unmapped_columns = source.unmapped_columns();

    // If no unmapped columns, show success state
    if unmapped_columns.is_empty() {
        return view_all_mapped_state(domain_code);
    }

    // Filter columns based on search and filter mode
    let filtered: Vec<String> = unmapped_columns
        .iter()
        .filter(|col: &&String| {
            // Search filter
            if !matches_search(col, &supp_ui.search_filter) {
                return false;
            }

            // Action filter
            let supp_config = source.supp_config.get(*col);
            match supp_ui.filter_mode {
                SuppFilterMode::All => true,
                SuppFilterMode::Pending => {
                    supp_config.is_none_or(|c| c.action == SuppAction::Pending)
                }
                SuppFilterMode::Included => {
                    supp_config.is_some_and(|c| c.action == SuppAction::Include)
                }
                SuppFilterMode::Skipped => {
                    supp_config.is_some_and(|c| c.action == SuppAction::Skip)
                }
            }
        })
        .cloned()
        .collect();

    // Build master header (pinned at top)
    let master_header = build_master_header_pinned(supp_ui, filtered.len());

    // Build master content (scrollable column list)
    let master_content = build_master_content(&filtered, source, supp_ui);

    // Build detail panel
    let detail = build_detail_panel(source, supp_ui, domain_code);

    // Use split view layout with pinned header
    SplitView::new(master_content, detail)
        .master_width(MASTER_WIDTH)
        .master_header(master_header)
        .view()
}

// =============================================================================
// ALL MAPPED STATE
// =============================================================================

fn view_all_mapped_state(domain_code: &str) -> Element<'static, Message> {
    let description = format!(
        "All source columns are mapped to {} variables. No SUPP configuration needed.",
        domain_code
    );

    EmptyState::new(
        container(lucide::circle_check().size(48)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().success.base.color),
            ..Default::default()
        }),
        "All Columns Mapped",
    )
    .description(description)
    .centered()
    .view()
}
