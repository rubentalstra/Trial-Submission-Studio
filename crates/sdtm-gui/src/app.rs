//! Main application struct and eframe::App implementation

use crate::state::{AppState, EditorTab, View};
use crate::views::{DomainEditorView, ExportView, HomeView};
use eframe::egui;

/// Main application struct
pub struct CdiscApp {
    state: AppState,
}

impl CdiscApp {
    /// Create a new application instance
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // TODO: Load preferences from disk
        // TODO: Set up custom fonts/styles

        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for CdiscApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        self.handle_shortcuts(ctx);

        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state.view.clone() {
                View::Home => {
                    HomeView::show(ui, &mut self.state);
                }
                View::DomainEditor { domain, tab } => {
                    DomainEditorView::show(ui, &mut self.state, &domain, tab);
                }
                View::Export => {
                    ExportView::show(ui, &mut self.state);
                }
            }
        });
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
