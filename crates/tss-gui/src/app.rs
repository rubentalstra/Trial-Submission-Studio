//! Main application struct and eframe::App implementation

use crate::menu;
use crate::services::StudyLoader;
use crate::settings::ui::{SettingsResult, SettingsWindow};
use crate::settings::{load_settings, save_settings};
use crate::state::{AppState, EditorTab, UpdatePhase, View};
use crate::views::{
    show_about_dialog, show_update_dialog, CommonMarkCache, DomainEditorView, ExportView,
    HomeAction, HomeView, UpdateDialogAction,
};
use crossbeam_channel::Receiver;
use eframe::egui;
use muda::{Menu, MenuEvent};
use std::sync::mpsc;
use std::thread;
use tss_updater::{DownloadProgress, UpdateInfo, UpdateService};

/// Message sent from update background thread.
pub enum UpdateMessage {
    /// Update check completed with result.
    CheckComplete(Result<Option<UpdateInfo>, String>),
    /// Download progress update.
    DownloadProgress { progress: f32, speed: u64 },
    /// Download completed with path and update info.
    DownloadComplete(Result<(std::path::PathBuf, UpdateInfo), String>),
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
    /// Current update info (if an update is available)
    pending_update: Option<UpdateInfo>,
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
            pending_update: None,
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

        // Show update dialog if open
        let update_action = show_update_dialog(
            ctx,
            &mut self.state.ui.update,
            &mut self.markdown_cache,
        );
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

impl CdiscApp {
    /// Handle update messages from background threads.
    fn handle_update_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.update_receiver.try_recv() {
            match msg {
                UpdateMessage::CheckComplete(result) => {
                    match result {
                        Ok(Some(update_info)) => {
                            tracing::info!(
                                "Update available: {}",
                                update_info.new_version.to_string()
                            );
                            self.state.ui.update.set_update_available(
                                update_info.new_version.to_string(),
                                update_info.changelog().to_string(),
                            );
                            self.pending_update = Some(update_info);
                            self.state.ui.update.open = true;
                        }
                        Ok(None) => {
                            tracing::info!("No update available");
                            // If dialog was opened for manual check, show "up to date" briefly
                            if self.state.ui.update.open {
                                self.state.ui.update.phase = UpdatePhase::Idle;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Update check failed: {}", e);
                            // Only show error if dialog is open (manual check)
                            if self.state.ui.update.open {
                                self.state.ui.update.set_error(e);
                            }
                        }
                    }
                    // Record check time
                    self.state.settings.updates.record_check();
                    if let Err(e) = save_settings(&self.state.settings) {
                        tracing::error!("Failed to save settings: {}", e);
                    }
                }
                UpdateMessage::DownloadProgress { progress, speed } => {
                    self.state.ui.update.update_progress(progress, speed);
                }
                UpdateMessage::DownloadComplete(result) => {
                    match result {
                        Ok((path, update_info)) => {
                            tracing::info!("Download complete: {:?}", path);
                            self.state.ui.update.set_downloaded(path);
                            self.pending_update = Some(update_info);
                        }
                        Err(e) => {
                            tracing::error!("Download failed: {}", e);
                            self.state.ui.update.set_error(e);
                        }
                    }
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
        self.start_update_check_background(ctx, false);
    }

    /// Start an update check (opens dialog for manual check).
    fn start_update_check(&mut self, ctx: &egui::Context) {
        // Check rate limiting for manual checks
        if !self.state.settings.updates.can_check_manually() {
            if let Some(secs) = self.state.settings.updates.seconds_until_manual_check_allowed() {
                tracing::info!(
                    "Manual check rate limited, {} seconds until allowed",
                    secs
                );
                // TODO: Show a toast message instead
            }
            return;
        }

        self.state.ui.update.open_checking();
        self.start_update_check_background(ctx, true);
    }

    /// Start update check in background thread.
    fn start_update_check_background(&mut self, ctx: &egui::Context, open_dialog: bool) {
        let sender = self.update_sender.clone();
        let settings = self.state.settings.updates.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            let result: Result<Option<UpdateInfo>, String> = (|| {
                let service = UpdateService::new().map_err(|e| e.to_string())?;

                match service.check_for_update(&settings) {
                    Ok(Some(update_info)) => {
                        // Check if this version is skipped
                        if let Some(ref skip_ver) = settings.skipped_version {
                            if skip_ver == &update_info.new_version.to_string() {
                                return Ok(None);
                            }
                        }
                        Ok(Some(update_info))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => Err(e.to_string()),
                }
            })();

            let _ = sender.send(UpdateMessage::CheckComplete(result));
            ctx.request_repaint();
        });

        if open_dialog && !self.state.ui.update.open {
            self.state.ui.update.open_checking();
        }
    }

    /// Handle actions from the update dialog.
    fn handle_update_action(&mut self, action: UpdateDialogAction, ctx: &egui::Context) {
        match action {
            UpdateDialogAction::None => {}
            UpdateDialogAction::SkipVersion => {
                if let Some(ref version) = self.state.ui.update.available_version {
                    self.state.settings.updates.skipped_version = Some(version.clone());
                    if let Err(e) = save_settings(&self.state.settings) {
                        tracing::error!("Failed to save settings: {}", e);
                    }
                }
            }
            UpdateDialogAction::RemindLater => {
                // Just close the dialog, nothing else to do
            }
            UpdateDialogAction::Download => {
                self.start_download(ctx);
            }
            UpdateDialogAction::InstallAndRestart => {
                self.install_and_restart();
            }
            UpdateDialogAction::Cancel => {
                // Dialog will handle closing itself
            }
        }
    }

    /// Start downloading the update.
    fn start_download(&mut self, ctx: &egui::Context) {
        let Some(update_info) = self.pending_update.clone() else {
            tracing::error!("No pending update to download");
            return;
        };

        self.state.ui.update.set_downloading();

        let sender = self.update_sender.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            let result: Result<(std::path::PathBuf, UpdateInfo), String> = (|| {
                let service = UpdateService::new().map_err(|e| e.to_string())?;

                // Download with progress
                let sender_clone = sender.clone();
                let ctx_clone = ctx.clone();
                let path = service
                    .download_update(&update_info, move |progress: DownloadProgress| {
                        let _ = sender_clone.send(UpdateMessage::DownloadProgress {
                            progress: f32::from(progress.percentage()) / 100.0,
                            speed: progress.speed_bps,
                        });
                        ctx_clone.request_repaint();
                    })
                    .map_err(|e| e.to_string())?;

                // Verify checksum if available
                service.verify_update(&path, &update_info).map_err(|e| e.to_string())?;

                Ok((path, update_info))
            })();

            let _ = sender.send(UpdateMessage::DownloadComplete(result));
            ctx.request_repaint();
        });
    }

    /// Install the downloaded update and restart.
    fn install_and_restart(&mut self) {
        let Some(ref path) = self.state.ui.update.downloaded_path else {
            tracing::error!("No downloaded update path");
            return;
        };
        let Some(ref update_info) = self.pending_update else {
            tracing::error!("No pending update info");
            return;
        };

        tracing::info!("Installing update from {:?}", path);

        // Install and restart - this function may not return on success
        match tss_updater::install_from_archive(path, update_info) {
            Ok(()) => {
                if let Err(e) = tss_updater::restart_application() {
                    tracing::error!("Failed to restart application: {}", e);
                    self.state.ui.update.set_error(e.to_string());
                }
            }
            Err(e) => {
                tracing::error!("Failed to install update: {}", e);
                self.state.ui.update.set_error(e.to_string());
            }
        }
    }
}
