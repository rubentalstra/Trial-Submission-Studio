//! Preview tab
//!
//! Shows transformed data before export with pagination.

use crate::state::AppState;
use crate::theme::spacing;
use crate::views::domain_editor::ensure_mapping_initialized;
use egui::{RichText, Ui};
use polars::prelude::DataFrame;
use sdtm_standards::load_default_ct_registry;
use sdtm_transform::build_preview_dataframe_with_omitted;
use std::collections::{BTreeMap, BTreeSet};

/// Render the preview tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Ensure mapping is initialized first
    if !ensure_mapping_initialized(ui, state, domain_code) {
        return;
    }

    // Lazy rebuild preview data if needed
    rebuild_preview_if_needed(state, domain_code);

    // Get domain state for display
    let Some(study) = &state.study else {
        ui.label("No study loaded");
        return;
    };

    let Some(domain) = study.get_domain(domain_code) else {
        ui.label(format!("Domain {} not found", domain_code));
        return;
    };

    // Check for error state
    if let Some(ref error) = domain.preview_state.error {
        show_error_state(ui, error);
        return;
    }

    // Check if preview data is available - clone to avoid borrow issues
    let Some(preview_df) = domain.preview_data.clone() else {
        ui.centered_and_justified(|ui| {
            ui.spinner();
            ui.label("Building preview...");
        });
        return;
    };

    // Render the preview UI
    show_preview_content(ui, state, domain_code, &preview_df);
}

/// Show error state
fn show_error_state(ui: &mut Ui, error: &str) {
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

    // Get pagination state
    let (current_page, rows_per_page) = {
        let study = state.study.as_ref().unwrap();
        let domain = study.get_domain(domain_code).unwrap();
        (
            domain.preview_state.current_page,
            domain.preview_state.rows_per_page,
        )
    };

    let total_pages = (total_rows + rows_per_page - 1).max(1) / rows_per_page.max(1);
    let start_row = current_page * rows_per_page;
    let end_row = (start_row + rows_per_page).min(total_rows);

    // Header with stats
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} SDTM Output Preview",
                egui_phosphor::regular::EYE
            ))
            .strong(),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{} rows · {} columns", total_rows, df.width()))
                    .weak()
                    .small(),
            );
        });
    });

    ui.add_space(spacing::SM);
    ui.separator();
    ui.add_space(spacing::SM);

    // Notes about transforms applied
    show_transform_notes(ui, domain_code);

    ui.add_space(spacing::SM);

    // Data table
    let available_height = ui.available_height() - 50.0; // Reserve space for pagination
    egui::ScrollArea::both()
        .max_height(available_height)
        .show(ui, |ui| {
            show_data_table(ui, df, start_row, end_row);
        });

    ui.add_space(spacing::SM);
    ui.separator();

    // Pagination controls
    show_pagination_controls(
        ui,
        state,
        domain_code,
        total_rows,
        current_page,
        total_pages,
    );
}

/// Show notes about which transforms were applied
fn show_transform_notes(ui: &mut Ui, domain_code: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(format!("{} Transforms applied:", egui_phosphor::regular::INFO))
                .weak()
                .small(),
        );
        ui.label(
            RichText::new(format!(
                "STUDYID/DOMAIN constants · USUBJID prefix · {}SEQ sequence · CT normalized · ISO 8601 dates",
                domain_code
            ))
            .weak()
            .small(),
        );
    });
}

/// Render the data table with columns and rows
fn show_data_table(ui: &mut Ui, df: &DataFrame, start_row: usize, end_row: usize) {
    if df.width() == 0 || df.height() == 0 {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("No data to display").weak());
        });
        return;
    }

    let column_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let row_count = end_row.saturating_sub(start_row);

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
            for col_name in &column_names {
                header.col(|ui| {
                    ui.label(RichText::new(col_name).strong().monospace());
                });
            }
        })
        .body(|body| {
            body.rows(text_height + 8.0, row_count, |mut row| {
                let row_idx = start_row + row.index();
                for col_name in &column_names {
                    row.col(|ui| {
                        let value = get_cell_value(df, col_name, row_idx);
                        ui.label(RichText::new(value).monospace().small());
                    });
                }
            });
        });
}

/// Get cell value as string
fn get_cell_value(df: &DataFrame, col_name: &str, row_idx: usize) -> String {
    let Some(series) = df.column(col_name).ok() else {
        return String::new();
    };

    series
        .get(row_idx)
        .map(|v| {
            use polars::prelude::AnyValue;
            match v {
                AnyValue::Null => String::new(),
                AnyValue::String(s) => s.to_string(),
                AnyValue::Int64(i) => i.to_string(),
                AnyValue::Float64(f) => format!("{:.2}", f),
                AnyValue::Boolean(b) => if b { "Y" } else { "N" }.to_string(),
                other => format!("{}", other),
            }
        })
        .unwrap_or_default()
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
            update_current_page(state, domain_code, current_page.saturating_sub(1));
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
            update_current_page(state, domain_code, current_page + 1);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Row count
            let start_row = current_page * 50 + 1;
            let end_row = ((current_page + 1) * 50).min(total_rows);
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

/// Update the current page in state
fn update_current_page(state: &mut AppState, domain_code: &str, new_page: usize) {
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            domain.preview_state.current_page = new_page;
        }
    }
}

/// Rebuild preview data if needed (lazy initialization)
fn rebuild_preview_if_needed(state: &mut AppState, domain_code: &str) {
    // Check if we need to rebuild
    let needs_rebuild = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .map(|d| d.mapping_state.is_some() && d.preview_data.is_none())
        .unwrap_or(false);

    if !needs_rebuild {
        return;
    }

    // Extract the data we need for preview generation
    let preview_data = {
        let study = state.study.as_ref().unwrap();
        let domain = study.get_domain(domain_code).unwrap();
        let ms = domain.mapping_state.as_ref().unwrap();

        // Build mappings BTreeMap from accepted mappings
        let mappings: BTreeMap<String, String> = ms
            .all_accepted()
            .iter()
            .map(|(var, (col, _))| (var.clone(), col.clone()))
            .collect();

        // Get omitted variables
        let omitted: BTreeSet<String> = ms.all_omitted().clone();

        // Get the SDTM domain definition and source data
        let sdtm_domain = ms.domain().clone();
        let source_df = domain.source_data.clone();
        let study_id = study.study_id.clone();

        // Load CT registry
        let ct = load_default_ct_registry().ok();

        // Build the preview DataFrame
        build_preview_dataframe_with_omitted(
            &source_df,
            &mappings,
            &omitted,
            &sdtm_domain,
            &study_id,
            ct.as_ref(),
        )
    };

    // Store the result in state
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            match preview_data {
                Ok(df) => {
                    domain.preview_data = Some(df);
                    domain.preview_state.error = None;
                    domain.preview_state.current_page = 0; // Reset to first page
                }
                Err(e) => {
                    domain.preview_data = None;
                    domain.preview_state.error = Some(format!("{}", e));
                }
            }
        }
    }
}
