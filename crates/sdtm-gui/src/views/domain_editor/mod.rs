//! Domain editor view
//!
//! Main editing interface with tabs: Mapping, Transform, Validation, Preview, SUPP.
//! Each tab is implemented in its own module for maintainability.

mod mapping;
mod preview;
mod supp;
mod transform;
mod validation;

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
            if ui
                .button(format!("{} Back", egui_phosphor::regular::ARROW_LEFT))
                .clicked()
            {
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

        // Tab content - delegate to submodules
        match active_tab {
            EditorTab::Mapping => mapping::show(ui, state, domain_code),
            EditorTab::Transform => transform::show(ui, state, domain_code),
            EditorTab::Validation => validation::show(ui, state, domain_code),
            EditorTab::Preview => preview::show(ui, state, domain_code),
            EditorTab::Supp => supp::show(ui, state, domain_code),
        }
    }
}
