//! Domain editor view
//!
//! Main editing interface with tabs: Mapping, Transform, Validation, Preview, SUPP.

use crate::services::{CodelistDisplayInfo, MappingService, VariableMappingStatus};
use crate::state::{AppState, DomainStatus, EditorTab};
use crate::theme::{colors, spacing};
use egui::{RichText, Ui};
use sdtm_standards::load_default_sdtm_ig_domains;
use std::collections::BTreeMap;

/// Domain editor view
pub struct DomainEditorView;

impl DomainEditorView {
    /// Render the domain editor
    pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str, active_tab: EditorTab) {
        let theme = colors(state.preferences.dark_mode);

        // Top bar with domain info and back button
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                state.go_home();
            }

            ui.separator();

            ui.heading(domain_code);

            if let Some(study) = &state.study {
                if let Some(domain) = study.get_domain(domain_code) {
                    ui.label(
                        RichText::new(format!(
                            "{}  •  {} rows",
                            domain.source_file.display(),
                            domain.row_count()
                        ))
                        .color(theme.text_muted),
                    );
                }
            }
        });

        ui.add_space(spacing::SM);

        // Tab bar
        ui.horizontal(|ui| {
            for tab in EditorTab::all() {
                let is_active = *tab == active_tab;
                let text = if is_active {
                    RichText::new(tab.label()).strong().color(theme.accent)
                } else {
                    RichText::new(tab.label()).color(theme.text_secondary)
                };

                if ui.selectable_label(is_active, text).clicked() {
                    state.switch_tab(*tab);
                }
            }
        });

        ui.separator();
        ui.add_space(spacing::SM);

        // Tab content
        match active_tab {
            EditorTab::Mapping => Self::show_mapping_tab(ui, state, domain_code),
            EditorTab::Transform => Self::show_transform_tab(ui, state, domain_code),
            EditorTab::Validation => Self::show_validation_tab(ui, state, domain_code),
            EditorTab::Preview => Self::show_preview_tab(ui, state, domain_code),
            EditorTab::Supp => Self::show_supp_tab(ui, state, domain_code),
        }
    }

    fn show_mapping_tab(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
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
                Self::show_loading_indicator(ui, &theme);
                // Request repaint to process loading on next frame
                ui.ctx().request_repaint();
                return;
            }

            // Loading: show spinner and do initialization
            (false, DomainStatus::Loading) => {
                Self::show_loading_indicator(ui, &theme);
                // Do the actual initialization (this takes time)
                Self::initialize_mapping(state, domain_code);
                // Request repaint to show the result
                ui.ctx().request_repaint();
                return;
            }

            // No mapping state but not in loading flow - error
            (false, _) => {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("Failed to initialize mapping").color(theme.error));
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
            .size(egui_extras::Size::exact(1.0))   // Separator
            .size(egui_extras::Size::remainder())   // Right panel takes rest
            .horizontal(|mut strip| {
                // Left: Variable list
                strip.cell(|ui| {
                    egui::ScrollArea::vertical()
                        .max_height(available_height)
                        .show(ui, |ui| {
                            Self::show_variable_list(ui, state, domain_code, &theme);
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
                            Self::show_variable_detail(ui, state, domain_code, &theme);
                        });
                });
            });
    }

    /// Auto-initialize mapping state for a domain
    fn initialize_mapping(state: &mut AppState, domain_code: &str) {
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
    fn show_loading_indicator(ui: &mut Ui, theme: &crate::theme::ThemeColors) {
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
        let (summary, filtered_vars, selected_idx, mut search_text) = {
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
            (summary, filtered, ms.selected_variable_idx, ms.search_filter.clone())
        };

        // Summary header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{}/{}", summary.mapped, summary.total_variables))
                    .strong(),
            );
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
            ui.label("🔍");
            let response = ui.text_edit_singleline(&mut search_text);
            if response.changed() {
                if let Some(study) = &mut state.study {
                    if let Some(domain) = study.get_domain_mut(domain_code) {
                        if let Some(ms) = &mut domain.mapping_state {
                            ms.search_filter = search_text;
                        }
                    }
                }
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
                    let is_auto = Self::is_auto_generated_variable(name, role.as_deref());

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
            if let Some(study) = &mut state.study {
                if let Some(domain) = study.get_domain_mut(domain_code) {
                    if let Some(ms) = &mut domain.mapping_state {
                        ms.selected_variable_idx = Some(idx);
                    }
                }
            }
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
                        egui::RichText::new("Select a variable from the list")
                            .color(theme.text_muted),
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

            let suggestion = ms.get_suggestion_for(&var_name).cloned();
            let accepted = ms.get_accepted_for(&var_name).map(|(c, f)| (c.to_string(), f));
            let status = ms.variable_status(&var_name);

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

            // Get CT data from pre-loaded cache (loaded when domain opened)
            // Clone to avoid borrow issues with state mutations in render loop
            let ct_data: Vec<CodelistDisplayInfo> = var_codelist
                .as_ref()
                .map(|codes| {
                    ms.get_ct_for_variable(codes)
                        .into_iter()
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();

            (
                var_name,
                var_label,
                var_core,
                var_data_type,
                var_role,
                var_codelist,
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
            status,
            source_info,
            source_col_label,
            confidence,
            available_cols,
            ct_data,
        ) = detail_data;

        egui::ScrollArea::vertical().show(ui, |ui| {
            // SDTM Target section
            ui.label(RichText::new("SDTM Target").strong().color(theme.text_muted));
            ui.separator();
            ui.add_space(spacing::SM);

            ui.heading(&var_name);
            if let Some(label) = &var_label {
                ui.label(RichText::new(label).color(theme.text_secondary));
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
                    RichText::new("Controlled Terminology")
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
                                ui.label(
                                    RichText::new("(Extensible)")
                                        .color(theme.warning)
                                        .small(),
                                );
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
                                        ui.label(
                                            RichText::new(d).color(theme.text_secondary).small(),
                                        );
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
            let is_auto = Self::is_auto_generated_variable(&var_name, var_role.as_deref());

            if is_auto {
                // Auto-generated variable section
                ui.label(
                    RichText::new("Value Source")
                        .strong()
                        .color(theme.text_muted),
                );
                ui.separator();
                ui.add_space(spacing::SM);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚙").color(theme.accent));
                    ui.label(RichText::new("Auto-generated").strong().color(theme.accent));
                });

                ui.add_space(spacing::SM);

                // Explain what this variable contains
                let description = match var_name.as_str() {
                    "DOMAIN" => "Set to the domain code (e.g., \"DM\", \"AE\")",
                    "STUDYID" => "Populated from study configuration",
                    "USUBJID" => "Derived: STUDYID + \"-\" + subject identifier",
                    name if name.ends_with("SEQ") => "Assigned sequentially per subject (1, 2, 3...)",
                    _ => "Generated by the system",
                };

                ui.label(
                    RichText::new(description)
                        .color(theme.text_secondary)
                        .italics(),
                );

                ui.add_space(spacing::MD);
                ui.label(
                    RichText::new("This variable cannot be mapped manually.")
                        .color(theme.text_muted)
                        .small(),
                );
            } else {
                // Source Column section for mappable variables
                ui.label(
                    RichText::new("Source Column")
                        .strong()
                        .color(theme.text_muted),
                );
                ui.separator();
                ui.add_space(spacing::SM);

                if let Some((col_name, is_numeric, unique_ratio, null_ratio, samples)) = source_info
                {
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
                            ui.label(
                                RichText::new(format!("{:.0}%", conf * 100.0)).color(conf_color),
                            );
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
                        RichText::new("No mapping")
                            .color(theme.text_muted)
                            .italics(),
                    );
                }
            }

            // Only show column selection and action buttons for non-auto variables
            if !is_auto {
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
                        for (col, label, confidence) in &available_cols {
                            // Format: "ID (Label)" or just "ID" if no label
                            let display_text = if let Some(lbl) = label {
                                format!("{} ({})", col, lbl)
                            } else {
                                col.clone()
                            };

                            // Build the full display with confidence
                            let conf_text = if *confidence > 0.01 {
                                format!(" — {:.0}%", confidence * 100.0)
                            } else {
                                String::new()
                            };

                            let full_text = format!("{}{}", display_text, conf_text);

                            // Color based on confidence
                            let text_color = if *confidence >= 0.95 {
                                theme.success
                            } else if *confidence >= 0.70 {
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
                    if let Some(study) = &mut state.study {
                        if let Some(domain) = study.get_domain_mut(domain_code) {
                            if let Some(ms) = &mut domain.mapping_state {
                                ms.accept_manual(&var_name, &col);
                            }
                        }
                    }
                }

                ui.add_space(spacing::LG);

                // Action buttons
                ui.horizontal(|ui| {
                    match status {
                        VariableMappingStatus::Suggested => {
                            if ui
                                .button(RichText::new("Accept").color(theme.success))
                                .clicked()
                            {
                                if let Some(study) = &mut state.study {
                                    if let Some(domain) = study.get_domain_mut(domain_code) {
                                        if let Some(ms) = &mut domain.mapping_state {
                                            ms.accept_suggestion(&var_name);
                                        }
                                    }
                                }
                            }
                        }
                        VariableMappingStatus::Accepted => {
                            if ui.button("Clear").clicked() {
                                if let Some(study) = &mut state.study {
                                    if let Some(domain) = study.get_domain_mut(domain_code) {
                                        if let Some(ms) = &mut domain.mapping_state {
                                            ms.clear_mapping(&var_name);
                                        }
                                    }
                                }
                            }
                        }
                        VariableMappingStatus::Unmapped => {
                            ui.label(
                                RichText::new("Select a source column above")
                                    .color(theme.text_muted)
                                    .small(),
                            );
                        }
                    }
                });
            }
        });
    }

    fn show_transform_tab(ui: &mut Ui, _state: &mut AppState, domain_code: &str) {
        ui.label(format!("Transform tab for {} - TODO", domain_code));
        ui.label("Configure data transformations");

        ui.add_space(spacing::MD);
        ui.group(|ui| {
            ui.label("Transform Rules");
            ui.label("(Implementation pending)");
        });
    }

    fn show_validation_tab(ui: &mut Ui, _state: &mut AppState, domain_code: &str) {
        ui.label(format!("Validation tab for {} - TODO", domain_code));
        ui.label("View validation results and fix issues");

        ui.add_space(spacing::MD);
        ui.group(|ui| {
            ui.label("Validation Issues");
            ui.label("(Implementation pending)");
        });
    }

    fn show_preview_tab(ui: &mut Ui, _state: &mut AppState, domain_code: &str) {
        ui.label(format!("Preview tab for {} - TODO", domain_code));
        ui.label("Preview processed SDTM output");

        ui.add_space(spacing::MD);
        ui.group(|ui| {
            ui.label("Output Preview");
            ui.label("(Implementation pending)");
        });
    }

    fn show_supp_tab(ui: &mut Ui, _state: &mut AppState, domain_code: &str) {
        ui.label(format!("SUPP tab for {} - TODO", domain_code));
        ui.label("Configure Supplemental Qualifiers");

        ui.add_space(spacing::MD);
        ui.group(|ui| {
            ui.label("SUPPQUAL Configuration");
            ui.label("(Implementation pending)");
        });
    }

    /// Check if a variable is auto-generated (not mapped from source)
    ///
    /// Based on SDTMIG v3.4 variable definitions, certain Identifier role variables
    /// are system-generated rather than mapped from source data:
    /// - STUDYID: From study-level configuration
    /// - DOMAIN: Set to the two-character domain abbreviation
    /// - USUBJID: Derived as STUDYID + subject identifier
    /// - --SEQ: Sequence numbers assigned per subject
    ///
    /// This uses the Variable's role field from the SDTM standards.
    fn is_auto_generated_variable(name: &str, role: Option<&str>) -> bool {
        // Only Identifier role variables can be auto-generated
        let is_identifier = role.map(|r| r.eq_ignore_ascii_case("Identifier")).unwrap_or(false);

        if !is_identifier {
            return false;
        }

        // These specific Identifier variables are auto-generated per SDTMIG:
        // - STUDYID: Study identifier from study-level config
        // - DOMAIN: Domain abbreviation (e.g., "DM", "AE")
        // - USUBJID: Unique subject ID (derived from STUDYID + subject ID)
        // - --SEQ: Sequence number assigned per subject within domain
        matches!(name, "STUDYID" | "DOMAIN" | "USUBJID")
            || (name.ends_with("SEQ") && name.len() >= 4)
    }
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
