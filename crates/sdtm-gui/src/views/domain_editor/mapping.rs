//! Mapping tab
//!
//! Interactive column-to-variable mapping with suggestions and CT display.

use crate::services::{MappingService, MappingState, VariableStatus, VariableStatusIcon};
use crate::state::{AppState, DomainStatus};
use crate::theme::spacing;
use egui::{Color32, RichText, Ui};
use sdtm_model::{CoreDesignation, VariableRole};
use sdtm_standards::load_default_sdtm_ig_domains;
use std::collections::BTreeMap;

/// Render the mapping tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
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
            show_loading_indicator(ui);
            // Request repaint to process loading on next frame
            ui.ctx().request_repaint();
            return;
        }

        // Loading: show spinner and do initialization
        (false, DomainStatus::Loading) => {
            show_loading_indicator(ui);
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
                    .color(ui.visuals().error_fg_color),
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
                        show_variable_list(ui, state, domain_code);
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
                        show_variable_detail(ui, state, domain_code);
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
            if let Some(sdtm_domain) = domains.into_iter().find(|d| d.name == domain_code) {
                tracing::info!(
                    "Found SDTM domain {} with {} variables",
                    domain_code,
                    sdtm_domain.variables.len()
                );

                // Extract domain label before moving sdtm_domain
                let domain_label = sdtm_domain.label.clone();

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
                let mut mapping_state = MappingService::create_mapping_state(
                    sdtm_domain.clone(),
                    &study_id,
                    &source_columns,
                    hints,
                );

                // Auto-mark auto-generated variables as accepted
                // These variables are populated automatically by the transform system
                // and don't need user decisions
                let auto_generated_count =
                    auto_accept_generated_variables(&mut mapping_state, &sdtm_domain);
                tracing::info!(
                    "Created mapping state with {} suggestions, {} auto-generated",
                    mapping_state.suggestions_count(),
                    auto_generated_count
                );

                // Store it
                if let Some(study) = &mut state.study {
                    if let Some(domain) = study.get_domain_mut(domain_code) {
                        domain.domain_label = domain_label;
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
pub(super) fn show_loading_indicator(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 3.0);
        ui.spinner();
        ui.add_space(spacing::MD);
        ui.label(RichText::new("Loading mapping configuration...").size(16.0));
        ui.add_space(spacing::SM);
        ui.label(
            RichText::new("Loading SDTM standards and controlled terminology")
                .weak()
                .small(),
        );
    });
}

fn show_variable_list(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
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
        let has_subjid_var = ms.domain().column_name("SUBJID").is_some();
        let filtered: Vec<_> = ms
            .filtered_variables()
            .iter()
            .map(|(idx, v)| {
                let status = ms.status(&v.name);
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
        ui.label(RichText::new("mapped").weak().small());

        if summary.suggested > 0 {
            ui.separator();
            ui.label(
                RichText::new(format!("{} suggested", summary.suggested))
                    .color(ui.visuals().warn_fg_color)
                    .small(),
            );
        }

        if summary.not_collected > 0 {
            ui.separator();
            ui.label(
                RichText::new(format!("{} not collected", summary.not_collected))
                    .weak()
                    .small(),
            );
        }

        if summary.omitted > 0 {
            ui.separator();
            ui.label(
                RichText::new(format!("{} omitted", summary.omitted))
                    .weak()
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
                let is_auto = is_auto_generated_variable(name, *role)
                    || (has_subjid_var && name.eq_ignore_ascii_case("USUBJID"));

                let core_text = match core {
                    Some(CoreDesignation::Required) => "Req",
                    Some(CoreDesignation::Expected) => "Exp",
                    Some(CoreDesignation::Permissible) => "Perm",
                    None => "—",
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
                    ui.label(RichText::new(core_text).weak().small());
                });

                // Status column - compute color inside the col callback
                row.col(|ui| {
                    let status_color = if is_auto {
                        ui.visuals().hyperlink_color
                    } else {
                        match status {
                            VariableStatus::Accepted => Color32::GREEN,
                            VariableStatus::Suggested => ui.visuals().warn_fg_color,
                            VariableStatus::NotCollected => ui.visuals().weak_text_color(),
                            VariableStatus::Omitted => ui.visuals().weak_text_color(),
                            VariableStatus::Unmapped => ui.visuals().weak_text_color(),
                        }
                    };
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

fn show_variable_detail(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Extract all needed data before entering closures to avoid borrow conflicts
    let data = {
        let Some(study) = &state.study else { return };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };
        let Some(ms) = &domain.mapping_state else {
            return;
        };
        let Some(variable) = ms.selected_variable() else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Select a variable from the list").weak());
            });
            return;
        };

        let var_name = variable.name.clone();
        let has_subjid = ms.domain().column_name("SUBJID").is_some();
        let is_auto = is_auto_generated(&var_name, variable.role, has_subjid);
        let is_usubjid_derived = has_subjid && var_name.eq_ignore_ascii_case("USUBJID");
        let status = ms.status(&var_name);
        let study_id = study.study_id.clone();

        // Extract data for display
        let variable_label: Option<String> = variable.label.clone();
        let variable_core: Option<CoreDesignation> = variable.core;
        let variable_data_type: String = format!("{:?}", variable.data_type);
        let variable_role: Option<String> = variable.role.map(|r| r.as_str().to_string());
        let codelist_code: Option<String> = variable.codelist_code.clone();

        // CT info - clone to owned values
        let ct_info: Vec<crate::services::CodelistDisplayInfo> = codelist_code
            .as_ref()
            .map(|c| ms.get_ct_for_variable(c).into_iter().cloned().collect())
            .unwrap_or_default();

        // Source mapping data
        let current_mapping = ms
            .accepted(&var_name)
            .or(ms.suggestion(&var_name))
            .map(|(col, conf)| (col.to_string(), conf));
        let column_label = current_mapping
            .as_ref()
            .and_then(|(col, _)| study.get_column_label(col).map(|s| s.to_string()));
        let sample_values: Vec<String> = current_mapping
            .as_ref()
            .map(|(col, _)| MappingService::get_sample_values(&domain.source_data, col, 5))
            .unwrap_or_default();

        // USUBJID derivation info
        let subjid_info = if is_usubjid_derived {
            ms.accepted("SUBJID").map(|(col, _)| {
                let samples = MappingService::get_sample_values(&domain.source_data, col, 3);
                (col.to_string(), samples)
            })
        } else {
            None
        };

        let not_collected_reason = ms.not_collected_reason(&var_name).map(|s| s.to_string());

        Some((
            var_name,
            is_auto,
            is_usubjid_derived,
            status,
            study_id,
            variable_label,
            variable_core,
            variable_data_type,
            variable_role,
            codelist_code,
            ct_info,
            current_mapping,
            column_label,
            sample_values,
            subjid_info,
            not_collected_reason,
        ))
    };

    let Some((
        var_name,
        is_auto,
        is_usubjid_derived,
        status,
        study_id,
        variable_label,
        variable_core,
        variable_data_type,
        variable_role,
        codelist_code,
        ct_info,
        current_mapping,
        column_label,
        sample_values,
        subjid_info,
        not_collected_reason,
    )) = data
    else {
        return;
    };

    egui::ScrollArea::vertical().show(ui, |ui| {
        // SDTM Target section
        ui.label(
            RichText::new(format!("{} SDTM Target", egui_phosphor::regular::CROSSHAIR))
                .strong()
                .weak(),
        );
        ui.separator();
        ui.add_space(spacing::SM);

        ui.heading(&var_name);
        if let Some(label) = &variable_label {
            ui.label(RichText::new(label).weak());
        }

        ui.add_space(spacing::MD);

        // Metadata grid
        egui::Grid::new("var_metadata")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Core").weak());
                ui.label(
                    variable_core
                        .map(|c: CoreDesignation| c.as_code())
                        .unwrap_or("—"),
                );
                ui.end_row();

                ui.label(RichText::new("Type").weak());
                ui.label(&variable_data_type);
                ui.end_row();

                ui.label(RichText::new("Role").weak());
                let role_str: &str = variable_role.as_deref().unwrap_or("—");
                ui.label(role_str);
                ui.end_row();

                if let Some(cl) = &codelist_code {
                    ui.label(RichText::new("Codelist").weak());
                    ui.label(cl);
                    ui.end_row();
                }
            });

        // Show CT values if available
        let ct_info: &Vec<crate::services::CodelistDisplayInfo> = &ct_info;
        if !ct_info.is_empty() {
            ui.add_space(spacing::MD);
            ui.label(
                RichText::new(format!(
                    "{} Controlled Terminology",
                    egui_phosphor::regular::LIST_BULLETS
                ))
                .strong()
                .weak(),
            );
            ui.separator();
            ui.add_space(spacing::SM);

            for info in ct_info {
                if info.found {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&info.name).strong());
                        if info.extensible {
                            ui.label(
                                RichText::new("(Extensible)")
                                    .color(ui.visuals().warn_fg_color)
                                    .small(),
                            );
                        }
                    });
                    ui.label(
                        RichText::new(format!("{} terms", info.total_terms))
                            .weak()
                            .small(),
                    );
                    for (val, def) in info.terms.iter().take(5) {
                        ui.label(
                            RichText::new(val)
                                .strong()
                                .color(ui.visuals().hyperlink_color),
                        );
                        if let Some(d) = def {
                            ui.label(RichText::new(d).weak().small());
                        }
                    }
                    if info.total_terms > 5 {
                        ui.label(
                            RichText::new(format!("... and {} more", info.total_terms - 5))
                                .weak()
                                .small()
                                .italics(),
                        );
                    }
                }
            }
        }

        ui.add_space(spacing::LG);

        // Source section
        if is_auto {
            show_auto_generated_info(ui, &var_name, &study_id, is_usubjid_derived, &subjid_info);
        } else {
            show_source_mapping_inline(
                ui,
                state,
                domain_code,
                &var_name,
                status,
                variable_core,
                &current_mapping,
                &column_label,
                &sample_values,
                &not_collected_reason,
            );
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
fn is_auto_generated_variable(name: &str, role: Option<VariableRole>) -> bool {
    // Only Identifier role variables can be auto-generated
    if role != Some(VariableRole::Identifier) {
        return false;
    }

    // These specific Identifier variables are auto-generated per SDTMIG:
    // - STUDYID: Study identifier from study-level config
    // - DOMAIN: Domain abbreviation (e.g., "DM", "AE")
    // - --SEQ: Sequence number assigned per subject within domain
    matches!(name, "STUDYID" | "DOMAIN") || (name.ends_with("SEQ") && name.len() >= 4)
}

/// Auto-accept auto-generated variables during mapping initialization.
///
/// These variables are populated automatically by the transform system:
/// - STUDYID: From study configuration
/// - DOMAIN: Set to domain code  
/// - --SEQ: Sequence number per subject
///
/// By marking them as Accepted upfront, we don't need to filter them
/// in every tab that checks mapping status.
fn auto_accept_generated_variables(
    mapping_state: &mut MappingState,
    domain: &sdtm_model::Domain,
) -> usize {
    let mut count = 0;
    for var in &domain.variables {
        if is_auto_generated_variable(&var.name, var.role) {
            // Use a special marker value for auto-generated variables
            // The column name "__AUTO__" indicates this is auto-generated
            let _ = mapping_state.accept_manual(&var.name, "__AUTO__");
            count += 1;
        }
    }
    count
}

/// Mutate mapping state and invalidate dependent state (validation, transform, preview)
fn with_mapping_state_mut<F>(state: &mut AppState, domain_code: &str, f: F)
where
    F: FnOnce(&mut MappingState),
{
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            if let Some(ms) = &mut domain.mapping_state {
                f(ms);
            }
            // Invalidate cached state that depends on mappings
            domain.validation = None;
            domain.validation_selected_idx = None;
            domain.transform_state = None;
            domain.preview_data = None;
            domain.supp_state = None;
        }
    }
}

fn sync_usubjid_from_subjid(ms: &mut MappingState) {
    let subjid_col = ms.accepted("SUBJID").map(|(col, _)| col.to_string());

    if let Some(col) = subjid_col {
        // Use accept_manual to set USUBJID to the same column as SUBJID
        // This is a special derived case, so we ignore the "already used" error
        let _ = ms.accept_manual("USUBJID", &col);
    } else {
        ms.clear("USUBJID");
    }
}

/// Check if a variable is auto-generated based on role and name
fn is_auto_generated(name: &str, role: Option<VariableRole>, has_subjid: bool) -> bool {
    // USUBJID is derived from SUBJID when SUBJID is available
    if has_subjid && name.eq_ignore_ascii_case("USUBJID") {
        return true;
    }

    // Only Identifier role variables can be auto-generated
    if role != Some(VariableRole::Identifier) {
        return false;
    }

    // These specific Identifier variables are auto-generated per SDTMIG
    matches!(name, "STUDYID" | "DOMAIN") || (name.ends_with("SEQ") && name.len() >= 4)
}

fn show_auto_generated_info(
    ui: &mut Ui,
    var_name: &str,
    study_id: &str,
    is_usubjid: bool,
    subjid_info: &Option<(String, Vec<String>)>,
) {
    ui.label(
        RichText::new(format!(
            "{} Value Source",
            egui_phosphor::regular::LIGHTNING
        ))
        .strong()
        .weak(),
    );
    ui.separator();
    ui.add_space(spacing::SM);

    ui.horizontal(|ui| {
        ui.label(RichText::new(egui_phosphor::regular::GEAR).color(ui.visuals().hyperlink_color));
        ui.label(
            RichText::new("Auto-generated")
                .strong()
                .color(ui.visuals().hyperlink_color),
        );
    });

    ui.add_space(spacing::SM);

    let desc = match var_name {
        "DOMAIN" => "Set to the domain code (e.g., \"DM\", \"AE\")",
        "STUDYID" => "Populated from study configuration",
        "USUBJID" => "Derived from STUDYID and SUBJID",
        name if name.ends_with("SEQ") => "Assigned sequentially per subject",
        _ => "Generated by the system",
    };
    ui.label(RichText::new(desc).weak().italics());

    if is_usubjid {
        ui.add_space(spacing::MD);
        egui::Grid::new("usubjid_derive")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Formula").weak());
                ui.label("STUDYID-SUBJID");
                ui.end_row();
                ui.label(RichText::new("Study ID").weak());
                ui.label(RichText::new(study_id).color(ui.visuals().hyperlink_color));
                ui.end_row();
            });

        if let Some((col, samples)) = subjid_info {
            ui.add_space(spacing::SM);
            ui.label(RichText::new(format!("SUBJID → {}", col)).weak().small());
            for val in samples {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(val).code());
                    ui.label(RichText::new("→").weak());
                    ui.label(
                        RichText::new(format!("{}-{}", study_id, val))
                            .code()
                            .color(ui.visuals().hyperlink_color),
                    );
                });
            }
        }
    }
}

fn show_source_mapping_inline(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    var_name: &str,
    status: VariableStatus,
    core: Option<CoreDesignation>,
    current_mapping: &Option<(String, f32)>,
    column_label: &Option<String>,
    sample_values: &[String],
    not_collected_reason: &Option<String>,
) {
    ui.label(
        RichText::new(format!("{} Source Column", egui_phosphor::regular::TABLE))
            .strong()
            .weak(),
    );
    ui.separator();
    ui.add_space(spacing::SM);

    let is_subjid = var_name.eq_ignore_ascii_case("SUBJID");
    let var_name_owned = var_name.to_string();

    // Show current mapping if any
    if let Some((col, conf)) = current_mapping {
        ui.horizontal(|ui| {
            ui.label(RichText::new(col).strong());
            let color = if *conf >= 0.95_f32 {
                Color32::GREEN
            } else if *conf >= 0.80_f32 {
                ui.visuals().warn_fg_color
            } else {
                ui.visuals().weak_text_color()
            };
            ui.label(RichText::new(format!("{:.0}%", conf * 100.0)).color(color));
        });

        if let Some(label) = column_label {
            ui.label(RichText::new(label).weak().italics());
        }

        if !sample_values.is_empty() {
            ui.add_space(spacing::SM);
            ui.label(RichText::new(sample_values.join(" · ")).weak().small());
        }
    } else {
        ui.label(
            RichText::new(format!("{} No mapping", egui_phosphor::regular::LINK_BREAK))
                .weak()
                .italics(),
        );
    }

    ui.add_space(spacing::MD);

    // Action buttons based on status
    match status {
        VariableStatus::Accepted => {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} Mapped", egui_phosphor::regular::CHECK_CIRCLE))
                        .color(Color32::GREEN),
                );
            });
            if ui
                .small_button(format!("{} Clear", egui_phosphor::regular::X))
                .clicked()
            {
                with_mapping_state_mut(state, domain_code, |ms| {
                    ms.clear(&var_name_owned);
                    if is_subjid {
                        sync_usubjid_from_subjid(ms);
                    }
                });
            }
        }
        VariableStatus::NotCollected => {
            ui.label(
                RichText::new(format!(
                    "{} Not Collected",
                    egui_phosphor::regular::PROHIBIT
                ))
                .color(ui.visuals().warn_fg_color),
            );
            if let Some(reason) = not_collected_reason {
                ui.label(
                    RichText::new(format!("Reason: {}", reason))
                        .weak()
                        .italics()
                        .small(),
                );
            }
            if ui
                .button(format!("{} Clear", egui_phosphor::regular::X))
                .clicked()
            {
                with_mapping_state_mut(state, domain_code, |ms| {
                    ms.clear_assignment(&var_name_owned);
                });
            }
        }
        VariableStatus::Omitted => {
            ui.label(
                RichText::new(format!("{} Omitted", egui_phosphor::regular::MINUS_CIRCLE))
                    .color(ui.visuals().warn_fg_color),
            );
            if ui
                .button(format!("{} Clear", egui_phosphor::regular::X))
                .clicked()
            {
                with_mapping_state_mut(state, domain_code, |ms| {
                    ms.clear_assignment(&var_name_owned);
                });
            }
        }
        VariableStatus::Suggested => {
            ui.label(
                RichText::new(format!(
                    "{} Suggestion available",
                    egui_phosphor::regular::LIGHTBULB
                ))
                .color(ui.visuals().warn_fg_color),
            );
            ui.add_space(spacing::SM);
            if ui
                .button(
                    RichText::new(format!("{} Accept", egui_phosphor::regular::CHECK))
                        .color(Color32::GREEN),
                )
                .clicked()
            {
                with_mapping_state_mut(state, domain_code, |ms| {
                    let _ = ms.accept_suggestion(&var_name_owned);
                    if is_subjid {
                        sync_usubjid_from_subjid(ms);
                    }
                });
            }
            show_manual_mapping_ui(ui, state, domain_code, &var_name_owned, is_subjid, core);
            show_alternative_actions(ui, state, domain_code, &var_name_owned, core);
        }
        VariableStatus::Unmapped => {
            show_manual_mapping_ui(ui, state, domain_code, &var_name_owned, is_subjid, core);
            show_alternative_actions(ui, state, domain_code, &var_name_owned, core);
        }
    }
}

fn show_manual_mapping_ui(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    var_name: &str,
    is_subjid: bool,
    _core: Option<CoreDesignation>,
) {
    // Get source columns for combo box
    let source_columns: Vec<String> = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .map(|d| d.source_columns())
        .unwrap_or_default();

    ui.add_space(spacing::MD);
    ui.label(RichText::new("Or select manually:").weak());

    let var_name_owned = var_name.to_string();
    egui::ComboBox::from_id_salt(format!("map_{}", var_name))
        .selected_text("Choose column...")
        .show_ui(ui, |ui| {
            for col in source_columns {
                if ui.selectable_label(false, &col).clicked() {
                    with_mapping_state_mut(state, domain_code, |ms| {
                        let _ = ms.accept_manual(&var_name_owned, &col);
                        if is_subjid {
                            sync_usubjid_from_subjid(ms);
                        }
                    });
                }
            }
        });
}

fn show_alternative_actions(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    var_name: &str,
    core: Option<CoreDesignation>,
) {
    ui.add_space(spacing::MD);
    ui.separator();
    ui.add_space(spacing::SM);

    let var_name_owned = var_name.to_string();

    // "Mark as Not Collected" for Expected variables only
    // Required variables cannot be marked as not collected - they are always required
    if core == Some(CoreDesignation::Expected) {
        // Check if we're editing a reason for this variable
        let is_editing = state
            .study
            .as_ref()
            .and_then(|s| s.get_domain(domain_code))
            .and_then(|d| d.mapping_state.as_ref())
            .map(|ms| ms.not_collected_reason_edit.contains_key(var_name))
            .unwrap_or(false);

        if is_editing {
            // Show reason input UI
            ui.label(
                RichText::new(format!(
                    "{} Not Collected Reason",
                    egui_phosphor::regular::PROHIBIT
                ))
                .strong(),
            );
            ui.label(
                RichText::new("Please provide a reason why this variable was not collected")
                    .weak()
                    .small(),
            );
            ui.add_space(spacing::XS);

            // Get the current reason text
            let current_reason = state
                .study
                .as_ref()
                .and_then(|s| s.get_domain(domain_code))
                .and_then(|d| d.mapping_state.as_ref())
                .and_then(|ms| ms.not_collected_reason_edit.get(var_name))
                .cloned()
                .unwrap_or_default();

            let mut reason = current_reason.clone();
            let response = ui.add(
                egui::TextEdit::multiline(&mut reason)
                    .desired_width(f32::INFINITY)
                    .desired_rows(2)
                    .hint_text("Enter reason..."),
            );

            if response.changed() {
                // Update the reason text
                if let Some(study) = &mut state.study {
                    if let Some(domain) = study.get_domain_mut(domain_code) {
                        if let Some(ms) = &mut domain.mapping_state {
                            ms.not_collected_reason_edit
                                .insert(var_name.to_string(), reason.clone());
                        }
                    }
                }
            }

            ui.add_space(spacing::SM);
            let reason_valid = !current_reason.trim().is_empty();
            ui.horizontal(|ui| {
                ui.add_enabled_ui(reason_valid, |ui| {
                    if ui
                        .button(format!("{} Confirm", egui_phosphor::regular::CHECK))
                        .clicked()
                    {
                        let reason_to_use = current_reason.trim().to_string();
                        // Remove from editing state and mark as not collected
                        if let Some(study) = &mut state.study {
                            if let Some(domain) = study.get_domain_mut(domain_code) {
                                if let Some(ms) = &mut domain.mapping_state {
                                    ms.not_collected_reason_edit.remove(&var_name_owned);
                                    let _ = ms.mark_not_collected(&var_name_owned, &reason_to_use);
                                    // Invalidate dependent states
                                    domain.validation = None;
                                    domain.validation_selected_idx = None;
                                    domain.transform_state = None;
                                    domain.preview_data = None;
                                    domain.supp_state = None;
                                }
                            }
                        }
                    }
                });

                if ui
                    .button(format!("{} Cancel", egui_phosphor::regular::X))
                    .clicked()
                {
                    // Remove from editing state
                    if let Some(study) = &mut state.study {
                        if let Some(domain) = study.get_domain_mut(domain_code) {
                            if let Some(ms) = &mut domain.mapping_state {
                                ms.not_collected_reason_edit.remove(&var_name_owned);
                            }
                        }
                    }
                }
            });

            if !reason_valid {
                ui.label(
                    RichText::new(format!(
                        "{} Reason is required",
                        egui_phosphor::regular::WARNING
                    ))
                    .color(ui.visuals().error_fg_color)
                    .small(),
                );
            }
        } else {
            // Show initial button to start editing
            if ui
                .button(format!(
                    "{} Mark Not Collected",
                    egui_phosphor::regular::PROHIBIT
                ))
                .clicked()
            {
                // Start editing - add empty reason
                if let Some(study) = &mut state.study {
                    if let Some(domain) = study.get_domain_mut(domain_code) {
                        if let Some(ms) = &mut domain.mapping_state {
                            ms.not_collected_reason_edit
                                .insert(var_name.to_string(), String::new());
                        }
                    }
                }
            }
        }
    }

    // "Omit" for Perm variables
    if core == Some(CoreDesignation::Permissible) {
        if ui
            .button(format!(
                "{} Omit Variable",
                egui_phosphor::regular::MINUS_CIRCLE
            ))
            .clicked()
        {
            with_mapping_state_mut(state, domain_code, |ms| {
                let _ = ms.mark_omit(&var_name_owned);
            });
        }
    }
}
