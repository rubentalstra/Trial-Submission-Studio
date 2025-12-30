//! Main application struct and eframe::App implementation

use crate::services::StudyLoader;
use crate::state::{AppState, EditorTab, View};
use crate::views::{DomainEditorView, ExportView, HomeView};
use eframe::egui;

/// Main application struct
pub struct CdiscApp {
    state: AppState,
}

impl CdiscApp {
    /// Create a new application instance
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize Phosphor icons font
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for CdiscApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        self.handle_shortcuts(ctx);

        // Track if we need to load a study
        let mut folder_to_load = None;

        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| match self.state.view.clone() {
            View::Home => {
                folder_to_load = HomeView::show(ui, &mut self.state);
            }
            View::DomainEditor { domain, tab } => {
                DomainEditorView::show(ui, &mut self.state, &domain, tab);
            }
            View::Export => {
                ExportView::show(ui, &mut self.state);
            }
        });

        // Load study if folder was selected
        if let Some(folder) = folder_to_load {
            self.load_study(&folder);
        }
    }
}

impl CdiscApp {
    /// Load a study from a folder
    fn load_study(&mut self, folder: &std::path::Path) {
        match StudyLoader::load_study(folder) {
            Ok(study) => {
                let domain_count = study.domains.len();
                tracing::info!(
                    "Loaded study '{}' with {} domains",
                    study.study_id,
                    domain_count
                );
                self.state.study = Some(study);

                // Add to recent studies
                let path = folder.to_path_buf();
                self.state.preferences.recent_studies.retain(|p| p != &path);
                self.state.preferences.recent_studies.insert(0, path);
                if self.state.preferences.recent_studies.len() > 10 {
                    self.state.preferences.recent_studies.truncate(10);
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
                    // TODO: Load study
                }
            }

            // Cmd/Ctrl+E - Go to Export
            if cmd_or_ctrl && i.key_pressed(egui::Key::E) {
                if self.state.study.is_some() {
                    self.state.go_export();
                }
            }

            // Escape - Go back
            if i.key_pressed(egui::Key::Escape) {
                match &self.state.view {
                    View::Home => {}
                    View::DomainEditor { .. } | View::Export => {
                        self.state.go_home();
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
