//! Mapping tab
//!
//! Interactive column-to-variable mapping with suggestions and CT display.
//! Mapping state is initialized during study loading, so this tab renders immediately.

use crate::state::AppState;
use crate::theme::spacing;
use egui::{Color32, RichText, Ui};
use tss_map::VariableStatus;
use tss_model::{CoreDesignation, VariableRole};

/// Info about a codelist for display.
#[derive(Clone)]
struct CodelistDisplayInfo {
    name: String,
    extensible: bool,
    values: Vec<String>,
}

/// Render the mapping tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Mapping is always ready (initialized during study loading)
    // Use DM-enforced access via state.domain()
    let Some(domain) = state.domain(domain_code) else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Domain not accessible").color(ui.visuals().error_fg_color));
        });
        return;
    };

    // Just verify we have a valid mapping
    let _summary = domain.mapping.summary();

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

fn show_variable_list(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Get UI state for this domain
    let ui_state = state.ui.domain_editor(domain_code);
    let selected_idx = ui_state.mapping.selected_idx;
    let mut search_text = ui_state.mapping.search_filter.clone();

    // Collect data we need from domain (immutable borrow)
    let (summary, filtered_vars, has_subjid_mapping) = {
        let Some(study) = state.study() else {
            ui.label("No study loaded");
            return;
        };
        let Some(domain) = study.get_domain(domain_code) else {
            ui.label("Domain not found");
            return;
        };

        let ms = &domain.mapping;
        let summary = ms.summary();
        // Check if SUBJID has been mapped (accepted), not just if it exists
        let has_subjid_mapping = ms.accepted("SUBJID").is_some();

        // Filter variables by search text
        let search_lower = search_text.to_lowercase();
        let filtered: Vec<_> = ms
            .domain()
            .variables
            .iter()
            .enumerate()
            .filter(|(_, v)| {
                search_text.is_empty() || v.name.to_lowercase().contains(&search_lower)
            })
            .map(|(idx, v)| {
                let status = ms.status(&v.name);
                let core = v.core;
                let role = v.role;
                (idx, v.name.clone(), core, role, status)
            })
            .collect();

        (summary, filtered, has_subjid_mapping)
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
            state.ui.domain_editor(domain_code).mapping.search_filter = search_text;
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
                let (idx, ref name, core, role, status) = filtered_vars[row_idx];
                let is_selected = selected_idx == Some(idx);

                // Check if this is an auto-generated variable
                // USUBJID is only auto-generated if SUBJID has been mapped
                let is_auto = is_auto_generated_variable(name, role)
                    || (has_subjid_mapping && name.eq_ignore_ascii_case("USUBJID"));

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
                        new_selection = Some(idx);
                    }
                });

                // Core column
                row.col(|ui| {
                    ui.label(RichText::new(core_text).weak().small());
                });

                // Status column
                row.col(|ui| {
                    let (icon, status_color) = match status {
                        VariableStatus::Accepted => (egui_phosphor::regular::CHECK_CIRCLE, Color32::GREEN),
                        VariableStatus::AutoGenerated => (egui_phosphor::regular::LIGHTNING, ui.visuals().hyperlink_color),
                        VariableStatus::Suggested => (egui_phosphor::regular::LIGHTBULB, ui.visuals().warn_fg_color),
                        VariableStatus::NotCollected => (egui_phosphor::regular::PROHIBIT, ui.visuals().weak_text_color()),
                        VariableStatus::Omitted => (egui_phosphor::regular::MINUS_CIRCLE, ui.visuals().weak_text_color()),
                        VariableStatus::Unmapped => (egui_phosphor::regular::CIRCLE_DASHED, ui.visuals().weak_text_color()),
                    };
                    if status == VariableStatus::AutoGenerated || is_auto {
                        ui.label(RichText::new("AUTO").color(ui.visuals().hyperlink_color).small());
                    } else {
                        ui.label(RichText::new(icon).color(status_color));
                    }
                });
            });
        });

    // Apply selection change
    if let Some(idx) = new_selection {
        state
            .ui
            .domain_editor(domain_code)
            .mapping
            .select(Some(idx));
    }
}

fn show_variable_detail(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Get UI state for this domain
    let selected_idx = state
        .ui
        .get_domain_editor(domain_code)
        .and_then(|ui| ui.mapping.selected_idx);

    // Extract all needed data before entering closures
    let data = {
        let Some(study) = state.study() else { return };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };

        let ms = &domain.mapping;
        let Some(variable) = selected_idx.and_then(|idx| ms.domain().variables.get(idx)) else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Select a variable from the list").weak());
            });
            return;
        };

        let var_name = variable.name.clone();
        // Check if SUBJID has been mapped (accepted) in this domain
        let has_subjid_mapping = ms.accepted("SUBJID").is_some();
        let is_auto = is_auto_generated(&var_name, variable.role, domain_code, has_subjid_mapping);
        // USUBJID is derived when SUBJID is mapped
        let is_usubjid_derived = has_subjid_mapping && var_name.eq_ignore_ascii_case("USUBJID");
        let status = ms.status(&var_name);
        let study_id = study.study_id.clone();

        // Extract data for display
        let variable_label = variable.label.clone();
        let variable_core = variable.core;
        let variable_data_type = format!("{:?}", variable.data_type);
        let variable_role = variable.role.map(|r| r.as_str().to_string());
        let codelist_code = variable.codelist_code.clone();

        // Load CT info for the codelist (from cached registry)
        let ct_info: Option<CodelistDisplayInfo> = codelist_code.as_ref().and_then(|code| {
            let registry = state.ct_registry()?;
            let resolved = registry.resolve(code, None)?;
            Some(CodelistDisplayInfo {
                name: resolved.codelist.name.clone(),
                extensible: resolved.codelist.extensible,
                values: resolved
                    .codelist
                    .submission_values()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            })
        });

        // Source mapping data
        let current_mapping = ms
            .accepted(&var_name)
            .or(ms.suggestion(&var_name))
            .map(|(col, conf)| (col.to_string(), conf));

        let column_label = current_mapping
            .as_ref()
            .and_then(|(col, _)| study.column_label(col).map(|s| s.to_string()));

        let sample_values: Vec<String> = current_mapping
            .as_ref()
            .map(|(col, _)| get_sample_values(&domain.source.data, col, 5))
            .unwrap_or_default();

        // USUBJID derivation info
        let subjid_info = if is_usubjid_derived {
            ms.accepted("SUBJID").map(|(col, _)| {
                let samples = get_sample_values(&domain.source.data, col, 3);
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
                ui.label(variable_core.map(|c| c.as_code()).unwrap_or("—"));
                ui.end_row();

                ui.label(RichText::new("Type").weak());
                ui.label(&variable_data_type);
                ui.end_row();

                ui.label(RichText::new("Role").weak());
                ui.label(variable_role.as_deref().unwrap_or("—"));
                ui.end_row();

                if let Some(cl) = &codelist_code {
                    ui.label(RichText::new("Codelist").weak());
                    ui.label(cl);
                    ui.end_row();
                }
            });

        // Show CT values if available
        if let Some(ct) = &ct_info {
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

            // Codelist header with name and extensibility
            ui.horizontal(|ui| {
                ui.label(RichText::new(&ct.name).strong());
                if ct.extensible {
                    ui.label(RichText::new("(extensible)").small().weak().italics());
                }
            });

            if let Some(code) = &codelist_code {
                ui.label(RichText::new(format!("Code: {}", code)).weak().small());
            }

            ui.add_space(spacing::SM);

            // Show valid values in a scrollable area
            let max_height = 120.0;
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (i, value) in ct.values.iter().enumerate() {
                            if i > 0 {
                                ui.label(RichText::new("·").weak());
                            }
                            ui.label(
                                RichText::new(value)
                                    .monospace()
                                    .small()
                                    .background_color(ui.visuals().code_bg_color),
                            );
                        }
                    });
                });

            ui.add_space(spacing::SM);
            ui.label(
                RichText::new(format!("{} valid values", ct.values.len()))
                    .weak()
                    .small(),
            );
        } else if codelist_code.is_some() {
            // Fallback: show just the codelist code if CT couldn't be loaded
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

            ui.label(
                RichText::new(format!("Codelist: {}", codelist_code.as_ref().unwrap()))
                    .weak()
                    .small(),
            );
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

/// Get sample values from a DataFrame column
fn get_sample_values(
    df: &polars::prelude::DataFrame,
    column_name: &str,
    limit: usize,
) -> Vec<String> {
    use tss_common::any_to_string;

    df.column(column_name)
        .ok()
        .map(|series| {
            (0..limit.min(series.len()))
                .filter_map(|i| series.get(i).ok().map(any_to_string))
                .filter(|s| !s.is_empty() && s != "null")
                .collect()
        })
        .unwrap_or_default()
}

/// Check if a variable is auto-generated (not mapped from source)
fn is_auto_generated_variable(name: &str, role: Option<VariableRole>) -> bool {
    if role != Some(VariableRole::Identifier) {
        return false;
    }
    matches!(name, "STUDYID" | "DOMAIN") || (name.ends_with("SEQ") && name.len() >= 4)
}

/// Check if a variable is auto-generated based on role, name, and SUBJID mapping.
///
/// USUBJID is auto-generated when SUBJID has been mapped (in any domain).
fn is_auto_generated(
    name: &str,
    role: Option<VariableRole>,
    _domain_code: &str,
    has_subjid: bool,
) -> bool {
    // USUBJID is auto-generated when SUBJID is mapped (derives as STUDYID-SUBJID)
    if has_subjid && name.eq_ignore_ascii_case("USUBJID") {
        return true;
    }
    is_auto_generated_variable(name, role)
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
    let is_subjid = var_name.eq_ignore_ascii_case("SUBJID");
    let is_usubjid = var_name.eq_ignore_ascii_case("USUBJID");
    let is_dm = domain_code.eq_ignore_ascii_case("DM");
    let var_name_owned = var_name.to_string();

    // Special header for USUBJID in non-DM domains
    if is_usubjid && !is_dm {
        // Get study_id for formula display
        let study_id = state
            .study()
            .map(|s| s.study_id.clone())
            .unwrap_or_default();

        ui.label(
            RichText::new(format!(
                "{} USUBJID Derivation",
                egui_phosphor::regular::LIGHTNING
            ))
            .strong()
            .weak(),
        );
        ui.separator();
        ui.add_space(spacing::SM);

        // Explain the formula
        ui.horizontal(|ui| {
            ui.label(RichText::new("Formula:").weak());
            ui.label(
                RichText::new("STUDYID")
                    .code()
                    .color(ui.visuals().hyperlink_color),
            );
            ui.label(RichText::new("-").weak());
            ui.label(
                RichText::new("Subject ID")
                    .code()
                    .color(ui.visuals().warn_fg_color),
            );
        });

        ui.add_space(spacing::XS);
        ui.label(
            RichText::new("Map the subject identifier column from your source data.")
                .weak()
                .italics()
                .small(),
        );

        ui.add_space(spacing::MD);

        // Show mapping status
        if let Some((col, _conf)) = current_mapping {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} Subject ID column:",
                        egui_phosphor::regular::CHECK_CIRCLE
                    ))
                    .color(Color32::GREEN),
                );
                ui.label(RichText::new(col).strong());
            });

            // Show sample transformations
            if !sample_values.is_empty() {
                ui.add_space(spacing::SM);
                ui.label(RichText::new("Sample transformations:").weak().small());
                for val in sample_values.iter().take(3) {
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
        } else {
            ui.label(
                RichText::new(format!(
                    "{} No subject ID column mapped",
                    egui_phosphor::regular::WARNING
                ))
                .color(ui.visuals().warn_fg_color),
            );
        }

        ui.add_space(spacing::MD);
    } else {
        ui.label(
            RichText::new(format!("{} Source Column", egui_phosphor::regular::TABLE))
                .strong()
                .weak(),
        );
        ui.separator();
        ui.add_space(spacing::SM);
    }

    // Show current mapping if any (skip for USUBJID in non-DM since we already showed it above)
    if !(is_usubjid && !is_dm) {
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
    }

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
                if let Some(domain) = state.domain_mut(domain_code) {
                    domain.with_mapping(|ms| {
                        ms.clear(&var_name_owned);
                        if is_subjid {
                            sync_usubjid_from_subjid(ms);
                        }
                    });
                }
                state.invalidate_preview(domain_code);
            }
        }
        VariableStatus::AutoGenerated => {
            ui.label(
                RichText::new(format!(
                    "{} Auto-generated",
                    egui_phosphor::regular::LIGHTNING
                ))
                .color(ui.visuals().hyperlink_color),
            );
            ui.add_space(spacing::SM);
            ui.label(
                RichText::new("This variable is automatically populated by the transform system.")
                    .weak()
                    .small(),
            );
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
                if let Some(domain) = state.domain_mut(domain_code) {
                    domain.with_mapping(|ms| {
                        ms.clear_assignment(&var_name_owned);
                    });
                }
                state.invalidate_preview(domain_code);
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
                if let Some(domain) = state.domain_mut(domain_code) {
                    domain.with_mapping(|ms| {
                        ms.clear_assignment(&var_name_owned);
                    });
                }
                state.invalidate_preview(domain_code);
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
                if let Some(domain) = state.domain_mut(domain_code) {
                    domain.with_mapping(|ms| {
                        let _ = ms.accept_suggestion(&var_name_owned);
                        if is_subjid {
                            sync_usubjid_from_subjid(ms);
                        }
                    });
                }
                state.invalidate_preview(domain_code);
            }
            show_manual_mapping_ui(ui, state, domain_code, &var_name_owned, is_subjid);
            show_alternative_actions(ui, state, domain_code, &var_name_owned, core);
        }
        VariableStatus::Unmapped => {
            show_manual_mapping_ui(ui, state, domain_code, &var_name_owned, is_subjid);
            show_alternative_actions(ui, state, domain_code, &var_name_owned, core);
        }
    }
}

fn sync_usubjid_from_subjid(ms: &mut tss_map::MappingState) {
    let has_subjid = ms.accepted("SUBJID").is_some();

    if has_subjid {
        ms.mark_auto_generated("USUBJID");
    } else {
        ms.clear_auto_generated("USUBJID");
    }
}

fn show_manual_mapping_ui(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    var_name: &str,
    is_subjid: bool,
) {
    // Get source columns and already-mapped columns
    let (source_columns, used_columns): (Vec<String>, std::collections::HashSet<String>) = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .map(|d| {
            let sources = d.source.columns();
            // Get source columns that are already mapped to OTHER variables
            let used: std::collections::HashSet<String> = d
                .mapping
                .all_accepted()
                .iter()
                .filter(|(target_var, _)| *target_var != var_name) // Exclude current variable
                .map(|(_, (src_col, _))| src_col.clone())
                .collect();
            (sources, used)
        })
        .unwrap_or_default();

    let is_usubjid = var_name.eq_ignore_ascii_case("USUBJID");
    let is_dm = domain_code.eq_ignore_ascii_case("DM");

    ui.add_space(spacing::MD);

    // Show appropriate label based on variable type
    let label = if is_usubjid && !is_dm {
        "Select subject identifier column:"
    } else {
        "Or select manually:"
    };
    ui.label(RichText::new(label).weak());

    let var_name_owned = var_name.to_string();
    let placeholder = if is_usubjid && !is_dm {
        "Choose subject ID column..."
    } else {
        "Choose column..."
    };

    let mut mapping_changed = false;
    egui::ComboBox::from_id_salt(format!("map_{}", var_name))
        .selected_text(placeholder)
        .show_ui(ui, |ui| {
            for col in source_columns {
                let is_used = used_columns.contains(&col);

                // Show used columns as disabled/grayed out
                if is_used {
                    ui.add_enabled(
                        false,
                        egui::Button::new(RichText::new(format!("{} (in use)", col)).weak())
                            .frame(false),
                    );
                } else if ui.selectable_label(false, &col).clicked() {
                    if let Some(domain) = state.domain_mut(domain_code) {
                        let col_clone = col.clone();
                        domain.with_mapping(|ms| {
                            let _ = ms.accept_manual(&var_name_owned, &col_clone);
                            if is_subjid {
                                sync_usubjid_from_subjid(ms);
                            }
                        });
                        mapping_changed = true;
                    }
                }
            }
        });
    if mapping_changed {
        state.invalidate_preview(domain_code);
    }
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
    if core == Some(CoreDesignation::Expected) {
        // Check if we're editing a reason for this variable
        let is_editing = state
            .ui
            .get_domain_editor(domain_code)
            .map(|ui| ui.mapping.not_collected_editing.contains_key(var_name))
            .unwrap_or(false);

        if is_editing {
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

            let current_reason = state
                .ui
                .get_domain_editor(domain_code)
                .and_then(|ui| ui.mapping.not_collected_editing.get(var_name))
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
                state
                    .ui
                    .domain_editor(domain_code)
                    .mapping
                    .set_editing_reason(var_name, &reason);
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
                        state
                            .ui
                            .domain_editor(domain_code)
                            .mapping
                            .clear_editing_reason(&var_name_owned);
                        if let Some(domain) = state.domain_mut(domain_code) {
                            domain.with_mapping(|ms| {
                                let _ = ms.mark_not_collected(&var_name_owned, &reason_to_use);
                            });
                        }
                        state.invalidate_preview(domain_code);
                    }
                });

                if ui
                    .button(format!("{} Cancel", egui_phosphor::regular::X))
                    .clicked()
                {
                    state
                        .ui
                        .domain_editor(domain_code)
                        .mapping
                        .clear_editing_reason(&var_name_owned);
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
            if ui
                .button(format!(
                    "{} Mark Not Collected",
                    egui_phosphor::regular::PROHIBIT
                ))
                .clicked()
            {
                state
                    .ui
                    .domain_editor(domain_code)
                    .mapping
                    .set_editing_reason(var_name, "");
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
            if let Some(domain) = state.domain_mut(domain_code) {
                domain.with_mapping(|ms| {
                    let _ = ms.mark_omit(&var_name_owned);
                });
            }
            state.invalidate_preview(domain_code);
        }
    }
}
