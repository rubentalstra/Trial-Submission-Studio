//! Preview tab
//!
//! Shows transformed data before export with pagination.
//! Preview is computed lazily when this tab is opened.

use crate::services::{PreviewState, ensure_preview};
use crate::state::AppState;
use crate::theme::spacing;
use egui::{RichText, Ui};
use polars::prelude::DataFrame;
use tss_model::any_to_string;

/// Render the preview tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Step 1: Ensure preview is ready
    match ensure_preview(state, domain_code) {
        PreviewState::Rebuilding => {
            show_spinner(ui, "Building preview...");
            ui.ctx().request_repaint();
            return;
        }
        PreviewState::NotConfigured => {
            show_message(ui, "Configure mappings to see preview");
            return;
        }
        PreviewState::Error(e) => {
            show_error(ui, &e);
            return;
        }
        PreviewState::Ready => {
            // Continue to render content
        }
    }

    // Step 2: Check domain accessibility
    let Some(domain) = state.domain(domain_code) else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Domain not accessible").color(ui.visuals().error_fg_color));
        });
        return;
    };

    // Step 3: Get preview DataFrame (guaranteed to exist after PreviewState::Ready)
    let Some(preview_df) = domain.derived.preview.as_ref() else {
        // This shouldn't happen, but handle gracefully
        show_message(ui, "Preview data not available");
        return;
    };

    // Clone DataFrame for rendering (needed because we borrow state mutably later)
    let preview_df = preview_df.clone();

    // Step 4: Render the preview UI
    show_preview_content(ui, state, domain_code, &preview_df);
}

/// Show spinner with message
fn show_spinner(ui: &mut Ui, message: &str) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.spinner();
            ui.add_space(spacing::SM);
            ui.label(RichText::new(message).weak());
        });
    });
}

/// Show informational message
fn show_message(ui: &mut Ui, message: &str) {
    ui.centered_and_justified(|ui| {
        ui.label(RichText::new(message).weak());
    });
}

/// Show error state
fn show_error(ui: &mut Ui, error: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(spacing::XL);
        ui.label(
            RichText::new(format!("{} Preview Error", egui_phosphor::regular::WARNING))
                .color(ui.visuals().error_fg_color)
                .size(18.0),
        );
        ui.add_space(spacing::MD);
        ui.label(RichText::new(error).color(ui.visuals().weak_text_color()));
        ui.add_space(spacing::MD);
        ui.label(
            RichText::new("Check that all required mappings are configured.")
                .weak()
                .small(),
        );
    });
}

/// Show the preview content with table and pagination
fn show_preview_content(ui: &mut Ui, state: &mut AppState, domain_code: &str, df: &DataFrame) {
    let total_rows = df.height();

    // Get pagination state from UI state
    let ui_state = state.ui.domain_editor(domain_code);
    let current_page = ui_state.preview.current_page;
    let rows_per_page = ui_state.preview.rows_per_page;

    let total_pages = total_rows.div_ceil(rows_per_page);
    let start_row = current_page * rows_per_page;
    let row_count = rows_per_page.min(total_rows.saturating_sub(start_row));

    // Header
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{} SDTM Preview", egui_phosphor::regular::TABLE)).strong());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{} rows × {} columns", total_rows, df.width()))
                    .weak()
                    .small(),
            );
        });
    });

    ui.add_space(spacing::SM);
    ui.separator();
    ui.add_space(spacing::SM);

    // Pagination controls at top
    show_pagination_controls(
        ui,
        state,
        domain_code,
        total_rows,
        current_page,
        total_pages,
    );

    ui.add_space(spacing::SM);

    // Data table
    let column_names: Vec<String> = df
        .get_column_names()
        .into_iter()
        .map(polars::prelude::PlSmallStr::to_string)
        .collect();
    let available_height = ui.available_height() - 40.0; // Reserve space for bottom pagination

    egui::ScrollArea::horizontal()
        .max_height(available_height)
        .show(ui, |ui| {
            show_data_table(ui, df, &column_names, start_row, row_count);
        });

    ui.add_space(spacing::SM);

    // Pagination controls at bottom
    show_pagination_controls(
        ui,
        state,
        domain_code,
        total_rows,
        current_page,
        total_pages,
    );
}

/// Show the data table
fn show_data_table(
    ui: &mut Ui,
    df: &DataFrame,
    column_names: &[String],
    start_row: usize,
    row_count: usize,
) {
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .columns(
            egui_extras::Column::auto().at_least(60.0).clip(true),
            column_names.len(),
        )
        .header(24.0, |mut header| {
            for col_name in column_names {
                header.col(|ui| {
                    ui.label(RichText::new(col_name).strong().monospace());
                });
            }
        })
        .body(|body| {
            body.rows(text_height + 8.0, row_count, |mut row| {
                let row_idx = start_row + row.index();
                for col_name in column_names {
                    row.col(|ui| {
                        let value = df
                            .column(col_name)
                            .ok()
                            .and_then(|s| s.get(row_idx).ok())
                            .map(any_to_string)
                            .unwrap_or_default();
                        ui.label(RichText::new(value).monospace().small());
                    });
                }
            });
        });
}

/// Show pagination controls
fn show_pagination_controls(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    total_rows: usize,
    current_page: usize,
    total_pages: usize,
) {
    ui.horizontal(|ui| {
        // Previous button
        let can_go_prev = current_page > 0;
        if ui
            .add_enabled(
                can_go_prev,
                egui::Button::new(format!("{} Prev", egui_phosphor::regular::CARET_LEFT)),
            )
            .clicked()
        {
            state.ui.domain_editor(domain_code).preview.prev_page();
        }

        // Page indicator
        ui.label(format!(
            "Page {} of {}",
            current_page + 1,
            total_pages.max(1)
        ));

        // Next button
        let can_go_next = current_page + 1 < total_pages;
        if ui
            .add_enabled(
                can_go_next,
                egui::Button::new(format!("Next {}", egui_phosphor::regular::CARET_RIGHT)),
            )
            .clicked()
        {
            state.ui.domain_editor(domain_code).preview.next_page();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let rows_per_page = state
                .ui
                .get_domain_editor(domain_code)
                .map(|ui| ui.preview.rows_per_page)
                .unwrap_or(50);
            let start_row = current_page * rows_per_page + 1;
            let end_row = ((current_page + 1) * rows_per_page).min(total_rows);
            ui.label(
                RichText::new(format!(
                    "Showing rows {}-{} of {}",
                    start_row, end_row, total_rows
                ))
                .weak()
                .small(),
            );
        });
    });
}
