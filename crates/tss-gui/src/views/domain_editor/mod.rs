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
use crate::theme::spacing;
use egui::{RichText, Ui};

/// Domain editor view
pub struct DomainEditorView;

impl DomainEditorView {
    /// Render the domain editor
    pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str, active_tab: EditorTab) {
        // Check if domain exists
        if !state.is_domain_accessible(domain_code) {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 3.0);
                ui.label(
                    RichText::new(format!(
                        "{} Domain Not Found",
                        egui_phosphor::regular::WARNING
                    ))
                    .size(24.0)
                    .color(ui.visuals().warn_fg_color),
                );
                ui.add_space(spacing::LG);
                if ui.button("Go to Home").clicked() {
                    state.go_home();
                }
            });
            return;
        }

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

            // Show domain info (use study.get_domain to bypass DM check - we already checked)
            if let Some(study) = state.study()
                && let Some(domain) = study.get_domain(domain_code)
            {
                let file_name = domain.source.file_name().unwrap_or("unknown");
                ui.label(
                    RichText::new(format!("{}  •  {} rows", file_name, domain.row_count())).weak(),
                );
            }
        });

        ui.add_space(spacing::SM);

        // Tab bar
        ui.horizontal(|ui| {
            for tab in EditorTab::all() {
                let is_active = *tab == active_tab;
                let text = if is_active {
                    RichText::new(tab.label())
                        .strong()
                        .color(ui.visuals().hyperlink_color)
                } else {
                    RichText::new(tab.label()).weak()
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
