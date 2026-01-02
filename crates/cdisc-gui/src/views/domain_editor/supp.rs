//! SUPP tab
//!
//! Shows SUPP domain configuration for non-standard variables.
//! Uses a 2-column layout: column list on left, configuration on right.
//! Unmapped source columns can be added to a SUPP-- dataset.

use crate::state::{
    AppState, SuppAction, SuppColumnConfig, SuppConfig, suggest_qnam, validate_qnam,
};
use crate::theme::spacing;
use egui::{Color32, RichText, Ui};

/// Render the SUPP tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Check if domain exists
    let Some(domain) = state.domain(domain_code) else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Domain not accessible").color(ui.visuals().error_fg_color));
        });
        return;
    };

    // Check if SUPP config needs building
    let has_supp = domain.derived.supp.is_some();

    if !has_supp {
        // Build SUPP config
        rebuild_supp_config(state, domain_code);
    }

    // Get SUPP config (clone to avoid borrow issues)
    let supp_config = state
        .domain(domain_code)
        .and_then(|d| d.derived.supp.as_ref())
        .cloned();

    let Some(supp_config) = supp_config else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("No SUPP configuration available").weak());
        });
        return;
    };

    // Check if there are any unmapped columns
    if supp_config.columns.is_empty() {
        show_no_unmapped(ui);
        return;
    }

    // 2-column layout using StripBuilder
    let available_height = ui.available_height();

    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(280.0)) // Left panel fixed width
        .size(egui_extras::Size::exact(1.0)) // Separator
        .size(egui_extras::Size::remainder()) // Right panel takes rest
        .horizontal(|mut strip| {
            // Left: Column list
            strip.cell(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        show_column_list(ui, state, domain_code, &supp_config);
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
                        show_column_detail(ui, state, domain_code, &supp_config);
                    });
            });
        });
}

/// Show "no unmapped columns" state
fn show_no_unmapped(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(spacing::XL);
        ui.label(
            RichText::new(egui_phosphor::regular::CHECK_CIRCLE)
                .size(48.0)
                .color(Color32::from_rgb(100, 180, 100)),
        );
        ui.add_space(spacing::MD);
        ui.label(RichText::new("All Columns Mapped").size(18.0).strong());
        ui.add_space(spacing::SM);
        ui.label(RichText::new("No unmapped source columns require SUPP configuration.").weak());
    });
}

/// Show the column list (left panel)
fn show_column_list(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    supp_config: &SuppConfig,
) {
    let selected_column = state
        .ui
        .get_domain_editor(domain_code)
        .and_then(|ui| ui.supp.selected_column.clone());

    let (pending, added, skipped) = supp_config.count_by_action();
    let total = supp_config.columns.len();

    // Header
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{}", total)).strong());
        ui.label(RichText::new("unmapped").weak().small());
        ui.separator();
        if added > 0 {
            ui.label(
                RichText::new(format!("{} added", added))
                    .small()
                    .color(Color32::from_rgb(100, 180, 100)),
            );
        }
        if pending > 0 {
            ui.label(
                RichText::new(format!("{} pending", pending))
                    .small()
                    .color(ui.visuals().warn_fg_color),
            );
        }
        if skipped > 0 {
            ui.label(RichText::new(format!("{} skipped", skipped)).small().weak());
        }
    });

    ui.add_space(spacing::SM);
    ui.separator();

    // Column list table
    let column_names: Vec<&str> = supp_config.column_names();
    let mut new_selection: Option<String> = None;
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(egui_extras::Column::exact(24.0)) // Status icon
        .column(egui_extras::Column::remainder().at_least(80.0)) // Column name
        .column(egui_extras::Column::exact(60.0)) // QNAM preview
        .header(text_height + 4.0, |mut header| {
            header.col(|_ui| {});
            header.col(|ui| {
                ui.label(RichText::new("Column").small().strong());
            });
            header.col(|ui| {
                ui.label(RichText::new("QNAM").small().strong());
            });
        })
        .body(|body| {
            body.rows(text_height + 8.0, column_names.len(), |mut row| {
                let row_idx = row.index();
                let col_name = column_names[row_idx];
                let config = supp_config.get(col_name).unwrap();
                let is_selected = selected_column.as_deref() == Some(col_name);

                // Status icon column
                row.col(|ui| {
                    let (icon, color) = action_icon_color(config.action, ui);
                    ui.label(RichText::new(icon).color(color));
                });

                // Column name (clickable)
                row.col(|ui| {
                    let mut label_text = RichText::new(col_name).monospace();
                    if is_selected {
                        label_text = label_text.strong();
                    }

                    let response = ui.selectable_label(is_selected, label_text);
                    if response.clicked() {
                        new_selection = Some(col_name.to_string());
                    }
                });

                // QNAM preview
                row.col(|ui| {
                    if config.action == SuppAction::AddToSupp && !config.qnam.is_empty() {
                        ui.label(RichText::new(&config.qnam).monospace().small());
                    } else {
                        ui.label(RichText::new("-").weak().small());
                    }
                });
            });
        });

    // Apply selection change
    if let Some(col_name) = new_selection {
        let selection = if selected_column.as_deref() == Some(&col_name) {
            None // Toggle off
        } else {
            Some(col_name)
        };
        state.ui.domain_editor(domain_code).supp.select(selection);
    }
}

/// Show the column detail (right panel)
fn show_column_detail(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    supp_config: &SuppConfig,
) {
    let selected_column = state
        .ui
        .get_domain_editor(domain_code)
        .and_then(|ui| ui.supp.selected_column.clone());

    let Some(col_name) = selected_column else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Select a column from the list").weak());
        });
        return;
    };

    let Some(config) = supp_config.get(&col_name) else {
        ui.label(RichText::new("Column not found").weak());
        return;
    };

    let (icon, color) = action_icon_color(config.action, ui);

    // Header
    ui.horizontal(|ui| {
        ui.label(RichText::new(icon).color(color).size(20.0));
        ui.heading(&col_name);
    });

    ui.add_space(spacing::SM);

    // Status badge
    let status_label = match config.action {
        SuppAction::Pending => "PENDING",
        SuppAction::AddToSupp => "ADDED TO SUPP",
        SuppAction::Skip => "SKIPPED",
    };
    ui.label(RichText::new(status_label).small().color(color).strong());

    ui.add_space(spacing::LG);

    // Current configuration section
    ui.label(
        RichText::new(format!(
            "{} Current Configuration",
            egui_phosphor::regular::GEAR
        ))
        .strong()
        .weak(),
    );
    ui.separator();
    ui.add_space(spacing::SM);

    egui::Grid::new("supp_current_config")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("QNAM").weak());
            if config.qnam.is_empty() {
                ui.label(RichText::new("(not set)").weak().italics());
            } else {
                let is_valid = config.validate_qnam().is_ok();
                let text = RichText::new(&config.qnam).monospace();
                if is_valid {
                    ui.label(text);
                } else {
                    ui.label(text.color(ui.visuals().error_fg_color));
                }
            }
            ui.end_row();

            ui.label(RichText::new("QLABEL").weak());
            if config.qlabel.is_empty() {
                ui.label(RichText::new("(not set)").weak().italics());
            } else {
                ui.label(&config.qlabel);
            }
            ui.end_row();
        });

    ui.add_space(spacing::LG);

    // Check if we're in editing mode
    let is_editing = state
        .ui
        .get_domain_editor(domain_code)
        .and_then(|ui| ui.supp.editing_for(&col_name))
        .is_some();

    if is_editing {
        show_editing_form(ui, state, domain_code, &col_name, config);
    } else {
        show_action_buttons(ui, state, domain_code, &col_name, config);
    }
}

/// Show action buttons (when not editing)
fn show_action_buttons(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    col_name: &str,
    config: &SuppColumnConfig,
) {
    ui.label(
        RichText::new(format!("{} Actions", egui_phosphor::regular::CURSOR_CLICK))
            .strong()
            .weak(),
    );
    ui.separator();
    ui.add_space(spacing::SM);

    ui.horizontal(|ui| {
        // Add to SUPP button - starts editing mode
        let is_added = config.action == SuppAction::AddToSupp;
        if ui
            .add_enabled(
                !is_added,
                egui::Button::new(RichText::new(format!(
                    "{} Add to SUPP",
                    egui_phosphor::regular::PLUS
                ))),
            )
            .clicked()
        {
            // Start editing mode with current values
            state.ui.domain_editor(domain_code).supp.start_editing(
                col_name,
                &config.qnam,
                &config.qlabel,
            );
        }

        // Skip button
        let is_skipped = config.action == SuppAction::Skip;
        if ui
            .add_enabled(
                !is_skipped,
                egui::Button::new(RichText::new(format!(
                    "{} Skip",
                    egui_phosphor::regular::MINUS
                ))),
            )
            .clicked()
        {
            apply_supp_action_change(state, domain_code, col_name, SuppAction::Skip, "", "");
        }

        // Reset button (if not pending)
        if config.action != SuppAction::Pending {
            if ui
                .button(RichText::new(format!(
                    "{} Reset",
                    egui_phosphor::regular::ARROW_COUNTER_CLOCKWISE
                )))
                .clicked()
            {
                apply_supp_action_change(state, domain_code, col_name, SuppAction::Pending, "", "");
            }
        }
    });
}

/// Show editing form (when adding to SUPP)
fn show_editing_form(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    col_name: &str,
    _config: &SuppColumnConfig,
) {
    ui.label(
        RichText::new(format!(
            "{} Configure SUPP Entry",
            egui_phosphor::regular::PENCIL
        ))
        .strong()
        .weak(),
    );
    ui.separator();
    ui.add_space(spacing::SM);

    // Get editing state values (we need to clone them to avoid borrow issues)
    let (current_qnam, current_qlabel) = state
        .ui
        .get_domain_editor(domain_code)
        .and_then(|ui| ui.supp.editing_for(col_name))
        .map(|e| (e.qnam.clone(), e.qlabel.clone()))
        .unwrap_or_default();

    let mut qnam = current_qnam;
    let mut qlabel = current_qlabel;

    // QNAM input
    ui.horizontal(|ui| {
        ui.label(RichText::new("QNAM:").strong());
        ui.label(RichText::new(egui_phosphor::regular::INFO).color(Color32::BLUE))
            .on_hover_text(
                "Maximum 8 characters\nUppercase letters and numbers only\nCannot start with a number",
            );
    });

    let qnam_response = ui.add(
        egui::TextEdit::singleline(&mut qnam)
            .char_limit(8)
            .desired_width(100.0)
            .font(egui::TextStyle::Monospace)
            .hint_text("e.g. DMEXTRA"),
    );

    // Auto-uppercase QNAM
    if qnam_response.changed() {
        qnam = qnam.to_uppercase();
    }

    // Validate QNAM and show character counter
    let qnam_validation = validate_qnam(&qnam);
    let qnam_len = qnam.len();
    ui.horizontal(|ui| {
        // Character counter
        let counter_color = if qnam_len > 8 {
            ui.visuals().error_fg_color
        } else if qnam_len == 8 {
            ui.visuals().warn_fg_color
        } else {
            ui.visuals().weak_text_color()
        };
        ui.label(
            RichText::new(format!("{}/8", qnam_len))
                .small()
                .color(counter_color),
        );

        // Validation status
        if qnam_validation.is_ok() && !qnam.is_empty() {
            ui.label(
                RichText::new(egui_phosphor::regular::CHECK)
                    .small()
                    .color(Color32::from_rgb(100, 180, 100)),
            );
        } else if let Err(err) = &qnam_validation {
            ui.label(
                RichText::new(err)
                    .small()
                    .color(ui.visuals().error_fg_color),
            );
        }
    });

    ui.add_space(spacing::SM);

    // QLABEL input
    ui.horizontal(|ui| {
        ui.label(RichText::new("QLABEL:").strong());
        ui.label(RichText::new(egui_phosphor::regular::INFO).color(Color32::BLUE))
            .on_hover_text("Maximum 40 characters\nRequired - describes the variable");
    });

    let qlabel_response = ui.add(
        egui::TextEdit::singleline(&mut qlabel)
            .char_limit(40)
            .desired_width(ui.available_width() - 20.0)
            .hint_text("Descriptive label for the variable"),
    );

    // Validate QLABEL and show character counter
    let qlabel_len = qlabel.len();
    let qlabel_valid = !qlabel.trim().is_empty() && qlabel_len <= 40;
    ui.horizontal(|ui| {
        // Character counter
        let counter_color = if qlabel_len > 40 {
            ui.visuals().error_fg_color
        } else if qlabel_len >= 35 {
            ui.visuals().warn_fg_color
        } else {
            ui.visuals().weak_text_color()
        };
        ui.label(
            RichText::new(format!("{}/40", qlabel_len))
                .small()
                .color(counter_color),
        );

        // Validation status
        if qlabel_valid {
            ui.label(
                RichText::new(egui_phosphor::regular::CHECK)
                    .small()
                    .color(Color32::from_rgb(100, 180, 100)),
            );
        } else if qlabel.trim().is_empty() {
            ui.label(
                RichText::new("Required")
                    .small()
                    .color(ui.visuals().error_fg_color),
            );
        }
    });

    ui.add_space(spacing::MD);

    // Update editing state if values changed
    if qnam_response.changed() || qlabel_response.changed() {
        if let Some(editing) = state
            .ui
            .domain_editor(domain_code)
            .supp
            .editing_for_mut(col_name)
        {
            editing.qnam = qnam.clone();
            editing.qlabel = qlabel.clone();
        }
    }

    // Action buttons
    ui.horizontal(|ui| {
        let can_confirm = qnam_validation.is_ok() && qlabel_valid;

        // Confirm button
        if ui
            .add_enabled(
                can_confirm,
                egui::Button::new(
                    RichText::new(format!("{} Confirm", egui_phosphor::regular::CHECK))
                        .color(Color32::from_rgb(100, 180, 100)),
                ),
            )
            .clicked()
        {
            // Apply the change
            apply_supp_action_change(
                state,
                domain_code,
                col_name,
                SuppAction::AddToSupp,
                &qnam,
                &qlabel,
            );
            // Clear editing state
            state.ui.domain_editor(domain_code).supp.cancel_editing();
        }

        // Cancel button
        if ui
            .button(RichText::new(format!(
                "{} Cancel",
                egui_phosphor::regular::X
            )))
            .clicked()
        {
            state.ui.domain_editor(domain_code).supp.cancel_editing();
        }
    });
}

/// Get icon and color for action
fn action_icon_color(action: SuppAction, ui: &Ui) -> (&'static str, Color32) {
    match action {
        SuppAction::Pending => (
            egui_phosphor::regular::CIRCLE_DASHED,
            ui.visuals().warn_fg_color,
        ),
        SuppAction::AddToSupp => (
            egui_phosphor::regular::CHECK,
            Color32::from_rgb(100, 180, 100),
        ),
        SuppAction::Skip => (
            egui_phosphor::regular::MINUS,
            ui.visuals().weak_text_color(),
        ),
    }
}

/// Apply a SUPP action change with QNAM and QLABEL
fn apply_supp_action_change(
    state: &mut AppState,
    domain_code: &str,
    column_name: &str,
    new_action: SuppAction,
    qnam: &str,
    qlabel: &str,
) {
    if let Some(domain) = state
        .study_mut()
        .and_then(|s| s.get_domain_mut(domain_code))
    {
        if let Some(supp) = domain.derived.supp_mut() {
            if let Some(config) = supp.get_mut(column_name) {
                config.action = new_action;
                if new_action == SuppAction::AddToSupp {
                    config.qnam = qnam.to_string();
                    config.qlabel = qlabel.to_string();
                } else if new_action == SuppAction::Pending {
                    // Reset to suggested QNAM, clear QLABEL
                    config.qnam = suggest_qnam(column_name, domain_code);
                    config.qlabel = String::new();
                }
            }
        }
    }
}

/// Rebuild the SUPP configuration
fn rebuild_supp_config(state: &mut AppState, domain_code: &str) {
    let supp_config = {
        let Some(study) = state.study() else {
            return;
        };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };

        // Get unmapped source columns
        // These are columns in the source data that aren't mapped to any SDTM variable
        let source_columns: std::collections::BTreeSet<String> =
            domain.source.columns().into_iter().collect();

        // Get mapped columns
        let mapped_columns: std::collections::BTreeSet<String> = domain
            .mapping
            .all_accepted()
            .values()
            .map(|(col, _)| col.clone())
            .collect();

        // Unmapped = source - mapped
        let unmapped: Vec<String> = source_columns
            .difference(&mapped_columns)
            .cloned()
            .collect();

        SuppConfig::from_unmapped(&unmapped, domain_code)
    };

    // Store the result in derived state (directly, no versioning)
    if let Some(domain) = state
        .study_mut()
        .and_then(|s| s.get_domain_mut(domain_code))
    {
        domain.derived.supp = Some(supp_config);
    }
}
