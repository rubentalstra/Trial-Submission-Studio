//! Main application struct and eframe::App implementation

use crate::menu;
use crate::services::StudyLoader;
use crate::settings::ui::{SettingsResult, SettingsWindow};
use crate::settings::{load_settings, save_settings};
use crate::state::{AppState, EditorTab, UpdateDialogState, View};
use crate::views::{
    CommonMarkCache, DomainEditorView, ExportView, HomeAction, HomeView, UpdateDialogAction,
    show_about_dialog, show_update_dialog,
};
use crossbeam_channel::Receiver;
use eframe::egui;
use muda::{Menu, MenuEvent};
use std::sync::mpsc;
use std::thread;
use tss_updater::UpdateService;

/// Message sent from update background thread.
enum UpdateMessage {
    /// Update check completed: Ok(Some((version, changelog))) if available, Ok(None) if up-to-date.
    CheckResult(Result<Option<(String, String)>, String>),
    /// Install failed with error (success means app restarted).
    InstallFailed(String),
}

/// Main application struct
pub struct CdiscApp {
    state: AppState,
    menu_receiver: Receiver<MenuEvent>,
    /// Keep the menu alive for the lifetime of the app
    #[allow(dead_code)]
    menu: Menu,
    /// Settings window UI component
    settings_window: SettingsWindow,
    /// Markdown cache for rendering changelogs
    markdown_cache: CommonMarkCache,
    /// Whether we've performed the startup update check
    startup_check_done: bool,
    /// Channel for receiving update messages from background threads
    update_receiver: mpsc::Receiver<UpdateMessage>,
    /// Sender for update messages (cloned for background threads)
    update_sender: mpsc::Sender<UpdateMessage>,
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

        // Create update message channel
        let (update_sender, update_receiver) = mpsc::channel();

        Self {
            state: AppState::new(settings),
            menu_receiver,
            menu,
            settings_window: SettingsWindow::default(),
            markdown_cache: CommonMarkCache::default(),
            startup_check_done: false,
            update_receiver,
            update_sender,
        }
    }
}

impl eframe::App for CdiscApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle preview results from background threads
        self.handle_preview_results(ctx);

        // Handle update messages from background threads
        self.handle_update_messages(ctx);

        // Handle menu events
        self.handle_menu_events(ctx);

        // Handle keyboard shortcuts
        self.handle_shortcuts(ctx);

        // Perform startup update check if needed
        self.maybe_startup_update_check(ctx);

        // Track home view action
        let mut home_action = HomeAction::None;

        // Show settings window if open
        if self.state.is_settings_open()
            && let Some(ref mut pending) = self.state.ui.settings.pending
        {
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

        // Show update dialog if open
        let update_action =
            show_update_dialog(ctx, &mut self.state.ui.update, &mut self.markdown_cache);
        self.handle_update_action(update_action, ctx);

        // Show about dialog if open
        show_about_dialog(ctx, &mut self.state.ui.about);

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
                    self.state.ui.about.open();
                }
                menu::ids::CHECK_UPDATES => {
                    self.start_update_check(ctx);
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
            if cmd_or_ctrl
                && i.key_pressed(egui::Key::O)
                && let Some(folder) = rfd::FileDialog::new().pick_folder()
            {
                tracing::info!("Opening study: {:?}", folder);
                // Note: Can't call load_study here due to borrow rules
                // The menu handles this instead
            }

            // Cmd/Ctrl+, - Open settings
            if cmd_or_ctrl && i.key_pressed(egui::Key::Comma) && !self.state.is_settings_open() {
                self.state.open_settings();
            }

            // Cmd/Ctrl+E - Go to Export
            if cmd_or_ctrl && i.key_pressed(egui::Key::E) && self.state.study.is_some() {
                self.state.go_export();
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
                if i.key_pressed(egui::Key::ArrowRight)
                    && !modifiers.shift
                    && current_idx < tabs.len() - 1
                {
                    self.state.switch_tab(tabs[current_idx + 1]);
                }

                // Left arrow - previous tab
                if i.key_pressed(egui::Key::ArrowLeft) && !modifiers.shift && current_idx > 0 {
                    self.state.switch_tab(tabs[current_idx - 1]);
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

impl CdiscApp {
    /// Handle update messages from background threads.
    fn handle_update_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.update_receiver.try_recv() {
            match msg {
                UpdateMessage::CheckResult(result) => {
                    match result {
                        Ok(Some((version, changelog))) => {
                            tracing::info!("Update available: {}", version);
                            self.state.ui.update =
                                UpdateDialogState::UpdateAvailable { version, changelog };
                        }
                        Ok(None) => {
                            tracing::info!("No update available");
                            // Show "up to date" only if dialog was opened (manual check)
                            if self.state.ui.update.is_open() {
                                self.state.ui.update = UpdateDialogState::NoUpdate;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Update check failed: {}", e);
                            // Show error only if dialog is open (manual check)
                            if self.state.ui.update.is_open() {
                                self.state.ui.update = UpdateDialogState::Error(e);
                            }
                        }
                    }
                    // Record check time
                    self.state.settings.updates.record_check();
                    if let Err(e) = save_settings(&self.state.settings) {
                        tracing::error!("Failed to save settings: {}", e);
                    }
                }
                UpdateMessage::InstallFailed(error) => {
                    tracing::error!("Update installation failed: {}", error);
                    self.state.ui.update = UpdateDialogState::Error(error);
                }
            }
            ctx.request_repaint();
        }
    }

    /// Perform startup update check if settings allow it.
    fn maybe_startup_update_check(&mut self, ctx: &egui::Context) {
        if self.startup_check_done {
            return;
        }
        self.startup_check_done = true;

        // Check if we should perform an automatic check
        if !self.state.settings.updates.should_check_now() {
            tracing::debug!("Skipping startup update check (disabled or recently checked)");
            return;
        }

        tracing::info!("Performing startup update check");
        self.start_update_check_background(ctx);
    }

    /// Start an update check (opens dialog for manual check).
    fn start_update_check(&mut self, ctx: &egui::Context) {
        // Check rate limiting for manual checks
        if !self.state.settings.updates.can_check_manually() {
            if let Some(secs) = self
                .state
                .settings
                .updates
                .seconds_until_manual_check_allowed()
            {
                tracing::info!("Manual check rate limited, {} seconds until allowed", secs);
            }
            return;
        }

        self.state.ui.update = UpdateDialogState::Checking;
        self.start_update_check_background(ctx);
    }

    /// Start update check in background thread.
    fn start_update_check_background(&mut self, ctx: &egui::Context) {
        let sender = self.update_sender.clone();
        let settings = self.state.settings.updates.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            let result = match UpdateService::check_for_update(&settings) {
                Ok(Some(info)) => Ok(Some((info.version, info.changelog))),
                Ok(None) => Ok(None),
                Err(e) => Err(e.to_string()),
            };

            let _ = sender.send(UpdateMessage::CheckResult(result));
            ctx.request_repaint();
        });
    }

    /// Handle actions from the update dialog.
    fn handle_update_action(&mut self, action: UpdateDialogAction, ctx: &egui::Context) {
        match action {
            UpdateDialogAction::None => {}
            UpdateDialogAction::SkipVersion => {
                // Extract version from current state
                if let UpdateDialogState::UpdateAvailable { ref version, .. } = self.state.ui.update
                {
                    self.state.settings.updates.skipped_version = Some(version.clone());
                    if let Err(e) = save_settings(&self.state.settings) {
                        tracing::error!("Failed to save settings: {}", e);
                    }
                }
                self.state.ui.update.close();
            }
            UpdateDialogAction::RemindLater => {
                self.state.ui.update.close();
            }
            UpdateDialogAction::InstallAndRestart => {
                self.start_install(ctx);
            }
            UpdateDialogAction::Cancel => {
                self.state.ui.update.close();
            }
        }
    }

    /// Download, install update, and restart the application.
    ///
    /// Uses self_update for the entire process. On success, the app will restart.
    fn start_install(&mut self, ctx: &egui::Context) {
        self.state.ui.update = UpdateDialogState::Installing;

        let sender = self.update_sender.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            // Download and install - self_update handles everything
            if let Err(e) = UpdateService::download_and_install() {
                let _ = sender.send(UpdateMessage::InstallFailed(e.to_string()));
                ctx.request_repaint();
                return;
            }

            // Restart the application
            if let Err(e) = UpdateService::restart() {
                let _ = sender.send(UpdateMessage::InstallFailed(e.to_string()));
                ctx.request_repaint();
            }
            // On success, the app restarts and this thread ends
        });
    }
}
