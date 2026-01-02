//! Main application struct and eframe::App implementation

use crate::menu;
use crate::services::StudyLoader;
use crate::settings::ui::{SettingsResult, SettingsWindow};
use crate::settings::{load_settings, save_settings};
use crate::state::{AppState, EditorTab, View};
use crate::views::{DomainEditorView, ExportView, HomeAction, HomeView};
use crossbeam_channel::Receiver;
use eframe::egui;
use muda::{Menu, MenuEvent};

/// Main application struct
pub struct CdiscApp {
    state: AppState,
    menu_receiver: Receiver<MenuEvent>,
    /// Keep the menu alive for the lifetime of the app
    #[allow(dead_code)]
    menu: Menu,
    /// Settings window UI component
    settings_window: SettingsWindow,
}

impl CdiscApp {
    /// Create a new application instance
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        menu_receiver: Receiver<MenuEvent>,
        menu: Menu,
    ) -> Self {
        // Initialize Phosphor icons font
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        // Load settings from disk
        let settings = load_settings();
        tracing::info!("Loaded settings: dark_mode={}", settings.general.dark_mode);

        Self {
            state: AppState::new(settings),
            menu_receiver,
            menu,
            settings_window: SettingsWindow::default(),
        }
    }
}

impl eframe::App for CdiscApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle preview results from background threads
        self.handle_preview_results(ctx);

        // Handle menu events
        self.handle_menu_events(ctx);

        // Handle keyboard shortcuts
        self.handle_shortcuts(ctx);

        // Track home view action
        let mut home_action = HomeAction::None;

        // Show settings window if open
        if self.state.is_settings_open() {
            if let Some(ref mut pending) = self.state.ui.settings.pending {
                let dark_mode = pending.general.dark_mode;
                let result = self.settings_window.show(ctx, pending, dark_mode);

                match result {
                    SettingsResult::Open => {}
                    SettingsResult::Apply => {
                        self.state.close_settings(true);
                        // Save settings to disk
                        if let Err(e) = save_settings(&self.state.settings) {
                            tracing::error!("Failed to save settings: {}", e);
                        }
                    }
                    SettingsResult::Cancel => {
                        self.state.close_settings(false);
                    }
                }
            }
        }

        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| match self.state.view.clone() {
            View::Home => {
                home_action = HomeView::show(ui, &mut self.state);
            }
            View::DomainEditor { domain, tab } => {
                DomainEditorView::show(ui, &mut self.state, &domain, tab);
            }
            View::Export => {
                ExportView::show(ui, &mut self.state);
            }
        });

        // Handle home view actions
        match home_action {
            HomeAction::LoadStudy(folder) => {
                self.load_study(&folder);
            }
            HomeAction::CloseStudy => {
                self.state.clear_study();
            }
            HomeAction::None => {}
        }
    }
}

impl CdiscApp {
    /// Handle menu events from the native menu bar
    fn handle_menu_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.menu_receiver.try_recv() {
            let id = event.id().0.as_str();
            tracing::debug!("Menu event: {}", id);

            match id {
                menu::ids::OPEN_STUDY => {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        tracing::info!("Opening study from menu: {:?}", folder);
                        self.load_study(&folder);
                    }
                }
                menu::ids::SETTINGS => {
                    self.state.open_settings();
                }
                menu::ids::EXIT => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                menu::ids::ABOUT => {
                    // TODO: Show about dialog
                    tracing::info!("About clicked");
                }
                _ => {
                    tracing::debug!("Unknown menu event: {}", id);
                }
            }

            // Request repaint after menu event
            ctx.request_repaint();
        }
    }

    /// Load a study from a folder
    fn load_study(&mut self, folder: &std::path::Path) {
        let header_rows = self.state.settings.general.header_rows;
        match StudyLoader::load_study(folder, header_rows) {
            Ok(study) => {
                let domain_count = study.domains.len();
                tracing::info!(
                    "Loaded study '{}' with {} domains",
                    study.study_id,
                    domain_count
                );
                self.state.set_study(study);

                // Add to recent studies
                let path = folder.to_path_buf();
                self.state.settings.recent_studies.retain(|p| p != &path);
                self.state.settings.recent_studies.insert(0, path);
                if self.state.settings.recent_studies.len() > 10 {
                    self.state.settings.recent_studies.truncate(10);
                }

                // Save settings with updated recent studies
                if let Err(e) = save_settings(&self.state.settings) {
                    tracing::error!("Failed to save settings: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Failed to load study: {}", e);
                // TODO: Show error toast
            }
        }
    }
}

impl CdiscApp {
    /// Handle global keyboard shortcuts
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // Use Cmd on macOS, Ctrl on other platforms
        let modifiers = ctx.input(|i| i.modifiers);
        let cmd_or_ctrl = if cfg!(target_os = "macos") {
            modifiers.command
        } else {
            modifiers.ctrl
        };

        ctx.input(|i| {
            // Cmd/Ctrl+O - Open study
            if cmd_or_ctrl && i.key_pressed(egui::Key::O) {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    tracing::info!("Opening study: {:?}", folder);
                    // Note: Can't call load_study here due to borrow rules
                    // The menu handles this instead
                }
            }

            // Cmd/Ctrl+, - Open settings
            if cmd_or_ctrl && i.key_pressed(egui::Key::Comma) {
                if !self.state.is_settings_open() {
                    self.state.open_settings();
                }
            }

            // Cmd/Ctrl+E - Go to Export
            if cmd_or_ctrl && i.key_pressed(egui::Key::E) {
                if self.state.study.is_some() {
                    self.state.go_export();
                }
            }

            // Escape - Go back or close settings
            if i.key_pressed(egui::Key::Escape) {
                if self.state.is_settings_open() {
                    self.state.close_settings(false);
                } else {
                    match &self.state.view {
                        View::Home => {}
                        View::DomainEditor { .. } | View::Export => {
                            self.state.go_home();
                        }
                    }
                }
            }

            // Tab navigation in Domain Editor
            if let View::DomainEditor { tab, .. } = &self.state.view {
                let tabs = EditorTab::all();
                let current_idx = tabs.iter().position(|t| t == tab).unwrap_or(0);

                // Right arrow - next tab
                if i.key_pressed(egui::Key::ArrowRight) && !modifiers.shift {
                    if current_idx < tabs.len() - 1 {
                        self.state.switch_tab(tabs[current_idx + 1]);
                    }
                }

                // Left arrow - previous tab
                if i.key_pressed(egui::Key::ArrowLeft) && !modifiers.shift {
                    if current_idx > 0 {
                        self.state.switch_tab(tabs[current_idx - 1]);
                    }
                }
            }
        });
    }
}

impl CdiscApp {
    /// Handle preview results from background threads.
    ///
    /// Preview computation runs in background threads and sends results
    /// via channel. This method receives those results and updates state.
    fn handle_preview_results(&mut self, ctx: &egui::Context) {
        while let Ok(result) = self.state.preview_receiver.try_recv() {
            let domain_code = &result.domain_code;

            // Update preview state
            match result.result {
                Ok(df) => {
                    if let Some(domain) = self
                        .state
                        .study_mut()
                        .and_then(|s| s.get_domain_mut(domain_code))
                    {
                        domain.derived.preview = Some(df);
                    }
                    // Clear error on success
                    self.state.ui.domain_editor(domain_code).preview.error = None;
                }
                Err(e) => {
                    self.state.ui.domain_editor(domain_code).preview.error = Some(e);
                }
            }

            // Clear rebuilding flag
            self.state
                .ui
                .domain_editor(domain_code)
                .preview
                .is_rebuilding = false;

            // Request repaint to show the new preview
            ctx.request_repaint();
        }
    }
}
