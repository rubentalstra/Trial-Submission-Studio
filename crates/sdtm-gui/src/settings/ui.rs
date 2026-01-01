//! Settings window UI implementation.
//!
//! Provides a modal settings window with tabbed categories:
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
use crate::theme::{colors, ThemeColors};
use eframe::egui;

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
}

impl Default for SettingsWindow {
    fn default() -> Self {
        Self {
            category: SettingsCategory::General,
        }
    }
}

impl SettingsWindow {
    /// Create a new settings window.
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the settings window.
    ///
    /// Returns the result of the window interaction.
    pub fn show(&mut self, ctx: &egui::Context, settings: &mut Settings, dark_mode: bool) -> SettingsResult {
        let theme = colors(dark_mode);
        let mut result = SettingsResult::Open;

        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(true)
            .default_width(700.0)
            .default_height(500.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                result = self.show_content(ui, settings, &theme);
            });

        result
    }

    /// Show the window content.
    fn show_content(&mut self, ui: &mut egui::Ui, settings: &mut Settings, theme: &ThemeColors) -> SettingsResult {
        let mut result = SettingsResult::Open;

        // Horizontal layout: sidebar + content
        ui.horizontal(|ui| {
            // Left sidebar with category tabs
            ui.vertical(|ui| {
                ui.set_min_width(150.0);
                ui.add_space(8.0);

                for category in SettingsCategory::all() {
                    let selected = self.category == *category;
                    let button = egui::Button::new(
                        egui::RichText::new(format!("{} {}", category.icon(), category.name()))
                            .color(if selected { theme.accent } else { theme.text_primary }),
                    )
                    .fill(if selected {
                        theme.accent.linear_multiply(0.15)
                    } else {
                        egui::Color32::TRANSPARENT
                    })
                    .min_size(egui::vec2(140.0, 32.0));

                    if ui.add(button).clicked() {
                        self.category = *category;
                    }
                }
            });

            ui.separator();

            // Right content panel
            ui.vertical(|ui| {
                ui.set_min_width(400.0);
                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.category {
                        SettingsCategory::General => self.show_general(ui, &mut settings.general, theme),
                        SettingsCategory::Validation => {
                            self.show_validation(ui, &mut settings.validation, theme)
                        }
                        SettingsCategory::Developer => {
                            self.show_developer(ui, &mut settings.developer, theme)
                        }
                        SettingsCategory::Export => self.show_export(ui, &mut settings.export, theme),
                        SettingsCategory::Display => self.show_display(ui, &mut settings.display, theme),
                        SettingsCategory::Shortcuts => self.show_shortcuts(ui, settings, theme),
                    }
                });
            });
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Bottom buttons
        ui.horizontal(|ui| {
            if ui.button("Reset to Defaults").clicked() {
                *settings = Settings::default();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Apply").clicked() {
                    result = SettingsResult::Apply;
                }
                if ui.button("Cancel").clicked() {
                    result = SettingsResult::Cancel;
                }
            });
        });

        result
    }

    /// Show general settings.
    fn show_general(&self, ui: &mut egui::Ui, general: &mut GeneralSettings, theme: &ThemeColors) {
        ui.heading("General Settings");
        ui.add_space(12.0);

        // Dark mode toggle
        ui.horizontal(|ui| {
            ui.label("Theme:");
            ui.checkbox(&mut general.dark_mode, "Dark Mode");
        });

        ui.add_space(8.0);

        // CT Version
        ui.horizontal(|ui| {
            ui.label("Controlled Terminology:");
            egui::ComboBox::from_id_salt("ct_version")
                .selected_text(general.ct_version.display_name())
                .show_ui(ui, |ui| {
                    for version in CtVersionSetting::all() {
                        ui.selectable_value(&mut general.ct_version, *version, version.display_name());
                    }
                });
        });

        ui.add_space(8.0);

        // Header rows
        ui.horizontal(|ui| {
            ui.label("CSV Header Rows:");
            ui.add(egui::DragValue::new(&mut general.header_rows).range(1..=10));
        });

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Number of header rows to skip when reading source CSV files.")
                .color(theme.text_muted)
                .small(),
        );
    }

    /// Show validation settings.
    fn show_validation(&self, ui: &mut egui::Ui, validation: &mut ValidationSettings, _theme: &ThemeColors) {
        ui.heading("Validation Settings");
        ui.add_space(12.0);

        // Validation mode
        ui.label("Validation Mode:");
        ui.add_space(4.0);

        for mode in ValidationModeSetting::all() {
            let selected = validation.mode == *mode;
            if ui
                .radio(selected, format!("{} - {}", mode.display_name(), mode.description()))
                .clicked()
            {
                validation.mode = *mode;
            }
        }

        ui.add_space(12.0);

        // XPT Version
        ui.horizontal(|ui| {
            ui.label("XPT Format Version:");
            egui::ComboBox::from_id_salt("xpt_version")
                .selected_text(validation.xpt_version.display_name())
                .show_ui(ui, |ui| {
                    for version in XptVersionSetting::all() {
                        ui.selectable_value(
                            &mut validation.xpt_version,
                            *version,
                            format!("{} - {}", version.display_name(), version.description()),
                        );
                    }
                });
        });

        // Custom rules (only shown when Custom mode is selected)
        if validation.mode == ValidationModeSetting::Custom {
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label("Enabled Validation Rules:");
            ui.add_space(8.0);

            for rule in XptValidationRule::all() {
                let mut enabled = validation.custom_enabled_rules.contains(rule);
                let label = format!(
                    "{}{} - {}",
                    rule.display_name(),
                    if rule.is_fda_only() { " (FDA)" } else { "" },
                    rule.description()
                );

                if ui.checkbox(&mut enabled, label).changed() {
                    if enabled {
                        validation.custom_enabled_rules.insert(*rule);
                    } else {
                        validation.custom_enabled_rules.remove(rule);
                    }
                }
            }
        }
    }

    /// Show developer settings.
    fn show_developer(&self, ui: &mut egui::Ui, developer: &mut DeveloperSettings, theme: &ThemeColors) {
        ui.heading("Developer Settings");
        ui.add_space(12.0);

        ui.label(
            egui::RichText::new(
                "Developer mode allows you to bypass validation checks for testing purposes.",
            )
            .color(theme.text_muted),
        );

        ui.add_space(12.0);

        // Enable developer mode
        ui.checkbox(&mut developer.enabled, "Enable Developer Mode");

        if developer.enabled {
            ui.add_space(16.0);

            // Allow export with errors
            ui.checkbox(
                &mut developer.allow_export_with_errors,
                "Allow export with validation errors",
            );

            ui.add_space(8.0);

            // Show debug info
            ui.checkbox(&mut developer.show_debug_info, "Show debug information in UI");

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("Bypass Validation Rules:");
            ui.label(
                egui::RichText::new("Selected rules will be skipped during validation.")
                    .color(theme.text_muted)
                    .small(),
            );
            ui.add_space(8.0);

            for rule in XptValidationRule::all() {
                let mut bypassed = developer.bypassed_rules.contains(rule);
                let label = format!("{} - {}", rule.display_name(), rule.description());

                if ui.checkbox(&mut bypassed, label).changed() {
                    if bypassed {
                        developer.bypassed_rules.insert(*rule);
                    } else {
                        developer.bypassed_rules.remove(rule);
                    }
                }
            }
        }
    }

    /// Show export settings.
    fn show_export(&self, ui: &mut egui::Ui, export: &mut ExportSettings, theme: &ThemeColors) {
        ui.heading("Export Settings");
        ui.add_space(12.0);

        // Default output directory
        ui.horizontal(|ui| {
            ui.label("Default Output Directory:");
            if let Some(ref dir) = export.default_output_dir {
                ui.label(dir.display().to_string());
            } else {
                ui.label(egui::RichText::new("(Study folder)").color(theme.text_muted));
            }

            if ui.button("Browse...").clicked() {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    export.default_output_dir = Some(folder);
                }
            }

            if export.default_output_dir.is_some() && ui.button("Clear").clicked() {
                export.default_output_dir = None;
            }
        });

        ui.add_space(12.0);

        // Default format
        ui.label("Data Format:");
        ui.add_space(4.0);
        for format in ExportFormat::all() {
            let selected = export.default_format == *format;
            if ui
                .radio(selected, format!("{} - {}", format.display_name(), format.description()))
                .clicked()
            {
                export.default_format = *format;
            }
        }

        ui.add_space(12.0);

        // Define-XML generation
        ui.checkbox(
            &mut export.generate_define_xml,
            "Generate Define-XML (metadata documentation)",
        );
        ui.label(
            egui::RichText::new("Define-XML is always generated alongside XPT or Dataset-XML exports.")
                .color(theme.text_muted)
                .small(),
        );

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        // Filename template
        ui.horizontal(|ui| {
            ui.label("Filename Template:");
            ui.text_edit_singleline(&mut export.filename_template);
        });
        ui.label(
            egui::RichText::new("Use {domain} for domain code, {studyid} for study ID.")
                .color(theme.text_muted)
                .small(),
        );

        ui.add_space(12.0);

        // Overwrite without prompt
        ui.checkbox(
            &mut export.overwrite_without_prompt,
            "Overwrite existing files without prompting",
        );
    }

    /// Show display settings.
    fn show_display(&self, ui: &mut egui::Ui, display: &mut DisplaySettings, _theme: &ThemeColors) {
        ui.heading("Display Settings");
        ui.add_space(12.0);

        // Max preview rows
        ui.horizontal(|ui| {
            ui.label("Preview Row Limit:");
            egui::ComboBox::from_id_salt("preview_rows")
                .selected_text(display.max_preview_rows.display_name())
                .show_ui(ui, |ui| {
                    for limit in PreviewRowLimit::all() {
                        ui.selectable_value(&mut display.max_preview_rows, *limit, limit.display_name());
                    }
                });
        });

        ui.add_space(12.0);

        // Decimal precision
        ui.horizontal(|ui| {
            ui.label("Decimal Precision:");
            ui.add(egui::DragValue::new(&mut display.decimal_precision).range(0..=10));
        });

        ui.add_space(12.0);

        // Truncate long text
        ui.horizontal(|ui| {
            ui.label("Truncate Text After:");
            ui.add(egui::DragValue::new(&mut display.truncate_long_text).range(10..=500));
            ui.label("characters");
        });

        ui.add_space(12.0);

        // Show row numbers
        ui.checkbox(&mut display.show_row_numbers, "Show row numbers in tables");
    }

    /// Show shortcuts settings.
    fn show_shortcuts(&self, ui: &mut egui::Ui, settings: &mut Settings, theme: &ThemeColors) {
        ui.heading("Keyboard Shortcuts");
        ui.add_space(12.0);

        ui.label(
            egui::RichText::new("Current keyboard shortcuts (read-only in this version):")
                .color(theme.text_muted),
        );

        ui.add_space(8.0);

        egui::Grid::new("shortcuts_grid")
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                for action in ShortcutAction::all() {
                    ui.label(action.display_name());
                    if let Some(binding) = settings.shortcuts.bindings.get(action) {
                        ui.label(egui::RichText::new(binding.display()).monospace());
                    } else {
                        ui.label(egui::RichText::new("-").color(theme.text_muted));
                    }
                    ui.end_row();
                }
            });
    }
}
