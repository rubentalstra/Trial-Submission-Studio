//! Validation tab
//!
//! Shows CT validation issues that must be resolved before export.

use crate::state::AppState;
use crate::theme::spacing;
use egui::Ui;

/// Render the validation tab
pub fn show(ui: &mut Ui, _state: &mut AppState, domain_code: &str) {
    ui.label(format!(
        "{} Validation tab for {} - Coming Soon",
        egui_phosphor::regular::CHECK_SQUARE,
        domain_code
    ));
    ui.label("View validation results and fix issues");

    ui.add_space(spacing::MD);
    ui.group(|ui| {
        ui.label("Validation Issues");
        ui.label("(Implementation pending)");
    });
}
