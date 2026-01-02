//! SUPP tab
//!
//! Configure Supplemental Qualifiers for unmapped source columns.
//!
//! Unmapped columns (columns not mapped to any SDTM variable) can be:
//! - Added to SUPP domain with QNAM/QLABEL configuration
//! - Skipped (excluded from export)

use crate::state::{AppState, SuppAction, SuppColumnConfig, SuppState};
use crate::theme::spacing;
use crate::views::domain_editor::ensure_mapping_initialized;
use egui::{Color32, RichText, Ui};
use polars::prelude::AnyValue;
use sdtm_model::CoreDesignation;

/// Check if all SDTM variables have a mapping decision.
///
/// This check is based ONLY on mapping status, NOT on validation results.
/// A variable is considered "resolved" if it has status: Accepted, NotCollected, or Omitted.
///
/// Note: Auto-generated variables (STUDYID, DOMAIN, --SEQ) are auto-accepted at
/// initialization time. USUBJID is auto-accepted when SUBJID is accepted.
/// So this check simply looks at the status without needing special filtering.
///
/// Returns (is_complete, pending_required, pending_expected, pending_permissible)
fn check_mapping_complete(state: &AppState, domain_code: &str) -> (bool, usize, usize, usize) {
    let Some(study) = &state.study else {
        return (false, 0, 0, 0);
    };
    let Some(domain) = study.get_domain(domain_code) else {
        return (false, 0, 0, 0);
    };
    let Some(ms) = &domain.mapping_state else {
        return (false, 0, 0, 0);
    };

    let sdtm_domain = ms.domain();
    let mut pending_required = 0usize;
    let mut pending_expected = 0usize;
    let mut pending_permissible = 0usize;

    for var in &sdtm_domain.variables {
        let status = ms.status(&var.name);
        let is_resolved = matches!(
            status,
            sdtm_map::VariableStatus::Accepted
                | sdtm_map::VariableStatus::NotCollected
                | sdtm_map::VariableStatus::Omitted
        );

        if !is_resolved {
            match var.core {
                Some(CoreDesignation::Required) => pending_required += 1,
                Some(CoreDesignation::Expected) => pending_expected += 1,
                Some(CoreDesignation::Permissible) => {
                    // Only count Perm as pending if it has a suggestion
                    if matches!(status, sdtm_map::VariableStatus::Suggested) {
                        pending_permissible += 1;
                    }
                }
                None => {}
            }
        }
    }

    let is_complete = pending_required == 0 && pending_expected == 0 && pending_permissible == 0;
    (
        is_complete,
        pending_required,
        pending_expected,
        pending_permissible,
    )
}

/// Render the SUPP tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Ensure mapping is initialized first
    if !ensure_mapping_initialized(ui, state, domain_code) {
        return;
    }

    // Check if mapping is complete before allowing SUPP configuration
    let (mapping_complete, pending_req, pending_exp, pending_perm) =
        check_mapping_complete(state, domain_code);
    if !mapping_complete {
        show_mapping_incomplete_message(ui, pending_req, pending_exp, pending_perm);
        return;
    }

    // Lazy rebuild SUPP state if needed
    rebuild_supp_state_if_needed(state, domain_code);

    // Get domain state for display
    let Some(study) = &state.study else {
        ui.label("No study loaded");
        return;
    };

    let Some(domain) = study.get_domain(domain_code) else {
        ui.label(format!("Domain {} not found", domain_code));
        return;
    };

    // Check if SUPP state is available
    let Some(ref supp_state) = domain.supp_state else {
        ui.centered_and_justified(|ui| {
            ui.spinner();
            ui.label("Initializing SUPP configuration...");
        });
        return;
    };

    // If no unmapped columns, show empty state
    if supp_state.columns.is_empty() {
        show_empty_state(ui);
        return;
    }

    // Render master-detail layout
    let available_height = ui.available_height();

    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(300.0)) // Left panel
        .size(egui_extras::Size::exact(1.0)) // Separator
        .size(egui_extras::Size::remainder()) // Right panel
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                show_column_list(ui, state, domain_code, available_height);
            });
            strip.cell(|ui| {
                ui.separator();
            });
            strip.cell(|ui| {
                show_column_detail(ui, state, domain_code, available_height);
            });
        });
}

/// Show empty state when no unmapped columns
fn show_empty_state(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(spacing::XL);
        ui.label(
            RichText::new(format!(
                "{} All Columns Mapped",
                egui_phosphor::regular::CHECK_CIRCLE
            ))
            .color(Color32::GREEN)
            .size(18.0),
        );
        ui.add_space(spacing::MD);
        ui.label(
            RichText::new("All source columns are mapped to SDTM variables.")
                .color(ui.visuals().weak_text_color()),
        );
        ui.label(
            RichText::new("No supplemental qualifiers needed for this domain.")
                .weak()
                .small(),
        );
    });
}

/// Show message when mapping is incomplete
fn show_mapping_incomplete_message(
    ui: &mut Ui,
    pending_required: usize,
    pending_expected: usize,
    pending_permissible: usize,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(spacing::XL);
        ui.label(
            RichText::new(format!(
                "{} Complete Mapping First",
                egui_phosphor::regular::WARNING
            ))
            .color(ui.visuals().warn_fg_color)
            .size(18.0),
        );
        ui.add_space(spacing::MD);
        ui.label(
            RichText::new(
                "Before configuring supplemental qualifiers, please complete all mapping decisions.",
            )
            .color(ui.visuals().weak_text_color()),
        );
        ui.add_space(spacing::SM);

        if pending_required > 0 {
            ui.label(
                RichText::new(format!(
                    "{} {} Required variable(s) need resolution",
                    egui_phosphor::regular::ASTERISK,
                    pending_required
                ))
                .color(ui.visuals().error_fg_color),
            );
        }
        if pending_expected > 0 {
            ui.label(
                RichText::new(format!(
                    "{} {} Expected variable(s) need resolution",
                    egui_phosphor::regular::CIRCLE,
                    pending_expected
                ))
                .color(ui.visuals().warn_fg_color),
            );
        }
        if pending_permissible > 0 {
            ui.label(
                RichText::new(format!(
                    "{} {} Permissible variable(s) with suggestions pending",
                    egui_phosphor::regular::CIRCLE_DASHED,
                    pending_permissible
                ))
                .color(ui.visuals().weak_text_color()),
            );
        }

        ui.add_space(spacing::MD);
        ui.label(
            RichText::new("Go to the Mapping tab to resolve these variables.")
                .weak()
                .italics(),
        );
    });
}

/// Show the column list (left panel)
fn show_column_list(ui: &mut Ui, state: &mut AppState, domain_code: &str, available_height: f32) {
    // Get SUPP state for display
    let (columns, selected, counts) = {
        let study = state.study.as_ref().unwrap();
        let domain = study.get_domain(domain_code).unwrap();
        let supp = domain.supp_state.as_ref().unwrap();
        let cols: Vec<(String, SuppAction)> = supp
            .columns
            .iter()
            .map(|(k, v)| (k.clone(), v.action))
            .collect();
        let sel = supp.selected_column.clone();
        let counts = supp.count_by_action();
        (cols, sel, counts)
    };

    // Header
    ui.label(
        RichText::new(format!(
            "{} Unmapped Columns",
            egui_phosphor::regular::TABLE
        ))
        .strong(),
    );

    // Summary counts
    let (pending, added, skipped) = counts;
    ui.label(
        RichText::new(format!(
            "{} pending · {} SUPP · {} skipped",
            pending, added, skipped
        ))
        .weak()
        .small(),
    );

    ui.add_space(spacing::SM);
    ui.separator();
    ui.add_space(spacing::SM);

    // Column list
    egui::ScrollArea::vertical()
        .max_height(available_height - 100.0)
        .show(ui, |ui| {
            for (column_name, action) in &columns {
                let is_selected = selected.as_ref() == Some(column_name);
                show_column_row(ui, state, domain_code, column_name, *action, is_selected);
            }
        });
}

/// Show a single column row
fn show_column_row(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    column_name: &str,
    action: SuppAction,
    is_selected: bool,
) {
    let (icon, color) = match action {
        SuppAction::Pending => (
            egui_phosphor::regular::CIRCLE_DASHED,
            ui.visuals().weak_text_color(),
        ),
        SuppAction::AddToSupp => (egui_phosphor::regular::CHECK, Color32::GREEN),
        SuppAction::Skip => (egui_phosphor::regular::MINUS, ui.visuals().warn_fg_color),
    };

    ui.horizontal(|ui| {
        ui.label(RichText::new(icon).color(color));

        let response = ui.selectable_label(is_selected, column_name);
        if response.clicked() {
            // Update selected column
            if let Some(study) = &mut state.study {
                if let Some(domain) = study.get_domain_mut(domain_code) {
                    if let Some(supp) = &mut domain.supp_state {
                        supp.selected_column = Some(column_name.to_string());
                    }
                }
            }
        }

        // Show QNAM on right if AddToSupp
        if action == SuppAction::AddToSupp {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Get QNAM for this column
                let qnam = state
                    .study
                    .as_ref()
                    .and_then(|s| s.get_domain(domain_code))
                    .and_then(|d| d.supp_state.as_ref())
                    .and_then(|s| s.columns.get(column_name))
                    .map(|c| c.qnam.clone())
                    .unwrap_or_default();
                ui.label(RichText::new(qnam).monospace().small());
            });
        }
    });
}

/// Show column detail panel (right panel)
fn show_column_detail(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    _available_height: f32,
) {
    // Get selected column
    let selected = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .and_then(|d| d.supp_state.as_ref())
        .and_then(|s| s.selected_column.clone());

    let Some(column_name) = selected else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Select a column to configure").weak());
        });
        return;
    };

    // Get config for this column
    let config = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .and_then(|d| d.supp_state.as_ref())
        .and_then(|s| s.columns.get(&column_name))
        .cloned();

    let Some(config) = config else {
        ui.label("Column not found");
        return;
    };

    // Header
    ui.heading(&column_name);
    ui.label(
        RichText::new("Source column for SUPP configuration")
            .weak()
            .small(),
    );

    ui.add_space(spacing::MD);
    ui.separator();
    ui.add_space(spacing::SM);

    // Sample values
    show_sample_values(ui, state, domain_code, &column_name);

    ui.add_space(spacing::LG);

    // Action selection
    ui.label(RichText::new("Action").strong());
    ui.add_space(spacing::SM);

    ui.horizontal(|ui| {
        // Add to SUPP button
        let add_selected = config.action == SuppAction::AddToSupp;
        if ui
            .selectable_label(
                add_selected,
                format!("{} Add to SUPP", egui_phosphor::regular::PLUS),
            )
            .clicked()
        {
            update_supp_action(state, domain_code, &column_name, SuppAction::AddToSupp);
        }

        ui.add_space(spacing::MD);

        // Skip button
        let skip_selected = config.action == SuppAction::Skip;
        if ui
            .selectable_label(
                skip_selected,
                format!("{} Skip", egui_phosphor::regular::MINUS),
            )
            .clicked()
        {
            update_supp_action(state, domain_code, &column_name, SuppAction::Skip);
        }
    });

    // QNAM/QLABEL fields (only if AddToSupp)
    if config.action == SuppAction::AddToSupp {
        ui.add_space(spacing::LG);
        ui.separator();
        ui.add_space(spacing::SM);

        show_qnam_input(ui, state, domain_code, &column_name, &config);
        ui.add_space(spacing::MD);
        show_qlabel_input(ui, state, domain_code, &column_name, &config);
    }
}

/// Show sample values from source data
fn show_sample_values(ui: &mut Ui, state: &mut AppState, domain_code: &str, column_name: &str) {
    ui.label(RichText::new("Sample Values").strong());
    ui.add_space(spacing::XS);

    // Get sample values from source data
    let samples = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .map(|d| get_sample_values_from_df(&d.source_data, column_name, 5))
        .unwrap_or_default();

    if samples.is_empty() {
        ui.label(RichText::new("No sample values available").weak().small());
    } else {
        ui.horizontal_wrapped(|ui| {
            for (i, sample) in samples.iter().enumerate() {
                if i > 0 {
                    ui.label(RichText::new("·").weak());
                }
                ui.label(RichText::new(sample).weak().small());
            }
        });
    }
}

/// Get sample values from DataFrame
fn get_sample_values_from_df(
    df: &polars::prelude::DataFrame,
    column: &str,
    limit: usize,
) -> Vec<String> {
    let Ok(series) = df.column(column) else {
        return Vec::new();
    };

    let mut seen = std::collections::HashSet::new();
    let mut samples = Vec::new();

    for i in 0..series.len().min(100) {
        // Check first 100 rows for unique values
        if let Ok(value) = series.get(i) {
            let s = match value {
                AnyValue::String(s) => s.to_string(),
                AnyValue::Null => continue,
                other => format!("{}", other),
            };
            if !s.is_empty() && seen.insert(s.clone()) {
                samples.push(s);
                if samples.len() >= limit {
                    break;
                }
            }
        }
    }

    samples
}

/// Show QNAM input field
fn show_qnam_input(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    column_name: &str,
    config: &SuppColumnConfig,
) {
    ui.label(RichText::new("QNAM").strong());
    ui.label(
        RichText::new("Max 8 characters, uppercase, no leading numbers")
            .weak()
            .small(),
    );

    ui.add_space(spacing::XS);

    ui.horizontal(|ui| {
        let mut qnam = config.qnam.clone();
        let response = ui.add(
            egui::TextEdit::singleline(&mut qnam)
                .desired_width(120.0)
                .char_limit(8)
                .font(egui::TextStyle::Monospace),
        );

        if response.changed() {
            let validated = qnam.to_uppercase();
            update_qnam(state, domain_code, column_name, validated);
        }

        // Suggest button
        if config.qnam != config.suggested_qnam {
            if ui
                .small_button(format!("Suggest: {}", config.suggested_qnam))
                .clicked()
            {
                update_qnam(
                    state,
                    domain_code,
                    column_name,
                    config.suggested_qnam.clone(),
                );
            }
        }
    });

    // Validation message
    if let Err(msg) = config.validate_qnam() {
        ui.label(
            RichText::new(format!("{} {}", egui_phosphor::regular::WARNING, msg))
                .color(ui.visuals().error_fg_color)
                .small(),
        );
    }
}

/// Show QLABEL input field
fn show_qlabel_input(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    column_name: &str,
    config: &SuppColumnConfig,
) {
    ui.label(RichText::new("QLABEL").strong());
    ui.label(
        RichText::new("Max 40 characters - descriptive label for the variable")
            .weak()
            .small(),
    );

    ui.add_space(spacing::XS);

    let mut qlabel = config.qlabel.clone();
    let response = ui.add(
        egui::TextEdit::singleline(&mut qlabel)
            .desired_width(300.0)
            .char_limit(40),
    );

    if response.changed() {
        update_qlabel(state, domain_code, column_name, qlabel.clone());
    }

    // Character count
    ui.label(
        RichText::new(format!("{}/40 characters", config.qlabel.len()))
            .weak()
            .small(),
    );
}

/// Update SUPP action for a column
fn update_supp_action(
    state: &mut AppState,
    domain_code: &str,
    column_name: &str,
    action: SuppAction,
) {
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            if let Some(supp) = &mut domain.supp_state {
                if let Some(config) = supp.columns.get_mut(column_name) {
                    config.action = action;
                }
            }
        }
    }
}

/// Update QNAM for a column
fn update_qnam(state: &mut AppState, domain_code: &str, column_name: &str, qnam: String) {
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            if let Some(supp) = &mut domain.supp_state {
                if let Some(config) = supp.columns.get_mut(column_name) {
                    config.qnam = qnam;
                }
            }
        }
    }
}

/// Update QLABEL for a column
fn update_qlabel(state: &mut AppState, domain_code: &str, column_name: &str, qlabel: String) {
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            if let Some(supp) = &mut domain.supp_state {
                if let Some(config) = supp.columns.get_mut(column_name) {
                    config.qlabel = qlabel;
                }
            }
        }
    }
}

/// Rebuild SUPP state if needed (lazy initialization)
fn rebuild_supp_state_if_needed(state: &mut AppState, domain_code: &str) {
    // Check if we need to rebuild
    let needs_rebuild = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .map(|d| d.mapping_state.is_some() && d.supp_state.is_none())
        .unwrap_or(false);

    if !needs_rebuild {
        return;
    }

    // Get unmapped columns from MappingState
    let unmapped_columns: Vec<String> = {
        let study = state.study.as_ref().unwrap();
        let domain = study.get_domain(domain_code).unwrap();
        let ms = domain.mapping_state.as_ref().unwrap();
        ms.available_columns()
            .iter()
            .map(|s| s.to_string())
            .collect()
    };

    // Build initial SuppState
    let mut columns = std::collections::BTreeMap::new();
    for col in unmapped_columns {
        columns.insert(col.clone(), SuppColumnConfig::new(col, domain_code));
    }

    let supp_state = SuppState {
        columns,
        selected_column: None,
    };

    // Store in DomainState
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            domain.supp_state = Some(supp_state);
        }
    }
}
