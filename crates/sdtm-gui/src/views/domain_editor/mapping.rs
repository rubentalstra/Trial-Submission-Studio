//! Mapping tab
//!
//! Interactive column-to-variable mapping with suggestions and CT display.

use crate::services::{MappingService, MappingState, VariableMappingStatus};
use crate::state::{AppState, DomainStatus};
use crate::theme::{colors, spacing};
use egui::{RichText, Ui};
use sdtm_standards::load_default_sdtm_ig_domains;
use std::collections::BTreeMap;

/// Render the mapping tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    let theme = colors(state.preferences.dark_mode);

    // Check domain status for loading state
    let (has_mapping_state, status) = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .map(|d| (d.mapping_state.is_some(), d.status))
        .unwrap_or((false, DomainStatus::NotStarted));

    // State machine for loading:
    // 1. NotStarted -> set to Loading, show spinner, request repaint
    // 2. Loading -> do initialization, will transition to MappingInProgress
    match (has_mapping_state, status) {
        // Not started: transition to loading and show spinner
        (false, DomainStatus::NotStarted) => {
            if let Some(study) = &mut state.study {
                if let Some(domain) = study.get_domain_mut(domain_code) {
                    domain.status = DomainStatus::Loading;
                }
            }
            // Show spinner immediately
            show_loading_indicator(ui, &theme);
            // Request repaint to process loading on next frame
            ui.ctx().request_repaint();
            return;
        }

        // Loading: show spinner and do initialization
        (false, DomainStatus::Loading) => {
            show_loading_indicator(ui, &theme);
            // Do the actual initialization (this takes time)
            initialize_mapping(state, domain_code);
            // Request repaint to show the result
            ui.ctx().request_repaint();
            return;
        }

        // No mapping state but not in loading flow - error
        (false, _) => {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} Failed to initialize mapping",
                        egui_phosphor::regular::WARNING
                    ))
                    .color(theme.error),
                );
            });
            return;
        }

        // Has mapping state - continue to render
        (true, _) => {}
    }

    // Master-detail layout using StripBuilder for proper sizing
    let available_height = ui.available_height();

    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(280.0)) // Left panel fixed width
        .size(egui_extras::Size::exact(1.0)) // Separator
        .size(egui_extras::Size::remainder()) // Right panel takes rest
        .horizontal(|mut strip| {
            // Left: Variable list
            strip.cell(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        show_variable_list(ui, state, domain_code, &theme);
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
                        show_variable_detail(ui, state, domain_code, &theme);
                    });
            });
        });
}

/// Auto-initialize mapping state for a domain
pub(super) fn initialize_mapping(state: &mut AppState, domain_code: &str) {
    // Get study info
    let (study_id, source_columns) = {
        if let Some(study) = &state.study {
            if let Some(domain) = study.get_domain(domain_code) {
                (study.study_id.clone(), domain.source_columns())
            } else {
                return;
            }
        } else {
            return;
        }
    };

    tracing::info!("Auto-initializing mapping for domain: {}", domain_code);

    // Load SDTM domain definition
    match load_default_sdtm_ig_domains() {
        Ok(domains) => {
            if let Some(sdtm_domain) = domains.into_iter().find(|d| d.code == domain_code) {
                tracing::info!(
                    "Found SDTM domain {} with {} variables",
                    domain_code,
                    sdtm_domain.variables.len()
                );

                // Get column hints from source data
                let hints = if let Some(study) = &state.study {
                    if let Some(domain) = study.get_domain(domain_code) {
                        MappingService::extract_column_hints(&domain.source_data)
                    } else {
                        BTreeMap::new()
                    }
                } else {
                    BTreeMap::new()
                };

                // Create mapping state
                let mapping_state = MappingService::create_mapping_state(
                    sdtm_domain,
                    &study_id,
                    &source_columns,
                    hints,
                );

                tracing::info!(
                    "Created mapping state with {} suggestions",
                    mapping_state.suggestions.len()
                );

                // Store it
                if let Some(study) = &mut state.study {
                    if let Some(domain) = study.get_domain_mut(domain_code) {
                        domain.mapping_state = Some(mapping_state);
                        domain.status = DomainStatus::MappingInProgress;
                    }
                }
            } else {
                tracing::warn!("SDTM domain {} not found in standards", domain_code);
            }
        }
        Err(e) => {
            tracing::error!("Failed to load SDTM domains: {}", e);
        }
    }
}

/// Show loading indicator with spinner
pub(super) fn show_loading_indicator(ui: &mut Ui, theme: &crate::theme::ThemeColors) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 3.0);
        ui.spinner();
        ui.add_space(spacing::MD);
        ui.label(
            RichText::new("Loading mapping configuration...")
                .size(16.0)
                .color(theme.text_secondary),
        );
        ui.add_space(spacing::SM);
        ui.label(
            RichText::new("Loading SDTM standards and controlled terminology")
                .color(theme.text_muted)
                .small(),
        );
    });
}

fn show_variable_list(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    theme: &crate::theme::ThemeColors,
) {
    // Collect data we need
    let (summary, filtered_vars, selected_idx, mut search_text, has_subjid_var) = {
        let Some(study) = &state.study else {
            ui.label("No study loaded");
            return;
        };
        let Some(domain) = study.get_domain(domain_code) else {
            ui.label("Domain not found");
            return;
        };
        let Some(ms) = &domain.mapping_state else {
            ui.label("No mapping state");
            return;
        };

        let summary = ms.summary();
        let has_subjid_var = ms.sdtm_domain.column_name("SUBJID").is_some();
        let filtered: Vec<_> = ms
            .filtered_variables()
            .iter()
            .map(|(idx, v)| {
                let status = ms.variable_status(&v.name);
                let core = v.core.clone();
                let role = v.role.clone();
                (*idx, v.name.clone(), core, role, status)
            })
            .collect();
        (
            summary,
            filtered,
            ms.selected_variable_idx,
            ms.search_filter.clone(),
            has_subjid_var,
        )
    };

    // Summary header
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{}/{}", summary.mapped, summary.total_variables)).strong());
        ui.label(RichText::new("mapped").color(theme.text_muted).small());

        if summary.suggested > 0 {
            ui.separator();
            ui.label(
                RichText::new(format!("{} suggested", summary.suggested))
                    .color(theme.warning)
                    .small(),
            );
        }
    });

    ui.add_space(spacing::SM);

    // Search box
    ui.horizontal(|ui| {
        ui.label(egui_phosphor::regular::MAGNIFYING_GLASS);
        let response = ui.text_edit_singleline(&mut search_text);
        if response.changed() {
            with_mapping_state_mut(state, domain_code, |ms| {
                ms.search_filter = search_text;
            });
        }
    });

    ui.add_space(spacing::SM);
    ui.separator();

    // Variable list using TableBuilder for proper alignment
    let mut new_selection: Option<usize> = None;
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(egui_extras::Column::remainder().at_least(100.0)) // Name
        .column(egui_extras::Column::exact(40.0)) // Core
        .column(egui_extras::Column::exact(40.0)) // Status
        .header(text_height + 4.0, |mut header| {
            header.col(|ui| {
                ui.label(RichText::new("Name").small().strong());
            });
            header.col(|ui| {
                ui.label(RichText::new("Core").small().strong());
            });
            header.col(|ui| {
                ui.label(RichText::new("St").small().strong());
            });
        })
        .body(|body| {
            body.rows(text_height + 8.0, filtered_vars.len(), |mut row| {
                let row_idx = row.index();
                let (idx, name, core, role, status) = &filtered_vars[row_idx];
                let is_selected = selected_idx == Some(*idx);

                // Check if this is an auto-generated variable using role from standards
                let is_auto = is_auto_generated_variable(name, role.as_deref())
                    || (has_subjid_var && name.eq_ignore_ascii_case("USUBJID"));

                let status_color = if is_auto {
                    theme.accent
                } else {
                    match status {
                        VariableMappingStatus::Accepted => theme.success,
                        VariableMappingStatus::Suggested => theme.warning,
                        VariableMappingStatus::Unmapped => theme.text_muted,
                    }
                };

                let core_text = match core.as_deref() {
                    Some("Req") => "Req",
                    Some("Exp") => "Exp",
                    Some("Perm") => "Perm",
                    _ => "—",
                };

                // Name column (clickable)
                row.col(|ui| {
                    let name_text = if is_selected {
                        RichText::new(name).strong()
                    } else {
                        RichText::new(name)
                    };

                    if ui.selectable_label(is_selected, name_text).clicked() {
                        new_selection = Some(*idx);
                    }
                });

                // Core column
                row.col(|ui| {
                    ui.label(RichText::new(core_text).color(theme.text_muted).small());
                });

                // Status column
                row.col(|ui| {
                    if is_auto {
                        ui.label(RichText::new("AUTO").color(status_color).small());
                    } else {
                        ui.label(RichText::new(status.icon()).color(status_color));
                    }
                });
            });
        });

    // Apply selection change
    if let Some(idx) = new_selection {
        with_mapping_state_mut(state, domain_code, |ms| {
            ms.selected_variable_idx = Some(idx);
        });
    }
}

fn show_variable_detail(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    theme: &crate::theme::ThemeColors,
) {
    // Collect all data we need first
    let detail_data = {
        let Some(study) = &state.study else {
            ui.label("No study");
            return;
        };
        let Some(domain) = study.get_domain(domain_code) else {
            ui.label("Domain not found");
            return;
        };
        let Some(ms) = &domain.mapping_state else {
            ui.label("No mapping state");
            return;
        };

        let Some(variable) = ms.selected_variable() else {
            // No variable selected - show help text
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("Select a variable from the list").color(theme.text_muted),
                );
            });
            return;
        };

        let var_name = variable.name.clone();
        let var_label = variable.label.clone();
        let var_core = variable.core.clone();
        let var_data_type = format!("{:?}", variable.data_type);
        let var_role = variable.role.clone();
        let var_codelist = variable.codelist_code.clone();
        let study_id = study.study_id.clone();
        let has_subjid_var = ms.sdtm_domain.column_name("SUBJID").is_some();
        let is_usubjid_derived = has_subjid_var && var_name.eq_ignore_ascii_case("USUBJID");
        let is_auto =
            is_auto_generated_variable(&var_name, var_role.as_deref()) || is_usubjid_derived;

        let suggestion = ms.get_suggestion_for(&var_name).cloned();
        let accepted = ms
            .get_accepted_for(&var_name)
            .map(|(c, f)| (c.to_string(), f));
        let status = ms.variable_status(&var_name);

        let (source_info, source_col_label, confidence, available_cols_sorted) = if is_auto {
            (None, None, None, Vec::new())
        } else {
            // Get available columns with confidence scores and labels for this variable
            // Tuple: (column_id, optional_label, confidence)
            let available_cols_with_info: Vec<(String, Option<String>, f32)> = ms
                .available_columns()
                .iter()
                .map(|col| {
                    // Calculate name similarity between column and variable
                    let similarity = calculate_name_similarity(col, &var_name);
                    // Get label from study metadata (Items.csv)
                    let label = study.get_column_label(col).map(String::from);
                    (col.to_string(), label, similarity)
                })
                .collect();

            // Sort by confidence (highest first), then by name
            let mut available_cols_sorted = available_cols_with_info;
            available_cols_sorted.sort_by(|a, b| {
                b.2.partial_cmp(&a.2)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            });

            // Get source column info if mapped/suggested
            let source_col_name = accepted
                .as_ref()
                .map(|(c, _)| c.clone())
                .or_else(|| suggestion.as_ref().map(|s| s.source_column.clone()));

            // Get source column label from study metadata (Items.csv)
            let source_col_label = source_col_name
                .as_ref()
                .and_then(|col| study.get_column_label(col).map(String::from));

            let source_info = source_col_name.as_ref().and_then(|col| {
                ms.column_hints.get(col).map(|hint| {
                    let samples = MappingService::get_sample_values(&domain.source_data, col, 5);
                    (
                        col.clone(),
                        hint.is_numeric,
                        hint.unique_ratio,
                        hint.null_ratio,
                        samples,
                    )
                })
            });

            let confidence = accepted
                .as_ref()
                .map(|(_, c)| *c)
                .or_else(|| suggestion.as_ref().map(|s| s.confidence));

            (
                source_info,
                source_col_label,
                confidence,
                available_cols_sorted,
            )
        };

        let (subjid_mapping, subjid_label, subjid_samples) = if is_usubjid_derived {
            let subjid_mapping = ms.get_accepted_for("SUBJID").map(|(c, _)| c.to_string());
            let subjid_label = subjid_mapping
                .as_ref()
                .and_then(|col| study.get_column_label(col).map(String::from));
            let subjid_samples = subjid_mapping
                .as_ref()
                .map(|col| MappingService::get_sample_values(&domain.source_data, col, 3))
                .unwrap_or_default();
            (subjid_mapping, subjid_label, subjid_samples)
        } else {
            (None, None, Vec::new())
        };

        // Get CT data from pre-loaded cache (loaded when domain opened)
        // Clone to avoid borrow issues with state mutations in render loop
        let ct_data: Vec<_> = var_codelist
            .as_ref()
            .map(|codes| ms.get_ct_for_variable(codes).into_iter().cloned().collect())
            .unwrap_or_default();

        (
            var_name,
            var_label,
            var_core,
            var_data_type,
            var_role,
            var_codelist,
            study_id,
            is_auto,
            is_usubjid_derived,
            subjid_mapping,
            subjid_label,
            subjid_samples,
            status,
            source_info,
            source_col_label,
            confidence,
            available_cols_sorted,
            ct_data,
        )
    };

    let (
        var_name,
        var_label,
        var_core,
        var_data_type,
        var_role,
        var_codelist,
        study_id,
        is_auto,
        is_usubjid_derived,
        subjid_mapping,
        subjid_label,
        subjid_samples,
        status,
        source_info,
        source_col_label,
        confidence,
        available_cols,
        ct_data,
    ) = detail_data;

    egui::ScrollArea::vertical().show(ui, |ui| {
        // SDTM Target section
        ui.label(
            RichText::new(format!("{} SDTM Target", egui_phosphor::regular::CROSSHAIR))
                .strong()
                .color(theme.text_muted),
        );
        ui.separator();
        ui.add_space(spacing::SM);

        ui.heading(&var_name);
        if let Some(label) = &var_label {
            ui.label(RichText::new(label).color(theme.text_secondary));
        }
        if is_usubjid_derived {
            ui.label(
                RichText::new("Derived as STUDYID-SUBJID from the mapped SUBJID column.")
                    .color(theme.text_secondary)
                    .small(),
            );
        }

        ui.add_space(spacing::MD);

        // Metadata table
        egui::Grid::new("var_metadata")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Core").color(theme.text_muted));
                ui.label(var_core.as_deref().unwrap_or("—"));
                ui.end_row();

                ui.label(RichText::new("Type").color(theme.text_muted));
                ui.label(&var_data_type);
                ui.end_row();

                ui.label(RichText::new("Role").color(theme.text_muted));
                ui.label(var_role.as_deref().unwrap_or("—"));
                ui.end_row();

                ui.label(RichText::new("Codelist").color(theme.text_muted));
                ui.label(var_codelist.as_deref().unwrap_or("—"));
                ui.end_row();
            });

        // Show codelist details using pre-fetched data (no loading during render)
        if !ct_data.is_empty() {
            ui.add_space(spacing::MD);

            ui.label(
                RichText::new(format!(
                    "{} Controlled Terminology",
                    egui_phosphor::regular::LIST_BULLETS
                ))
                .strong()
                .color(theme.text_muted),
            );
            ui.separator();
            ui.add_space(spacing::SM);

            for (cl_idx, cl_info) in ct_data.iter().enumerate() {
                if cl_idx > 0 {
                    ui.add_space(spacing::SM);
                    ui.separator();
                    ui.add_space(spacing::SM);
                }

                if cl_info.found {
                    // Show codelist code, name, and extensibility
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&cl_info.code).color(theme.text_muted).small());
                        ui.label(RichText::new(&cl_info.name).strong());
                        if cl_info.extensible {
                            ui.label(RichText::new("(Extensible)").color(theme.warning).small());
                        }
                    });

                    // Show valid values
                    if !cl_info.terms.is_empty() {
                        ui.add_space(spacing::SM);
                        ui.label(
                            RichText::new(format!("Valid values ({}):", cl_info.total_terms))
                                .color(theme.text_muted)
                                .small(),
                        );

                        for (value, def) in &cl_info.terms {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(value).strong().color(theme.accent));
                                if let Some(d) = def {
                                    ui.label(RichText::new(d).color(theme.text_secondary).small());
                                }
                            });
                        }

                        if cl_info.total_terms > cl_info.terms.len() {
                            ui.label(
                                RichText::new(format!(
                                    "... and {} more values",
                                    cl_info.total_terms - cl_info.terms.len()
                                ))
                                .color(theme.text_muted)
                                .small()
                                .italics(),
                            );
                        }
                    }
                } else {
                    ui.label(
                        RichText::new(format!("{} - not found in CT", cl_info.code))
                            .color(theme.warning)
                            .small(),
                    );
                }
            }
        }

        ui.add_space(spacing::LG);

        // Check if this is an auto-generated variable using role from standards
        if is_auto {
            // Auto-generated variable section
            ui.label(
                RichText::new(format!(
                    "{} Value Source",
                    egui_phosphor::regular::LIGHTNING
                ))
                .strong()
                .color(theme.text_muted),
            );
            ui.separator();
            ui.add_space(spacing::SM);

            ui.horizontal(|ui| {
                ui.label(RichText::new(egui_phosphor::regular::GEAR).color(theme.accent));
                ui.label(RichText::new("Auto-generated").strong().color(theme.accent));
            });

            ui.add_space(spacing::SM);

            // Explain what this variable contains
            let description = match var_name.as_str() {
                "DOMAIN" => "Set to the domain code (e.g., \"DM\", \"AE\")",
                "STUDYID" => "Populated from study configuration",
                "USUBJID" => "Derived from STUDYID and SUBJID",
                name if name.ends_with("SEQ") => "Assigned sequentially per subject (1, 2, 3...)",
                _ => "Generated by the system",
            };

            ui.label(
                RichText::new(description)
                    .color(theme.text_secondary)
                    .italics(),
            );

            if is_usubjid_derived {
                ui.add_space(spacing::MD);
                ui.label(RichText::new("Derivation").strong().color(theme.text_muted));
                ui.add_space(spacing::SM);

                egui::Grid::new("usubjid_derive")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Formula").color(theme.text_muted));
                        ui.label("STUDYID-SUBJID");
                        ui.end_row();

                        ui.label(RichText::new("Study ID").color(theme.text_muted));
                        ui.label(RichText::new(&study_id).color(theme.accent));
                        ui.end_row();
                    });

                if let Some(subjid_col) = &subjid_mapping {
                    ui.add_space(spacing::SM);
                    ui.label(
                        RichText::new("Source Mapping")
                            .strong()
                            .color(theme.text_muted),
                    );
                    ui.add_space(spacing::SM);

                    egui::Grid::new("usubjid_source")
                        .num_columns(2)
                        .spacing([20.0, 4.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("SUBJID").color(theme.text_muted));
                            ui.label(subjid_col);
                            ui.end_row();

                            ui.label(RichText::new("Label").color(theme.text_muted));
                            ui.label(subjid_label.as_deref().unwrap_or("—"));
                            ui.end_row();
                        });

                    if !subjid_samples.is_empty() {
                        ui.add_space(spacing::SM);
                        ui.label(
                            RichText::new("Sample Values")
                                .color(theme.text_muted)
                                .small(),
                        );
                        for val in &subjid_samples {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(val).code());
                                ui.label(RichText::new("→").color(theme.text_muted));
                                ui.label(
                                    RichText::new(format!("{}-{}", study_id, val))
                                        .code()
                                        .color(theme.accent),
                                );
                            });
                        }
                    }
                } else {
                    ui.add_space(spacing::SM);
                    ui.label(
                        RichText::new(format!(
                            "{} Map SUBJID to generate USUBJID",
                            egui_phosphor::regular::INFO
                        ))
                        .color(theme.warning)
                        .small(),
                    );
                }
            } else {
                ui.add_space(spacing::MD);
                ui.label(
                    RichText::new("This variable cannot be mapped manually.")
                        .color(theme.text_muted)
                        .small(),
                );
            }
        } else {
            // Source Column section for mappable variables
            ui.label(
                RichText::new(format!("{} Source Column", egui_phosphor::regular::TABLE))
                    .strong()
                    .color(theme.text_muted),
            );
            ui.separator();
            ui.add_space(spacing::SM);

            if let Some((col_name, is_numeric, unique_ratio, null_ratio, samples)) = source_info {
                // Show mapped/suggested column
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&col_name).strong());

                    if let Some(conf) = confidence {
                        let conf_color = if conf >= 0.95 {
                            theme.success
                        } else if conf >= 0.80 {
                            theme.warning
                        } else {
                            theme.text_muted
                        };
                        ui.label(RichText::new(format!("{:.0}%", conf * 100.0)).color(conf_color));
                    }
                });

                // Show source column label from metadata if available
                if let Some(label) = &source_col_label {
                    ui.label(RichText::new(label).color(theme.text_secondary).italics());
                }

                ui.add_space(spacing::SM);

                // Source column metadata
                egui::Grid::new("source_metadata")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Type").color(theme.text_muted));
                        ui.label(if is_numeric { "Numeric" } else { "Text" });
                        ui.end_row();

                        ui.label(RichText::new("Unique").color(theme.text_muted));
                        ui.label(format!("{:.0}%", unique_ratio * 100.0));
                        ui.end_row();

                        ui.label(RichText::new("Missing").color(theme.text_muted));
                        ui.label(format!("{:.1}%", null_ratio * 100.0));
                        ui.end_row();
                    });

                // Sample values
                if !samples.is_empty() {
                    ui.add_space(spacing::SM);
                    ui.label(RichText::new("Sample Values").color(theme.text_muted));
                    ui.label(
                        RichText::new(samples.join(" · "))
                            .color(theme.text_secondary)
                            .small(),
                    );
                }
            } else {
                ui.label(
                    RichText::new(format!("{} No mapping", egui_phosphor::regular::LINK_BREAK))
                        .color(theme.text_muted)
                        .italics(),
                );
            }
        }

        // Only show column selection and action buttons for non-auto variables
        if !is_auto {
            let is_subjid = var_name.eq_ignore_ascii_case("SUBJID");
            ui.add_space(spacing::MD);

            // Column selection dropdown with confidence display
            ui.label(RichText::new("Select column:").color(theme.text_muted));

            let mut selected_new_col: Option<String> = None;

            // Calculate popup width based on longest item
            let max_text_len = available_cols
                .iter()
                .map(|(col, label, _)| {
                    if let Some(lbl) = label {
                        format!("{} ({}) 100%", col, lbl).len()
                    } else {
                        format!("{} 100%", col).len()
                    }
                })
                .max()
                .unwrap_or(20);
            let popup_width = (max_text_len as f32 * 7.5).max(250.0).min(450.0);

            egui::ComboBox::from_id_salt("col_select")
                .selected_text("Choose a column...")
                .width(popup_width)
                .show_ui(ui, |ui| {
                    ui.set_min_width(popup_width);
                    for (col, label, conf) in &available_cols {
                        // Format: "ID (Label)" or just "ID" if no label
                        let display_text = if let Some(lbl) = label {
                            format!("{} ({})", col, lbl)
                        } else {
                            col.clone()
                        };

                        // Build the full display with confidence
                        let conf_text = if *conf > 0.01 {
                            format!(" — {:.0}%", conf * 100.0)
                        } else {
                            String::new()
                        };

                        let full_text = format!("{}{}", display_text, conf_text);

                        // Color based on confidence
                        let text_color = if *conf >= 0.95 {
                            theme.success
                        } else if *conf >= 0.70 {
                            theme.warning
                        } else {
                            theme.text_primary
                        };

                        if ui
                            .selectable_label(false, RichText::new(&full_text).color(text_color))
                            .clicked()
                        {
                            selected_new_col = Some(col.clone());
                        }
                    }
                });

            // Apply manual selection
            if let Some(col) = selected_new_col {
                with_mapping_state_mut(state, domain_code, |ms| {
                    ms.accept_manual(&var_name, &col);
                    if is_subjid {
                        sync_usubjid_from_subjid(ms);
                    }
                });
                // Invalidate cached validation/preview when mappings change
                if let Some(study) = &mut state.study {
                    if let Some(domain) = study.get_domain_mut(domain_code) {
                        domain.invalidate_mapping_dependents();
                    }
                }
            }

            ui.add_space(spacing::LG);

            // Action buttons
            ui.horizontal(|ui| match status {
                VariableMappingStatus::Suggested => {
                    if ui
                        .button(
                            RichText::new(format!("{} Accept", egui_phosphor::regular::CHECK))
                                .color(theme.success),
                        )
                        .clicked()
                    {
                        with_mapping_state_mut(state, domain_code, |ms| {
                            ms.accept_suggestion(&var_name);
                            if is_subjid {
                                sync_usubjid_from_subjid(ms);
                            }
                        });
                        // Invalidate cached validation/preview when mappings change
                        if let Some(study) = &mut state.study {
                            if let Some(domain) = study.get_domain_mut(domain_code) {
                                domain.invalidate_mapping_dependents();
                            }
                        }
                    }
                }
                VariableMappingStatus::Accepted => {
                    if ui
                        .button(format!("{} Clear", egui_phosphor::regular::X))
                        .clicked()
                    {
                        with_mapping_state_mut(state, domain_code, |ms| {
                            ms.clear_mapping(&var_name);
                            if is_subjid {
                                sync_usubjid_from_subjid(ms);
                            }
                        });
                        // Invalidate cached validation/preview when mappings change
                        if let Some(study) = &mut state.study {
                            if let Some(domain) = study.get_domain_mut(domain_code) {
                                domain.invalidate_mapping_dependents();
                            }
                        }
                    }
                }
                VariableMappingStatus::Unmapped => {
                    ui.label(
                        RichText::new(format!(
                            "{} Select a source column above",
                            egui_phosphor::regular::INFO
                        ))
                        .color(theme.text_muted)
                        .small(),
                    );
                }
            });
        }
    });
}

/// Check if a variable is auto-generated (not mapped from source)
///
/// Based on SDTMIG v3.4 variable definitions, certain Identifier role variables
/// are system-generated rather than mapped from source data. USUBJID derivation
/// is handled separately when SUBJID is available in the domain.
/// - STUDYID: From study-level configuration
/// - DOMAIN: Set to the two-character domain abbreviation
/// - --SEQ: Sequence numbers assigned per subject
///
/// This uses the Variable's role field from the SDTM standards.
fn is_auto_generated_variable(name: &str, role: Option<&str>) -> bool {
    // Only Identifier role variables can be auto-generated
    let is_identifier = role
        .map(|r| r.eq_ignore_ascii_case("Identifier"))
        .unwrap_or(false);

    if !is_identifier {
        return false;
    }

    // These specific Identifier variables are auto-generated per SDTMIG:
    // - STUDYID: Study identifier from study-level config
    // - DOMAIN: Domain abbreviation (e.g., "DM", "AE")
    // - --SEQ: Sequence number assigned per subject within domain
    matches!(name, "STUDYID" | "DOMAIN") || (name.ends_with("SEQ") && name.len() >= 4)
}

/// Calculate similarity between a source column name and an SDTM variable name.
/// Returns a score between 0.0 (no match) and 1.0 (exact match).
fn calculate_name_similarity(source: &str, target: &str) -> f32 {
    let source_upper = source.to_uppercase();
    let target_upper = target.to_uppercase();

    // Exact match (case-insensitive)
    if source_upper == target_upper {
        return 1.0;
    }

    // Source ends with target (e.g., "AETERM" ends with "TERM")
    if source_upper.ends_with(&target_upper) {
        return 0.95;
    }

    // Target ends with source (e.g., target "AESTDTC" ends with source "STDTC")
    if target_upper.ends_with(&source_upper) {
        return 0.90;
    }

    // Source contains target or vice versa
    if source_upper.contains(&target_upper) || target_upper.contains(&source_upper) {
        return 0.80;
    }

    // Calculate character overlap (Jaccard-like similarity)
    let source_chars: std::collections::HashSet<char> = source_upper.chars().collect();
    let target_chars: std::collections::HashSet<char> = target_upper.chars().collect();

    let intersection = source_chars.intersection(&target_chars).count();
    let union = source_chars.union(&target_chars).count();

    if union == 0 {
        return 0.0;
    }

    let jaccard = intersection as f32 / union as f32;

    // Scale to 0.0-0.6 range for partial matches
    jaccard * 0.6
}

fn with_mapping_state_mut<F>(state: &mut AppState, domain_code: &str, f: F)
where
    F: FnOnce(&mut MappingState),
{
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            if let Some(ms) = &mut domain.mapping_state {
                f(ms);
            }
        }
    }
}

fn sync_usubjid_from_subjid(ms: &mut MappingState) {
    let subjid = ms
        .get_accepted_for("SUBJID")
        .map(|(col, conf)| (col.to_string(), conf));

    if let Some((col, conf)) = subjid {
        ms.accepted.insert("USUBJID".to_string(), (col, conf));
    } else {
        ms.accepted.remove("USUBJID");
    }
}
