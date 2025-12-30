//! SUPP tab
//!
//! Configure Supplemental Qualifiers for unmapped source columns.

use crate::state::AppState;
use crate::theme::spacing;
use egui::Ui;

/// Render the SUPP tab
pub fn show(ui: &mut Ui, _state: &mut AppState, domain_code: &str) {
    ui.label(format!(
        "{} SUPP tab for {} - Coming Soon",
        egui_phosphor::regular::PLUS_SQUARE,
        domain_code
    ));
    ui.label("Configure Supplemental Qualifiers");

    ui.add_space(spacing::MD);
    ui.group(|ui| {
        ui.label("SUPPQUAL Configuration");
        ui.label("(Implementation pending)");
    });
}
