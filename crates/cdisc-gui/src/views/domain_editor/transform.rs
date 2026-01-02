//! Transform tab
//!
//! Shows source data being transformed to SDTM-compliant data.
//! Uses a 2-column layout: transformation list on left, details on right.
//! Preview is computed lazily when this tab is opened.

use crate::services::{PreviewState, ensure_preview};
use crate::state::AppState;
use crate::theme::spacing;
use cdisc_common::any_to_string;
use egui::{Color32, RichText, Ui};
use polars::prelude::*;

/// Render the transform tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Step 1: Ensure preview is ready
    match ensure_preview(state, domain_code) {
        PreviewState::Rebuilding => {
            show_spinner(ui, "Building preview...");
            ui.ctx().request_repaint();
            return;
        }
        PreviewState::NotConfigured => {
            show_not_configured(ui);
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
    if state.domain(domain_code).is_none() {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Domain not accessible").color(ui.visuals().error_fg_color));
        });
        return;
    }

    // Step 3: Master-detail layout
    let available_height = ui.available_height();

    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(300.0)) // Left panel fixed width
        .size(egui_extras::Size::exact(1.0)) // Separator
        .size(egui_extras::Size::remainder()) // Right panel takes rest
        .horizontal(|mut strip| {
            // Left: Transformation list
            strip.cell(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        show_transform_list(ui, state, domain_code);
                    });
            });

            // Separator
            strip.cell(|ui| {
                ui.separator();
            });

            // Right: Detail panel
            strip.cell(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        show_transform_detail(ui, state, domain_code);
                    });
            });
        });
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

/// Show "not configured" message
fn show_not_configured(ui: &mut Ui) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(egui_phosphor::regular::INFO)
                    .size(24.0)
                    .weak(),
            );
            ui.add_space(spacing::SM);
            ui.label(RichText::new("No transformations yet").weak());
            ui.add_space(spacing::XS);
            ui.label(
                RichText::new(
                    "Map variables in the Mapping tab to see their transformations here.",
                )
                .weak()
                .small(),
            );
        });
    });
}

/// Show error state
fn show_error(ui: &mut Ui, error: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(spacing::XL);
        ui.label(
            RichText::new(format!(
                "{} Transform Error",
                egui_phosphor::regular::WARNING
            ))
            .color(ui.visuals().error_fg_color)
            .size(18.0),
        );
        ui.add_space(spacing::MD);
        ui.label(RichText::new(error).weak());
    });
}

/// Data for a single transformation row
struct TransformRow {
    target_variable: String,
    source_column: Option<String>,
    category: String,
}

/// Show the transformation list (left panel)
fn show_transform_list(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Get selected index from UI state
    let selected_idx = state
        .ui
        .get_domain_editor(domain_code)
        .and_then(|ui| ui.transform.selected_idx);

    // Collect transformation info from mappings (only accepted + auto-generated)
    let rows: Vec<TransformRow> = {
        let Some(study) = state.study() else { return };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };

        let mapping = &domain.mapping;
        let sdtm_domain = mapping.domain();

        // Get all accepted mappings and auto-generated set
        let accepted = mapping.all_accepted();
        let auto_generated = mapping.all_auto_generated();

        // Build rows only for variables that are accepted OR auto-generated
        sdtm_domain
            .variables
            .iter()
            .filter_map(|var| {
                let source_column = accepted.get(&var.name).map(|(col, _)| col.clone());
                let is_auto = auto_generated.contains(&var.name);

                // Only include if accepted (has source) or auto-generated
                if source_column.is_none() && !is_auto {
                    return None;
                }

                let category = infer_category(
                    &var.name,
                    source_column.is_some(),
                    var.codelist_code.is_some(),
                );
                Some(TransformRow {
                    target_variable: var.name.clone(),
                    source_column,
                    category,
                })
            })
            .collect()
    };

    if rows.is_empty() {
        show_not_configured(ui);
        return;
    }

    // Summary counts
    let mapped_count = rows.iter().filter(|r| r.source_column.is_some()).count();
    let auto_count = rows.iter().filter(|r| r.source_column.is_none()).count();
    let total = rows.len();

    // Header
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{}", total)).strong());
        ui.label(RichText::new("variables").weak().small());
        ui.separator();
        ui.label(
            RichText::new(format!("{} mapped", mapped_count))
                .small()
                .color(Color32::from_rgb(100, 180, 100)),
        );
        if auto_count > 0 {
            ui.label(RichText::new(format!("{} auto", auto_count)).small().weak());
        }
    });

    ui.add_space(spacing::SM);
    ui.separator();

    // Transform list using TableBuilder
    let mut new_selection: Option<usize> = None;
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(egui_extras::Column::exact(24.0)) // Icon
        .column(egui_extras::Column::remainder().at_least(80.0)) // Target var
        .column(egui_extras::Column::exact(60.0)) // Category
        .header(text_height + 4.0, |mut header| {
            header.col(|_ui| {});
            header.col(|ui| {
                ui.label(RichText::new("Variable").small().strong());
            });
            header.col(|ui| {
                ui.label(RichText::new("Type").small().strong());
            });
        })
        .body(|body| {
            body.rows(text_height + 8.0, rows.len(), |mut row| {
                let row_idx = row.index();
                let transform_row = &rows[row_idx];
                let is_selected = selected_idx == Some(row_idx);
                let is_auto = transform_row.source_column.is_none();

                // Icon column
                row.col(|ui| {
                    let (icon, color) =
                        get_category_icon_color(&transform_row.category, is_auto, ui);
                    ui.label(RichText::new(icon).color(color));
                });

                // Target variable column (clickable)
                row.col(|ui| {
                    let mut label_text = RichText::new(&transform_row.target_variable).monospace();
                    if is_selected {
                        label_text = label_text.strong();
                    }
                    if is_auto {
                        label_text = label_text.weak();
                    }

                    let response = ui.selectable_label(is_selected, label_text);
                    if response.clicked() {
                        new_selection = Some(row_idx);
                    }

                    if let Some(src) = &transform_row.source_column {
                        response.on_hover_text(format!("← {}", src));
                    }
                });

                // Category column
                row.col(|ui| {
                    let cat_short = match transform_row.category.as_str() {
                        "CT Normalization" => "CT",
                        "Copy" => "Copy",
                        "Auto" => "Auto",
                        _ => "—",
                    };
                    ui.label(RichText::new(cat_short).weak().small());
                });
            });
        });

    // Apply selection change
    if let Some(idx) = new_selection {
        state
            .ui
            .domain_editor(domain_code)
            .transform
            .select(Some(idx));
    }
}

/// Show the transformation detail (right panel)
fn show_transform_detail(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Get selected index
    let selected_idx = state
        .ui
        .get_domain_editor(domain_code)
        .and_then(|ui| ui.transform.selected_idx);

    let Some(idx) = selected_idx else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Select a variable from the list").weak());
        });
        return;
    };

    // Extract detail data
    let detail = {
        let Some(study) = state.study() else { return };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };

        let mapping = &domain.mapping;
        let sdtm_domain = mapping.domain();
        let accepted = mapping.all_accepted();
        let auto_generated = mapping.all_auto_generated();

        // Build filtered list (same as left panel) to get correct variable by index
        let filtered_vars: Vec<_> = sdtm_domain
            .variables
            .iter()
            .filter(|var| accepted.contains_key(&var.name) || auto_generated.contains(&var.name))
            .collect();

        let Some(var) = filtered_vars.get(idx) else {
            ui.label(RichText::new("Variable not found").weak());
            return;
        };

        let source_column = accepted.get(&var.name).map(|(col, _)| col.clone());
        let is_auto = auto_generated.contains(&var.name);
        let source_df = &domain.source.data;
        let preview_df = domain.derived.preview.as_ref();

        // Get source samples
        let source_samples: Vec<String> = source_column
            .as_ref()
            .and_then(|col| source_df.column(col).ok())
            .map(|col| get_unique_samples(col.as_materialized_series(), 10))
            .unwrap_or_default();

        // Get transformed samples from preview
        let transformed_samples: Vec<String> = preview_df
            .and_then(|df| df.column(&var.name).ok())
            .map(|col| get_unique_samples(col.as_materialized_series(), 10))
            .unwrap_or_default();

        let category = if is_auto {
            "Auto".to_string()
        } else {
            infer_category(
                &var.name,
                source_column.is_some(),
                var.codelist_code.is_some(),
            )
        };

        TransformDetail {
            target_variable: var.name.clone(),
            target_label: var.label.clone(),
            source_column,
            category,
            codelist_code: var.codelist_code.clone(),
            source_samples,
            transformed_samples,
            is_auto,
        }
    };

    // Render detail view
    show_detail_content(ui, &detail);
}

/// Data for displaying transformation details
struct TransformDetail {
    target_variable: String,
    target_label: Option<String>,
    source_column: Option<String>,
    category: String,
    codelist_code: Option<String>,
    source_samples: Vec<String>,
    transformed_samples: Vec<String>,
    is_auto: bool,
}

fn show_detail_content(ui: &mut Ui, detail: &TransformDetail) {
    let is_auto = detail.is_auto;

    // Header with variable name and label
    ui.heading(&detail.target_variable);
    if let Some(label) = &detail.target_label {
        ui.label(RichText::new(label).weak().italics());
    }

    ui.add_space(spacing::MD);

    // Metadata grid
    egui::Grid::new("transform_metadata")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Transform").weak());
            let (icon, color) = get_category_icon_color(&detail.category, is_auto, ui);
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).color(color));
                ui.label(&detail.category);
            });
            ui.end_row();

            if let Some(src) = &detail.source_column {
                ui.label(RichText::new("Source").weak());
                ui.label(RichText::new(src).monospace());
                ui.end_row();
            }

            if let Some(cl) = &detail.codelist_code {
                ui.label(RichText::new("Codelist").weak());
                ui.label(RichText::new(cl).monospace().small());
                ui.end_row();
            }
        });

    ui.add_space(spacing::LG);

    // Auto-generated explanation
    if is_auto {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(egui_phosphor::regular::MAGIC_WAND)
                    .color(Color32::from_rgb(100, 180, 100)),
            );
            ui.label(RichText::new("Auto-generated value").weak().italics());
        });
        ui.add_space(spacing::SM);
        let explanation = match detail.target_variable.as_str() {
            "STUDYID" => "Populated from study configuration",
            "DOMAIN" => "Set to the domain code",
            "USUBJID" => "Derived from STUDYID + SUBJID",
            name if name.ends_with("SEQ") => "Assigned sequentially per subject",
            _ => "Generated by the system",
        };
        ui.label(RichText::new(explanation).weak().small());
        return;
    }

    // Before → After section (only for mapped variables)
    ui.label(
        RichText::new(format!(
            "{} Before → After",
            egui_phosphor::regular::ARROWS_LEFT_RIGHT
        ))
        .strong()
        .weak(),
    );
    ui.separator();
    ui.add_space(spacing::SM);

    if detail.source_samples.is_empty() {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(egui_phosphor::regular::WARNING).color(ui.visuals().warn_fg_color),
            );
            ui.label(
                RichText::new(format!(
                    "Column '{}' not found in source data",
                    detail.source_column.as_ref().unwrap_or(&"?".to_string())
                ))
                .weak()
                .italics(),
            );
        });
        return;
    }

    let has_change = detail.source_samples != detail.transformed_samples;
    if has_change {
        ui.label(
            RichText::new(format!(
                "{} Values transformed",
                egui_phosphor::regular::CHECK
            ))
            .color(Color32::from_rgb(100, 180, 100)),
        );
    } else {
        ui.label(
            RichText::new(format!(
                "{} Values unchanged",
                egui_phosphor::regular::EQUALS
            ))
            .weak(),
        );
    }

    ui.add_space(spacing::SM);

    // Show side-by-side comparison (max 10 examples, no truncation)
    let pairs: Vec<_> = detail
        .source_samples
        .iter()
        .zip(detail.transformed_samples.iter())
        .take(10)
        .collect();

    for (src, dst) in pairs {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(src).monospace().weak());

            if src != dst {
                ui.label(
                    RichText::new(format!(" {} ", egui_phosphor::regular::ARROW_RIGHT))
                        .color(Color32::from_rgb(100, 180, 100)),
                );
                ui.label(
                    RichText::new(dst)
                        .monospace()
                        .color(Color32::from_rgb(100, 180, 100)),
                );
            } else {
                ui.label(RichText::new(" = ").weak());
                ui.label(RichText::new(dst).monospace().weak());
            }
        });
    }

    // Show note if there are more examples
    if detail.source_samples.len() > 10 {
        ui.add_space(spacing::SM);
        ui.label(
            RichText::new(format!(
                "... and {} more values",
                detail.source_samples.len() - 10
            ))
            .weak()
            .small()
            .italics(),
        );
    }
}

/// Get unique sample values from a series
fn get_unique_samples(series: &Series, limit: usize) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut samples = Vec::new();

    for i in 0..series.len().min(100) {
        if let Ok(val) = series.get(i) {
            let s = any_to_string(val);
            if !s.is_empty() && s != "null" && seen.insert(s.clone()) {
                samples.push(s);
                if samples.len() >= limit {
                    break;
                }
            }
        }
    }
    samples
}

/// Infer the transformation category
fn infer_category(_var_name: &str, has_source: bool, has_codelist: bool) -> String {
    if !has_source {
        return "Auto".to_string();
    }
    if has_codelist {
        return "CT Normalization".to_string();
    }
    "Copy".to_string()
}

/// Get icon and color for category
fn get_category_icon_color(category: &str, is_auto: bool, ui: &Ui) -> (&'static str, Color32) {
    if is_auto {
        (
            egui_phosphor::regular::MAGIC_WAND,
            Color32::from_rgb(100, 180, 100),
        )
    } else {
        match category {
            "CT Normalization" => (
                egui_phosphor::regular::LIST_CHECKS,
                Color32::from_rgb(100, 150, 220),
            ),
            "Copy" => (egui_phosphor::regular::COPY, ui.visuals().weak_text_color()),
            _ => (egui_phosphor::regular::GEAR, ui.visuals().text_color()),
        }
    }
}
