//! Domain editor view
//!
//! Main editing interface with tabs: Mapping, Transform, Validation, Preview, SUPP.

use crate::state::{AppState, EditorTab};
use crate::theme::{colors, spacing};
use egui::{RichText, Ui};

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

    fn show_mapping_tab(ui: &mut Ui, _state: &mut AppState, domain_code: &str) {
        ui.label(format!("Mapping tab for {} - TODO", domain_code));
        ui.label("Map source columns to SDTM variables");

        // Placeholder for mapping interface
        ui.add_space(spacing::MD);
        ui.group(|ui| {
            ui.label("Source Columns → SDTM Variables");
            ui.label("(Implementation pending)");
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
}
