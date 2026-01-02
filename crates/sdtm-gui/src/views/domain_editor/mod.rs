//! Domain editor view
//!
//! Main editing interface with tabs: Mapping, Transform, Validation, Preview, SUPP.
//! Each tab is implemented in its own module for maintainability.

mod mapping;
mod preview;
mod supp;
mod transform;
mod validation;

use crate::state::{AppState, DomainInitState, EditorTab};
use crate::theme::spacing;
use egui::{RichText, Ui};

/// Ensure domain mapping is initialized, showing loading UI if needed.
/// Returns true if ready to render tab content, false if still loading.
pub fn ensure_mapping_initialized(ui: &mut Ui, state: &mut AppState, domain_code: &str) -> bool {
    let init_state = state
        .study
        .as_mut()
        .map(|s| s.check_domain_init(domain_code))
        .unwrap_or(DomainInitState::Error);

    match init_state {
        DomainInitState::Ready => true,
        DomainInitState::StartLoading => {
            show_loading_spinner(ui);
            ui.ctx().request_repaint();
            false
        }
        DomainInitState::DoInitialize => {
            show_loading_spinner(ui);
            mapping::initialize_mapping(state, domain_code);
            ui.ctx().request_repaint();
            false
        }
        DomainInitState::Error => {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} Failed to initialize mapping",
                        egui_phosphor::regular::WARNING
                    ))
                    .color(ui.visuals().error_fg_color),
                );
            });
            false
        }
    }
}

/// Show loading spinner with message
fn show_loading_spinner(ui: &mut Ui) {
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

/// Domain editor view
pub struct DomainEditorView;

impl DomainEditorView {
    /// Render the domain editor
    pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str, active_tab: EditorTab) {
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
                        .weak(),
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
