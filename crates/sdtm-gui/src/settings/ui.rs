//! Settings window UI implementation.
//!
//! Provides a native settings window with tabbed categories following macOS HIG:
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
use eframe::egui::{self, Color32, CornerRadius, Stroke, Vec2};

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

    /// Show the settings as a separate native window using viewports.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        settings: &mut Settings,
        dark_mode: bool,
    ) -> SettingsResult {
        let theme = colors(dark_mode);
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
                // Apply native styling
                self.apply_native_style(ctx, &theme);

                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
                    .show(ctx, |ui| {
                        result = self.show_native_layout(ui, settings, &theme);
                    });

                // Handle window close
                if ctx.input(|i| i.viewport().close_requested()) {
                    result = SettingsResult::Cancel;
                }
            },
        );

        result
    }

    /// Apply native-looking style to the context.
    fn apply_native_style(&self, ctx: &egui::Context, theme: &ThemeColors) {
        let mut style = (*ctx.style()).clone();

        // Use native-looking visuals
        style.visuals.widgets.noninteractive.bg_fill = theme.bg_secondary;
        style.visuals.widgets.inactive.bg_fill = theme.bg_secondary;
        style.visuals.widgets.hovered.bg_fill = theme.accent.linear_multiply(0.1);
        style.visuals.widgets.active.bg_fill = theme.accent.linear_multiply(0.2);

        // Subtle borders
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, theme.border);
        style.visuals.widgets.hovered.bg_stroke =
            Stroke::new(1.0, theme.accent.linear_multiply(0.5));

        // Use rounded corners globally
        style.visuals.window_corner_radius = CornerRadius::same(8);
        style.visuals.menu_corner_radius = CornerRadius::same(6);

        ctx.set_style(style);
    }

    /// Show the native macOS-style layout with sidebar.
    fn show_native_layout(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut Settings,
        theme: &ThemeColors,
    ) -> SettingsResult {
        let mut result = SettingsResult::Open;

        // Main horizontal split: sidebar + content
        ui.horizontal(|ui| {
            // Left sidebar with category navigation
            self.show_sidebar(ui, theme);

            // Vertical separator
            ui.add(egui::Separator::default().vertical());

            // Right content area
            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width());

                // Content area with scroll
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            ui.vertical(|ui| {
                                ui.set_max_width(500.0);
                                match self.category {
                                    SettingsCategory::General => {
                                        self.show_general(ui, &mut settings.general, theme)
                                    }
                                    SettingsCategory::Validation => {
                                        self.show_validation(ui, &mut settings.validation, theme)
                                    }
                                    SettingsCategory::Developer => {
                                        self.show_developer(ui, &mut settings.developer, theme)
                                    }
                                    SettingsCategory::Export => {
                                        self.show_export(ui, &mut settings.export, theme)
                                    }
                                    SettingsCategory::Display => {
                                        self.show_display(ui, &mut settings.display, theme)
                                    }
                                    SettingsCategory::Shortcuts => {
                                        self.show_shortcuts(ui, settings, theme)
                                    }
                                }
                            });
                        });
                    });

                // Bottom button bar
                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);

                        // Reset button on left
                        if ui.button("Reset to Defaults").clicked() {
                            *settings = Settings::default();
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(16.0);

                            // Primary action button (Apply)
                            if ui
                                .add(
                                    egui::Button::new("Apply")
                                        .fill(theme.accent)
                                        .min_size(Vec2::new(80.0, 28.0)),
                                )
                                .clicked()
                            {
                                result = SettingsResult::Apply;
                            }

                            ui.add_space(8.0);

                            // Cancel button
                            if ui
                                .add(egui::Button::new("Cancel").min_size(Vec2::new(80.0, 28.0)))
                                .clicked()
                            {
                                result = SettingsResult::Cancel;
                            }
                        });
                    });
                    ui.add_space(8.0);
                    ui.separator();
                });
            });
        });

        result
    }

    /// Show the sidebar with category navigation.
    fn show_sidebar(&mut self, ui: &mut egui::Ui, theme: &ThemeColors) {
        ui.vertical(|ui| {
            ui.set_min_width(180.0);
            ui.set_max_width(180.0);

            // Sidebar background
            let rect = ui.available_rect_before_wrap();
            ui.painter()
                .rect_filled(rect, CornerRadius::ZERO, theme.bg_secondary);

            ui.add_space(16.0);

            for category in SettingsCategory::all() {
                let selected = self.category == *category;

                // Create a selectable row
                let response = ui.allocate_response(
                    Vec2::new(ui.available_width() - 16.0, 32.0),
                    egui::Sense::click(),
                );

                // Draw background for selected/hovered
                let bg_rect = response.rect.expand2(Vec2::new(8.0, 0.0));
                if selected {
                    ui.painter().rect_filled(
                        bg_rect,
                        CornerRadius::same(6),
                        theme.accent.linear_multiply(0.15),
                    );
                } else if response.hovered() {
                    ui.painter().rect_filled(
                        bg_rect,
                        CornerRadius::same(6),
                        Color32::from_white_alpha(10),
                    );
                }

                // Draw icon and text
                let text_color = if selected {
                    theme.accent
                } else {
                    theme.text_primary
                };

                ui.painter().text(
                    response.rect.left_center() + Vec2::new(12.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    format!("{} {}", category.icon(), category.name()),
                    egui::FontId::proportional(14.0),
                    text_color,
                );

                if response.clicked() {
                    self.category = *category;
                }
            }
        });
    }

    /// Show a section header.
    fn section_header(&self, ui: &mut egui::Ui, title: &str, theme: &ThemeColors) {
        ui.label(
            egui::RichText::new(title)
                .size(20.0)
                .strong()
                .color(theme.text_primary),
        );
        ui.add_space(16.0);
    }

    /// Show a setting row with label and widget.
    fn setting_row<R>(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        description: Option<&str>,
        theme: &ThemeColors,
        add_widget: impl FnOnce(&mut egui::Ui) -> R,
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_min_width(200.0);
                ui.label(egui::RichText::new(label).color(theme.text_primary));
                if let Some(desc) = description {
                    ui.label(
                        egui::RichText::new(desc)
                            .small()
                            .color(theme.text_muted),
                    );
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
        theme: &ThemeColors,
        add_content: impl FnOnce(&mut egui::Ui),
    ) {
        if let Some(t) = title {
            ui.label(
                egui::RichText::new(t)
                    .size(13.0)
                    .strong()
                    .color(theme.text_muted),
            );
            ui.add_space(8.0);
        }

        egui::Frame::none()
            .fill(theme.bg_secondary)
            .rounding(CornerRadius::same(8))
            .inner_margin(16.0)
            .show(ui, |ui| {
                add_content(ui);
            });

        ui.add_space(20.0);
    }

    /// Show general settings.
    fn show_general(&self, ui: &mut egui::Ui, general: &mut GeneralSettings, theme: &ThemeColors) {
        self.section_header(ui, "General", theme);

        self.setting_group(ui, Some("APPEARANCE"), theme, |ui| {
            self.setting_row(ui, "Dark Mode", Some("Use dark color scheme"), theme, |ui| {
                ui.add(toggle(&mut general.dark_mode));
            });
        });

        self.setting_group(ui, Some("DATA IMPORT"), theme, |ui| {
            self.setting_row(
                ui,
                "Controlled Terminology",
                Some("CDISC CT version for validation"),
                theme,
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
                theme,
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
    fn show_validation(
        &self,
        ui: &mut egui::Ui,
        validation: &mut ValidationSettings,
        theme: &ThemeColors,
    ) {
        self.section_header(ui, "Validation", theme);

        self.setting_group(ui, Some("VALIDATION MODE"), theme, |ui| {
            for mode in ValidationModeSetting::all() {
                let selected = validation.mode == *mode;
                if ui
                    .add(egui::RadioButton::new(selected, mode.display_name()))
                    .clicked()
                {
                    validation.mode = *mode;
                }
                ui.label(
                    egui::RichText::new(mode.description())
                        .small()
                        .color(theme.text_muted),
                );
                ui.add_space(8.0);
            }
        });

        self.setting_group(ui, Some("XPT FORMAT"), theme, |ui| {
            self.setting_row(
                ui,
                "XPT Version",
                Some("Transport file format version"),
                theme,
                |ui| {
                    egui::ComboBox::from_id_salt("xpt_version")
                        .width(200.0)
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
                },
            );
        });

        // Custom rules (only shown when Custom mode is selected)
        if validation.mode == ValidationModeSetting::Custom {
            self.setting_group(ui, Some("ENABLED RULES"), theme, |ui| {
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
                    ui.label(
                        egui::RichText::new(rule.description())
                            .small()
                            .color(theme.text_muted),
                    );
                    ui.add_space(4.0);
                }
            });
        }
    }

    /// Show developer settings.
    fn show_developer(
        &self,
        ui: &mut egui::Ui,
        developer: &mut DeveloperSettings,
        theme: &ThemeColors,
    ) {
        self.section_header(ui, "Developer", theme);

        self.setting_group(ui, None, theme, |ui| {
            ui.label(
                egui::RichText::new(
                    "Developer mode allows you to bypass validation checks for testing.",
                )
                .color(theme.text_muted),
            );
            ui.add_space(12.0);

            self.setting_row(ui, "Developer Mode", None, theme, |ui| {
                ui.add(toggle(&mut developer.enabled));
            });
        });

        if developer.enabled {
            self.setting_group(ui, Some("OPTIONS"), theme, |ui| {
                self.setting_row(
                    ui,
                    "Export with Errors",
                    Some("Allow export even with validation errors"),
                    theme,
                    |ui| {
                        ui.add(toggle(&mut developer.allow_export_with_errors));
                    },
                );

                ui.separator();
                ui.add_space(8.0);

                self.setting_row(
                    ui,
                    "Debug Info",
                    Some("Show debug information in UI"),
                    theme,
                    |ui| {
                        ui.add(toggle(&mut developer.show_debug_info));
                    },
                );
            });

            self.setting_group(ui, Some("BYPASS RULES"), theme, |ui| {
                ui.label(
                    egui::RichText::new("Selected rules will be skipped during validation.")
                        .small()
                        .color(theme.text_muted),
                );
                ui.add_space(8.0);

                for rule in XptValidationRule::all() {
                    let mut bypassed = developer.bypassed_rules.contains(rule);
                    if ui.checkbox(&mut bypassed, rule.display_name()).changed() {
                        if bypassed {
                            developer.bypassed_rules.insert(*rule);
                        } else {
                            developer.bypassed_rules.remove(rule);
                        }
                    }
                }
            });
        }
    }

    /// Show export settings.
    fn show_export(&self, ui: &mut egui::Ui, export: &mut ExportSettings, theme: &ThemeColors) {
        self.section_header(ui, "Export", theme);

        self.setting_group(ui, Some("OUTPUT"), theme, |ui| {
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
                        .color(theme.text_muted),
                );
            } else {
                ui.label(
                    egui::RichText::new("Uses study folder by default")
                        .small()
                        .color(theme.text_muted),
                );
            }
        });

        self.setting_group(ui, Some("DATA FORMAT"), theme, |ui| {
            for format in ExportFormat::all() {
                let selected = export.default_format == *format;
                if ui
                    .add(egui::RadioButton::new(selected, format.display_name()))
                    .clicked()
                {
                    export.default_format = *format;
                }
                ui.label(
                    egui::RichText::new(format.description())
                        .small()
                        .color(theme.text_muted),
                );
                ui.add_space(4.0);
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.checkbox(&mut export.generate_define_xml, "Generate Define-XML");
            });
            ui.label(
                egui::RichText::new("Metadata documentation generated with exports")
                    .small()
                    .color(theme.text_muted),
            );
        });

        self.setting_group(ui, Some("FILE OPTIONS"), theme, |ui| {
            self.setting_row(ui, "Filename Template", None, theme, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut export.filename_template)
                        .desired_width(150.0),
                );
            });
            ui.label(
                egui::RichText::new("Use {domain} for domain code, {studyid} for study ID")
                    .small()
                    .color(theme.text_muted),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            self.setting_row(
                ui,
                "Overwrite Files",
                Some("Skip confirmation when overwriting"),
                theme,
                |ui| {
                    ui.add(toggle(&mut export.overwrite_without_prompt));
                },
            );
        });
    }

    /// Show display settings.
    fn show_display(
        &self,
        ui: &mut egui::Ui,
        display: &mut DisplaySettings,
        theme: &ThemeColors,
    ) {
        self.section_header(ui, "Display", theme);

        self.setting_group(ui, Some("DATA PREVIEW"), theme, |ui| {
            self.setting_row(
                ui,
                "Row Limit",
                Some("Maximum rows shown in preview"),
                theme,
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
                theme,
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
                theme,
                |ui| {
                    ui.add(
                        egui::DragValue::new(&mut display.truncate_long_text)
                            .range(10..=500)
                            .speed(1.0),
                    );
                },
            );
        });

        self.setting_group(ui, Some("TABLE OPTIONS"), theme, |ui| {
            self.setting_row(
                ui,
                "Row Numbers",
                Some("Show row numbers in data tables"),
                theme,
                |ui| {
                    ui.add(toggle(&mut display.show_row_numbers));
                },
            );
        });
    }

    /// Show shortcuts settings.
    fn show_shortcuts(&self, ui: &mut egui::Ui, settings: &mut Settings, theme: &ThemeColors) {
        self.section_header(ui, "Keyboard Shortcuts", theme);

        self.setting_group(ui, None, theme, |ui| {
            ui.label(
                egui::RichText::new("Current keyboard shortcuts (read-only in this version)")
                    .color(theme.text_muted),
            );
            ui.add_space(12.0);

            egui::Grid::new("shortcuts_grid")
                .num_columns(2)
                .spacing([40.0, 12.0])
                .show(ui, |ui| {
                    for action in ShortcutAction::all() {
                        ui.label(
                            egui::RichText::new(action.display_name()).color(theme.text_primary),
                        );
                        if let Some(binding) = settings.shortcuts.bindings.get(action) {
                            ui.label(
                                egui::RichText::new(binding.display())
                                    .monospace()
                                    .color(theme.text_muted),
                            );
                        } else {
                            ui.label(egui::RichText::new("-").color(theme.text_muted));
                        }
                        ui.end_row();
                    }
                });
        });
    }
}

/// A toggle switch widget (iOS-style).
fn toggle(value: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| -> egui::Response {
        let desired_size = Vec2::new(44.0, 24.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if response.clicked() {
            *value = !*value;
        }

        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool_responsive(response.id, *value);

            let bg_color = if *value {
                Color32::from_rgb(52, 199, 89) // iOS green
            } else {
                Color32::from_gray(180)
            };

            // Background pill
            ui.painter()
                .rect_filled(rect, CornerRadius::same(12), bg_color);

            // Knob
            let knob_radius = 10.0;
            let knob_x = egui::lerp(
                (rect.left() + knob_radius + 2.0)..=(rect.right() - knob_radius - 2.0),
                how_on,
            );
            let knob_center = egui::pos2(knob_x, rect.center().y);

            ui.painter()
                .circle_filled(knob_center, knob_radius, Color32::WHITE);
        }

        response
    }
}
