//! Settings window UI implementation.
//!
//! Provides a settings window with tabbed categories:
//! - General (dark mode, CT version)
//! - Validation (mode, XPT version, custom rules)
//! - Developer (bypass rules, allow export with errors)
//! - Export (default directory, format)
//! - Display (preview rows, decimal precision)
//! - Shortcuts (key bindings)

use super::{
    CtVersionSetting, DeveloperSettings, DisplaySettings, ExportFormat, ExportSettings,
    GeneralSettings, PreviewRowLimit, Settings, ShortcutAction, ValidationModeSetting,
    ValidationSettings, XptValidationRule, XptVersionSetting,
};
use eframe::egui::{self, CornerRadius, Vec2};
use tss_validate::rules::Category;

/// Settings category tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsCategory {
    #[default]
    General,
    Validation,
    Developer,
    Export,
    Display,
    Shortcuts,
}

impl SettingsCategory {
    /// Get all categories.
    pub const fn all() -> &'static [SettingsCategory] {
        &[
            Self::General,
            Self::Validation,
            Self::Developer,
            Self::Export,
            Self::Display,
            Self::Shortcuts,
        ]
    }

    /// Get the display name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Validation => "Validation",
            Self::Developer => "Developer",
            Self::Export => "Export",
            Self::Display => "Display",
            Self::Shortcuts => "Shortcuts",
        }
    }

    /// Get the icon.
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::General => egui_phosphor::regular::GEAR,
            Self::Validation => egui_phosphor::regular::CHECK_SQUARE,
            Self::Developer => egui_phosphor::regular::CODE,
            Self::Export => egui_phosphor::regular::EXPORT,
            Self::Display => egui_phosphor::regular::EYE,
            Self::Shortcuts => egui_phosphor::regular::KEYBOARD,
        }
    }
}

/// Result of showing the settings window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsResult {
    /// Keep the window open.
    Open,
    /// Apply changes and close.
    Apply,
    /// Cancel changes and close.
    Cancel,
}

/// Settings window state.
pub struct SettingsWindow {
    /// Currently selected category.
    category: SettingsCategory,
    /// Buffer for new rule ID input.
    new_rule_id_input: String,
}

impl Default for SettingsWindow {
    fn default() -> Self {
        Self {
            category: SettingsCategory::General,
            new_rule_id_input: String::new(),
        }
    }
}

impl SettingsWindow {
    /// Show the settings as a separate native window using viewports.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        settings: &mut Settings,
        _dark_mode: bool,
    ) -> SettingsResult {
        let mut result = SettingsResult::Open;

        // Use a native window viewport for settings
        let viewport_id = egui::ViewportId::from_hash_of("settings_window");

        ctx.show_viewport_immediate(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title("Settings")
                .with_inner_size([720.0, 520.0])
                .with_min_inner_size([600.0, 400.0])
                .with_resizable(true),
            |ctx, _class| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
                    .show(ctx, |ui| {
                        result = self.show_layout(ui, settings);
                    });

                // Handle window close
                if ctx.input(|i| i.viewport().close_requested()) {
                    result = SettingsResult::Cancel;
                }
            },
        );

        result
    }

    /// Show the layout with sidebar.
    fn show_layout(&mut self, ui: &mut egui::Ui, settings: &mut Settings) -> SettingsResult {
        let mut result = SettingsResult::Open;
        let available_rect = ui.available_rect_before_wrap();

        // Calculate layout dimensions
        let sidebar_width = 180.0;
        let button_bar_height = 56.0;

        // Draw sidebar background
        let sidebar_rect = egui::Rect::from_min_size(
            available_rect.min,
            Vec2::new(sidebar_width, available_rect.height()),
        );
        ui.painter().rect_filled(
            sidebar_rect,
            CornerRadius::ZERO,
            ui.visuals().faint_bg_color,
        );

        // Main vertical layout
        ui.vertical(|ui| {
            // Top area: sidebar + content
            let content_height = available_rect.height() - button_bar_height;

            ui.horizontal(|ui| {
                ui.set_height(content_height);

                // Left sidebar
                ui.allocate_ui_with_layout(
                    Vec2::new(sidebar_width, content_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.show_sidebar_content(ui);
                    },
                );

                // Separator line
                let sep_rect = egui::Rect::from_min_size(
                    egui::pos2(sidebar_width, available_rect.min.y),
                    Vec2::new(1.0, content_height),
                );
                ui.painter().rect_filled(
                    sep_rect,
                    0.0,
                    ui.visuals().widgets.inactive.bg_stroke.color,
                );

                // Right content area
                ui.vertical(|ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add_space(20.0);
                            ui.horizontal(|ui| {
                                ui.add_space(24.0);
                                ui.vertical(|ui| {
                                    ui.set_max_width(480.0);
                                    match self.category {
                                        SettingsCategory::General => {
                                            self.show_general(ui, &mut settings.general)
                                        }
                                        SettingsCategory::Validation => {
                                            self.show_validation(ui, &mut settings.validation)
                                        }
                                        SettingsCategory::Developer => {
                                            self.show_developer(ui, &mut settings.developer)
                                        }
                                        SettingsCategory::Export => {
                                            self.show_export(ui, &mut settings.export)
                                        }
                                        SettingsCategory::Display => {
                                            self.show_display(ui, &mut settings.display)
                                        }
                                        SettingsCategory::Shortcuts => {
                                            self.show_shortcuts(ui, settings)
                                        }
                                    }
                                    ui.add_space(20.0);
                                });
                            });
                        });
                });
            });

            // Bottom button bar - fixed at bottom
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.add_space(16.0);

                // Reset button on left
                if ui.button("Reset to Defaults").clicked() {
                    *settings = Settings::default();
                }

                // Spacer to push buttons to the right
                ui.add_space(ui.available_width() - 200.0);

                // Cancel button
                if ui
                    .add(egui::Button::new("Cancel").min_size(Vec2::new(80.0, 28.0)))
                    .clicked()
                {
                    result = SettingsResult::Cancel;
                }

                ui.add_space(8.0);

                // Apply button (primary action)
                if ui
                    .add(
                        egui::Button::new("Apply")
                            .fill(ui.visuals().selection.bg_fill)
                            .min_size(Vec2::new(80.0, 28.0)),
                    )
                    .clicked()
                {
                    result = SettingsResult::Apply;
                }

                ui.add_space(16.0);
            });

            ui.add_space(12.0);
        });

        result
    }

    /// Show sidebar content (navigation items).
    fn show_sidebar_content(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);

        for category in SettingsCategory::all() {
            let selected = self.category == *category;

            let text_color = if selected {
                ui.visuals().hyperlink_color
            } else {
                ui.visuals().text_color()
            };

            let bg_fill = if selected {
                ui.visuals().selection.bg_fill
            } else {
                egui::Color32::TRANSPARENT
            };

            let button = egui::Button::new(
                egui::RichText::new(format!("{} {}", category.icon(), category.name()))
                    .color(text_color),
            )
            .fill(bg_fill)
            .stroke(egui::Stroke::NONE)
            .min_size(Vec2::new(160.0, 32.0));

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if ui.add(button).clicked() {
                    self.category = *category;
                }
            });
        }
    }

    /// Show a section header.
    fn section_header(&self, ui: &mut egui::Ui, title: &str) {
        ui.label(egui::RichText::new(title).size(20.0).strong());
        ui.add_space(16.0);
    }

    /// Show a setting row with label and widget.
    fn setting_row<R>(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        description: Option<&str>,
        add_widget: impl FnOnce(&mut egui::Ui) -> R,
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_min_width(200.0);
                ui.label(label);
                if let Some(desc) = description {
                    ui.label(egui::RichText::new(desc).small().weak());
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                add_widget(ui);
            });
        });
        ui.add_space(12.0);
    }

    /// Show a group box for related settings.
    fn setting_group(
        &self,
        ui: &mut egui::Ui,
        title: Option<&str>,
        add_content: impl FnOnce(&mut egui::Ui),
    ) {
        if let Some(t) = title {
            ui.label(egui::RichText::new(t).size(13.0).strong().weak());
            ui.add_space(8.0);
        }

        egui::Frame::default()
            .fill(ui.visuals().faint_bg_color)
            .corner_radius(CornerRadius::same(8))
            .inner_margin(16.0)
            .show(ui, |ui| {
                add_content(ui);
            });

        ui.add_space(20.0);
    }

    /// Show general settings.
    fn show_general(&self, ui: &mut egui::Ui, general: &mut GeneralSettings) {
        self.section_header(ui, "General");

        self.setting_group(ui, Some("APPEARANCE"), |ui| {
            self.setting_row(ui, "Dark Mode", Some("Use dark color scheme"), |ui| {
                ui.checkbox(&mut general.dark_mode, "");
            });
        });

        self.setting_group(ui, Some("DATA IMPORT"), |ui| {
            self.setting_row(
                ui,
                "Controlled Terminology",
                Some("CDISC CT version for validation"),
                |ui| {
                    egui::ComboBox::from_id_salt("ct_version")
                        .width(180.0)
                        .selected_text(general.ct_version.display_name())
                        .show_ui(ui, |ui| {
                            for version in CtVersionSetting::all() {
                                ui.selectable_value(
                                    &mut general.ct_version,
                                    *version,
                                    version.display_name(),
                                );
                            }
                        });
                },
            );

            ui.separator();
            ui.add_space(8.0);

            self.setting_row(
                ui,
                "CSV Header Rows",
                Some("Number of header rows to skip"),
                |ui| {
                    ui.add(
                        egui::DragValue::new(&mut general.header_rows)
                            .range(1..=10)
                            .speed(0.1),
                    );
                },
            );
        });
    }

    /// Show validation settings.
    fn show_validation(&self, ui: &mut egui::Ui, validation: &mut ValidationSettings) {
        self.section_header(ui, "Validation");

        self.setting_group(ui, Some("VALIDATION MODE"), |ui| {
            for mode in ValidationModeSetting::all() {
                let selected = validation.mode == *mode;
                if ui
                    .add(egui::RadioButton::new(selected, mode.display_name()))
                    .clicked()
                {
                    validation.mode = *mode;
                }
                ui.label(egui::RichText::new(mode.description()).small().weak());
                ui.add_space(8.0);
            }
        });

        self.setting_group(ui, Some("XPT FORMAT"), |ui| {
            self.setting_row(
                ui,
                "XPT Version",
                Some("Transport file format version"),
                |ui| {
                    egui::ComboBox::from_id_salt("xpt_version")
                        .width(200.0)
                        .selected_text(validation.xpt_version.display_name())
                        .show_ui(ui, |ui| {
                            for version in XptVersionSetting::all() {
                                ui.selectable_value(
                                    &mut validation.xpt_version,
                                    *version,
                                    format!(
                                        "{} - {}",
                                        version.display_name(),
                                        version.description()
                                    ),
                                );
                            }
                        });
                },
            );
        });

        // Custom rules (only shown when Custom mode is selected)
        if validation.mode == ValidationModeSetting::Custom {
            self.setting_group(ui, Some("ENABLED RULES"), |ui| {
                for rule in XptValidationRule::all() {
                    let mut enabled = validation.custom_enabled_rules.contains(rule);
                    let label = if rule.is_fda_only() {
                        format!("{} (FDA)", rule.display_name())
                    } else {
                        rule.display_name().to_string()
                    };

                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut enabled, label).changed() {
                            if enabled {
                                validation.custom_enabled_rules.insert(*rule);
                            } else {
                                validation.custom_enabled_rules.remove(rule);
                            }
                        }
                    });
                    ui.label(egui::RichText::new(rule.description()).small().weak());
                    ui.add_space(4.0);
                }
            });
        }
    }

    /// Show developer settings.
    fn show_developer(&mut self, ui: &mut egui::Ui, developer: &mut DeveloperSettings) {
        self.section_header(ui, "Developer");

        self.setting_group(ui, None, |ui| {
            ui.label(
                egui::RichText::new(
                    "Developer mode allows you to bypass validation checks for testing.",
                )
                .weak(),
            );
            ui.add_space(12.0);

            self.setting_row(ui, "Developer Mode", None, |ui| {
                ui.checkbox(&mut developer.enabled, "");
            });
        });

        if developer.enabled {
            self.setting_group(ui, Some("OPTIONS"), |ui| {
                self.setting_row(
                    ui,
                    "Incomplete Mappings",
                    Some("Allow export with missing required mappings"),
                    |ui| {
                        ui.checkbox(&mut developer.allow_incomplete_mappings, "");
                    },
                );

                self.setting_row(
                    ui,
                    "Export with Errors",
                    Some("Allow export even with validation errors"),
                    |ui| {
                        ui.checkbox(&mut developer.allow_export_with_errors, "");
                    },
                );

                ui.separator();
                ui.add_space(8.0);

                self.setting_row(
                    ui,
                    "Debug Info",
                    Some("Show debug information in UI"),
                    |ui| {
                        ui.checkbox(&mut developer.show_debug_info, "");
                    },
                );
            });

            self.setting_group(ui, Some("BYPASS CATEGORIES"), |ui| {
                ui.label(
                    egui::RichText::new(
                        "Selected validation categories will be bypassed during export.",
                    )
                    .small()
                    .weak(),
                );
                ui.add_space(8.0);

                for category in Category::all() {
                    let mut bypassed = developer.bypassed_categories.contains(category);
                    if ui
                        .checkbox(&mut bypassed, category.display_name())
                        .changed()
                    {
                        if bypassed {
                            developer.bypassed_categories.insert(*category);
                        } else {
                            developer.bypassed_categories.remove(category);
                        }
                    }
                    ui.label(egui::RichText::new(category.description()).small().weak());
                    ui.add_space(4.0);
                }
            });

            // BYPASS RULE IDS section (inlined to avoid borrow conflict with self.new_rule_id_input)
            ui.label(
                egui::RichText::new("BYPASS RULE IDS")
                    .size(13.0)
                    .strong()
                    .weak(),
            );
            ui.add_space(8.0);

            egui::Frame::default()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Enter rule IDs to bypass (e.g., SD0056, SD0002):")
                            .small()
                            .weak(),
                    );
                    ui.add_space(8.0);

                    // Show current bypassed rules
                    let ids: Vec<_> = developer.bypassed_rule_ids.iter().cloned().collect();
                    for id in ids {
                        ui.horizontal(|ui| {
                            ui.label(&id);
                            if ui.small_button("×").clicked() {
                                developer.bypassed_rule_ids.remove(&id);
                            }
                        });
                    }

                    // Add new rule ID
                    ui.horizontal(|ui| {
                        ui.label("Add:");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.new_rule_id_input)
                                .desired_width(100.0)
                                .hint_text("e.g. SD0056"),
                        );

                        let should_add = ui.button("+").clicked()
                            || (response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter)));

                        if should_add && !self.new_rule_id_input.trim().is_empty() {
                            let rule_id = self.new_rule_id_input.trim().to_uppercase();
                            developer.bypassed_rule_ids.insert(rule_id);
                            self.new_rule_id_input.clear();
                        }
                    });
                });
            ui.add_space(20.0);
        }
    }

    /// Show export settings.
    fn show_export(&self, ui: &mut egui::Ui, export: &mut ExportSettings) {
        self.section_header(ui, "Export");

        self.setting_group(ui, Some("OUTPUT"), |ui| {
            ui.horizontal(|ui| {
                ui.label("Default Directory:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if export.default_output_dir.is_some() && ui.button("Clear").clicked() {
                        export.default_output_dir = None;
                    }
                    if ui.button("Browse...").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            export.default_output_dir = Some(folder);
                        }
                    }
                });
            });

            if let Some(ref dir) = export.default_output_dir {
                ui.label(
                    egui::RichText::new(dir.display().to_string())
                        .small()
                        .weak(),
                );
            } else {
                ui.label(
                    egui::RichText::new("Uses study folder by default")
                        .small()
                        .weak(),
                );
            }
        });

        self.setting_group(ui, Some("DATA FORMAT"), |ui| {
            for format in ExportFormat::all() {
                let selected = export.default_format == *format;
                if ui
                    .add(egui::RadioButton::new(selected, format.display_name()))
                    .clicked()
                {
                    export.default_format = *format;
                }
                ui.label(egui::RichText::new(format.description()).small().weak());
                ui.add_space(4.0);
            }

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Define-XML is always generated with exports")
                    .small()
                    .weak(),
            );
        });

        self.setting_group(ui, Some("FILE OPTIONS"), |ui| {
            self.setting_row(ui, "Filename Template", None, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut export.filename_template).desired_width(150.0),
                );
            });
            ui.label(
                egui::RichText::new("Use {domain} for domain code, {studyid} for study ID")
                    .small()
                    .weak(),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            self.setting_row(
                ui,
                "Overwrite Files",
                Some("Skip confirmation when overwriting"),
                |ui| {
                    ui.checkbox(&mut export.overwrite_without_prompt, "");
                },
            );
        });
    }

    /// Show display settings.
    fn show_display(&self, ui: &mut egui::Ui, display: &mut DisplaySettings) {
        self.section_header(ui, "Display");

        self.setting_group(ui, Some("DATA PREVIEW"), |ui| {
            self.setting_row(
                ui,
                "Row Limit",
                Some("Maximum rows shown in preview"),
                |ui| {
                    egui::ComboBox::from_id_salt("preview_rows")
                        .width(120.0)
                        .selected_text(display.max_preview_rows.display_name())
                        .show_ui(ui, |ui| {
                            for limit in PreviewRowLimit::all() {
                                ui.selectable_value(
                                    &mut display.max_preview_rows,
                                    *limit,
                                    limit.display_name(),
                                );
                            }
                        });
                },
            );

            ui.separator();
            ui.add_space(8.0);

            self.setting_row(
                ui,
                "Decimal Precision",
                Some("Digits after decimal point"),
                |ui| {
                    ui.add(
                        egui::DragValue::new(&mut display.decimal_precision)
                            .range(0..=10)
                            .speed(0.1),
                    );
                },
            );

            ui.separator();
            ui.add_space(8.0);

            self.setting_row(
                ui,
                "Truncate Text",
                Some("Maximum characters before truncation"),
                |ui| {
                    ui.add(
                        egui::DragValue::new(&mut display.truncate_long_text)
                            .range(10..=500)
                            .speed(1.0),
                    );
                },
            );
        });

        self.setting_group(ui, Some("TABLE OPTIONS"), |ui| {
            self.setting_row(
                ui,
                "Row Numbers",
                Some("Show row numbers in data tables"),
                |ui| {
                    ui.checkbox(&mut display.show_row_numbers, "");
                },
            );
        });
    }

    /// Show shortcuts settings.
    fn show_shortcuts(&self, ui: &mut egui::Ui, settings: &mut Settings) {
        self.section_header(ui, "Keyboard Shortcuts");

        self.setting_group(ui, None, |ui| {
            ui.label(
                egui::RichText::new("Current keyboard shortcuts (read-only in this version)")
                    .weak(),
            );
            ui.add_space(12.0);

            egui::Grid::new("shortcuts_grid")
                .num_columns(2)
                .spacing([40.0, 12.0])
                .show(ui, |ui| {
                    for action in ShortcutAction::all() {
                        ui.label(action.display_name());
                        if let Some(binding) = settings.shortcuts.bindings.get(action) {
                            ui.label(egui::RichText::new(binding.display()).monospace().weak());
                        } else {
                            ui.label(egui::RichText::new("-").weak());
                        }
                        ui.end_row();
                    }
                });
        });
    }
}
