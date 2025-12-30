//! Domain editor view
//!
//! Main editing interface with tabs: Mapping, Transform, Validation, Preview, SUPP.

use crate::services::{MappingService, MappingState};
use crate::state::{AppState, DomainStatus, EditorTab};
use crate::theme::{colors, spacing};
use egui::{RichText, Ui};
use sdtm_map::ConfidenceLevel;
use sdtm_standards::load_default_sdtm_ig_domains;
use std::collections::BTreeMap;

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

    fn show_mapping_tab(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
        let theme = colors(state.preferences.dark_mode);

        // Get study and domain info
        let (study_id, source_columns, has_mapping_state) = {
            if let Some(study) = &state.study {
                if let Some(domain) = study.get_domain(domain_code) {
                    (
                        study.study_id.clone(),
                        domain.source_columns(),
                        domain.mapping_state.is_some(),
                    )
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        // Initialize mapping state if needed
        if !has_mapping_state {
            if ui.button("Generate Mapping Suggestions").clicked() {
                // Load SDTM domain definition
                if let Ok(domains) = load_default_sdtm_ig_domains() {
                    if let Some(sdtm_domain) = domains.into_iter().find(|d| d.code == domain_code) {
                        // Generate mapping suggestions
                        let hints = if let Some(study) = &state.study {
                            if let Some(domain) = study.get_domain(domain_code) {
                                MappingService::extract_column_hints(&domain.source_data)
                            } else {
                                BTreeMap::new()
                            }
                        } else {
                            BTreeMap::new()
                        };

                        let mapping_state = MappingService::create_mapping_state(
                            &sdtm_domain,
                            &study_id,
                            &source_columns,
                            hints,
                        );

                        // Store the mapping state
                        if let Some(study) = &mut state.study {
                            if let Some(domain) = study.get_domain_mut(domain_code) {
                                domain.mapping_state = Some(mapping_state);
                                domain.status = DomainStatus::MappingInProgress;
                            }
                        }
                    }
                }
            }

            ui.add_space(spacing::MD);
            ui.label("Click the button above to analyze source columns and suggest mappings.");

            // Show source columns
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Source Columns").strong());
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for col in &source_columns {
                        ui.label(format!("  • {}", col));
                    }
                });
            return;
        }

        // Show mapping interface
        Self::show_mapping_interface(ui, state, domain_code, &theme);
    }

    fn show_mapping_interface(
        ui: &mut Ui,
        state: &mut AppState,
        domain_code: &str,
        theme: &crate::theme::ThemeColors,
    ) {
        // Collect mapping info to avoid borrowing issues
        let mapping_info = if let Some(study) = &state.study {
            if let Some(domain) = study.get_domain(domain_code) {
                domain.mapping_state.as_ref().map(|ms| {
                    let summary = ms.summary();
                    let pending_by_level = ms.pending_by_level();
                    let accepted: Vec<_> = ms.accepted.iter().map(|m| {
                        (m.source_column.clone(), m.target_variable.clone(), m.confidence)
                    }).collect();
                    let pending: Vec<_> = ms.pending.iter().map(|m| {
                        (m.source_column.clone(), m.target_variable.clone(), m.confidence)
                    }).collect();
                    let unmapped = ms.unmapped.clone();
                    let high_count = pending_by_level.get(&ConfidenceLevel::High).map(|v| v.len()).unwrap_or(0);
                    (summary, accepted, pending, unmapped, high_count)
                })
            } else {
                None
            }
        } else {
            None
        };

        let Some((summary, accepted, pending, unmapped, high_count)) = mapping_info else {
            return;
        };

        // Summary bar
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("✓ {} accepted", summary.accepted))
                    .color(theme.success),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("○ {} pending", summary.pending))
                    .color(theme.warning),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("✕ {} unmapped", summary.unmapped))
                    .color(theme.text_muted),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if high_count > 0 && ui.button(format!("Accept all high ({high_count})")).clicked() {
                    if let Some(study) = &mut state.study {
                        if let Some(domain) = study.get_domain_mut(domain_code) {
                            if let Some(ms) = &mut domain.mapping_state {
                                ms.accept_all_above(ConfidenceLevel::High);
                            }
                        }
                    }
                }
            });
        });

        ui.add_space(spacing::SM);
        ui.separator();
        ui.add_space(spacing::SM);

        // Track actions to perform after display
        let mut accept_column: Option<String> = None;
        let mut reject_column: Option<String> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Accepted mappings
            if !accepted.is_empty() {
                ui.collapsing(
                    RichText::new(format!("Accepted ({})", accepted.len())).strong(),
                    |ui| {
                        for (source, target, conf) in &accepted {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("✓").color(theme.success));
                                ui.label(source);
                                ui.label("→");
                                ui.label(RichText::new(target).strong());
                                ui.label(
                                    RichText::new(format!("{:.0}%", conf * 100.0))
                                        .color(theme.text_muted)
                                        .small(),
                                );
                            });
                        }
                    },
                );
                ui.add_space(spacing::SM);
            }

            // Pending mappings
            if !pending.is_empty() {
                ui.label(RichText::new(format!("Pending ({})", pending.len())).strong());
                ui.add_space(spacing::SM);

                for (source, target, conf) in &pending {
                    ui.horizontal(|ui| {
                        // Confidence indicator
                        let conf_color = if *conf >= 0.95 {
                            theme.success
                        } else if *conf >= 0.80 {
                            theme.accent
                        } else {
                            theme.warning
                        };

                        ui.label(RichText::new("○").color(conf_color));
                        ui.label(source);
                        ui.label("→");
                        ui.label(RichText::new(target).strong());
                        ui.label(
                            RichText::new(format!("{:.0}%", conf * 100.0))
                                .color(theme.text_muted)
                                .small(),
                        );

                        // Accept/Reject buttons
                        if ui.small_button("✓").clicked() {
                            accept_column = Some(source.clone());
                        }
                        if ui.small_button("✕").clicked() {
                            reject_column = Some(source.clone());
                        }
                    });
                }
                ui.add_space(spacing::SM);
            }

            // Unmapped columns
            if !unmapped.is_empty() {
                ui.collapsing(
                    RichText::new(format!("Unmapped ({})", unmapped.len()))
                        .color(theme.text_muted),
                    |ui| {
                        for col in &unmapped {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("✕").color(theme.text_muted));
                                ui.label(col);
                                // TODO: Add manual mapping dropdown
                            });
                        }
                    },
                );
            }
        });

        // Apply actions
        if let Some(col) = accept_column {
            if let Some(study) = &mut state.study {
                if let Some(domain) = study.get_domain_mut(domain_code) {
                    if let Some(ms) = &mut domain.mapping_state {
                        ms.accept(&col);
                    }
                }
            }
        }
        if let Some(col) = reject_column {
            if let Some(study) = &mut state.study {
                if let Some(domain) = study.get_domain_mut(domain_code) {
                    if let Some(ms) = &mut domain.mapping_state {
                        ms.reject(&col);
                    }
                }
            }
        }
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
