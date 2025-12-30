//! Preview tab
//!
//! Shows transformed data before export.

use crate::state::AppState;
use crate::theme::spacing;
use egui::Ui;

/// Render the preview tab
pub fn show(ui: &mut Ui, _state: &mut AppState, domain_code: &str) {
    ui.label(format!(
        "{} Preview tab for {} - Coming Soon",
        egui_phosphor::regular::EYE,
        domain_code
    ));
    ui.label("Preview processed SDTM output");

    ui.add_space(spacing::MD);
    ui.group(|ui| {
        ui.label("Output Preview");
        ui.label("(Implementation pending)");
    });
}
