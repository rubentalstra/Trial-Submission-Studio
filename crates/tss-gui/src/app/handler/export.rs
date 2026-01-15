//! Export flow message handlers.
//!
//! Handles:
//! - Export start/cancel
//! - Progress updates
//! - Completion (success/error/cancelled)
//! - Output folder operations

use iced::window;
use iced::{Size, Task};

use crate::app::App;
use crate::message::export::ExportProgress;
use crate::message::{ExportMessage, Message};
use crate::state::{ExportPhase, ExportResult, ViewState};

impl App {
    /// Handle export messages.
    pub fn handle_export_message(&mut self, msg: ExportMessage) -> Task<Message> {
        match msg {
            ExportMessage::DomainToggled(domain) => {
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.toggle_domain(&domain);
                }
                Task::none()
            }

            ExportMessage::SelectAll => {
                if let Some(study) = &self.state.study {
                    let domains: Vec<String> = study
                        .domain_codes()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect();
                    if let ViewState::Export(export_state) = &mut self.state.view {
                        export_state.select_all(domains);
                    }
                }
                Task::none()
            }

            ExportMessage::DeselectAll => {
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.deselect_all();
                }
                Task::none()
            }

            ExportMessage::FormatChanged(format) => {
                self.state.settings.export.default_format = format;
                let _ = self.state.settings.save();
                Task::none()
            }

            ExportMessage::OutputDirChangeClicked => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Select Output Directory")
                        .pick_folder()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                |path| match path {
                    Some(p) => Message::Export(ExportMessage::OutputDirSelected(p)),
                    None => Message::Noop,
                },
            ),

            ExportMessage::OutputDirSelected(path) => {
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.output_dir = Some(path);
                }
                Task::none()
            }

            ExportMessage::XptVersionChanged(version) => {
                self.state.settings.export.xpt_version = version;
                let _ = self.state.settings.save();
                Task::none()
            }

            ExportMessage::ToggleDefineXml => {
                // Define-XML is always generated - this is a no-op
                Task::none()
            }

            ExportMessage::StartExport => self.start_export(),

            ExportMessage::CancelExport => self.cancel_export(),

            ExportMessage::Progress(progress) => {
                self.update_export_progress(progress);
                Task::none()
            }

            ExportMessage::Complete(result) => self.complete_export(result),

            ExportMessage::DismissCompletion => {
                // Close completion dialog
                let task = if let Some((id, _)) = self.state.dialog_windows.export_complete.take() {
                    window::close(id)
                } else {
                    Task::none()
                };

                // Reset view state
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.reset_phase();
                }

                task
            }

            ExportMessage::RetryExport => {
                // Close completion dialog and restart export
                let close_task =
                    if let Some((id, _)) = self.state.dialog_windows.export_complete.take() {
                        window::close(id)
                    } else {
                        Task::none()
                    };

                // Reset view state
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.reset_phase();
                }

                // Batch close and then trigger start export
                Task::batch([
                    close_task,
                    Task::done(Message::Export(ExportMessage::StartExport)),
                ])
            }

            ExportMessage::OpenOutputFolder => {
                // Get output dir from completion dialog state
                if let Some((_, ref result)) = self.state.dialog_windows.export_complete {
                    if let ExportResult::Success { output_dir, .. } = result {
                        let _ = open::that(output_dir);
                    }
                }
                Task::none()
            }
        }
    }

    /// Start the export process.
    fn start_export(&mut self) -> Task<Message> {
        // Get export configuration
        let (selected_domains, output_dir) = match &self.state.view {
            ViewState::Export(export_state) => {
                let study_folder = self
                    .state
                    .study
                    .as_ref()
                    .map(|s| s.study_folder.clone())
                    .unwrap_or_default();
                (
                    export_state.selected_domains.clone(),
                    export_state.effective_output_dir(&study_folder),
                )
            }
            _ => return Task::none(),
        };

        if selected_domains.is_empty() {
            return Task::none();
        }

        // Get study data
        let study = match &self.state.study {
            Some(s) => s,
            None => return Task::none(),
        };

        let study_id = study.study_id.clone();
        let terminology = self.state.terminology.clone();

        // Build export data for each selected domain
        let mut domain_data = Vec::new();
        let mut not_collected_map = std::collections::HashMap::new();

        for code in &selected_domains {
            if let Some(gui_domain) = study.domain(code) {
                // Collect not_collected variables for validation
                let not_collected: std::collections::BTreeSet<String> = gui_domain
                    .mapping
                    .all_not_collected()
                    .keys()
                    .cloned()
                    .collect();
                if !not_collected.is_empty() {
                    not_collected_map.insert(code.clone(), not_collected);
                }

                match crate::service::export::build_domain_export_data(
                    code,
                    gui_domain,
                    &study_id,
                    terminology.as_ref(),
                ) {
                    Ok(data) => domain_data.push(data),
                    Err(e) => {
                        // Return error immediately
                        return Task::done(Message::Export(ExportMessage::Complete(
                            ExportResult::Error {
                                message: e.message,
                                domain: e.domain,
                            },
                        )));
                    }
                }
            }
        }

        if domain_data.is_empty() {
            return Task::none();
        }

        // Set exporting state in ViewState
        if let ViewState::Export(export_state) = &mut self.state.view {
            export_state.phase = ExportPhase::Exporting {
                current_domain: None,
                current_step: "Preparing...".to_string(),
                progress: 0.0,
                files_written: vec![],
            };
        }

        // Open progress dialog window
        let settings = window::Settings {
            size: Size::new(400.0, 300.0),
            resizable: false,
            decorations: true,
            level: window::Level::AlwaysOnTop,
            exit_on_close_request: false,
            ..Default::default()
        };
        let (id, open_task) = window::open(settings);

        // Store the progress state with window ID
        self.state.dialog_windows.export_progress = Some((
            id,
            crate::state::ExportProgressState {
                current_domain: None,
                current_step: "Exporting...".to_string(),
                progress: 0.0,
                files_written: 0,
            },
        ));

        // Build export input with validation settings
        let export_input = crate::service::export::ExportInput {
            output_dir,
            format: self.state.settings.export.default_format,
            xpt_version: self.state.settings.export.xpt_version,
            domains: domain_data,
            study_id,
            bypass_validation: self.state.settings.developer.bypass_validation,
            ct_registry: terminology,
            not_collected: not_collected_map,
        };

        // Start actual export task
        let export_task = Task::perform(
            crate::service::export::execute_export(export_input),
            |result| Message::Export(ExportMessage::Complete(result)),
        );

        Task::batch([open_task.map(|_| Message::Noop), export_task])
    }

    /// Cancel the export process.
    fn cancel_export(&mut self) -> Task<Message> {
        let mut tasks = vec![];

        // Close progress dialog if open
        if let Some((id, _)) = self.state.dialog_windows.export_progress.take() {
            tasks.push(window::close(id));
        }

        // Update view state
        if let ViewState::Export(export_state) = &mut self.state.view {
            export_state.phase = ExportPhase::Complete(ExportResult::Cancelled);
        }

        // Open completion dialog
        let settings = window::Settings {
            size: Size::new(400.0, 350.0),
            resizable: false,
            decorations: true,
            level: window::Level::AlwaysOnTop,
            exit_on_close_request: false,
            ..Default::default()
        };
        let (id, open_task) = window::open(settings);
        self.state.dialog_windows.export_complete = Some((id, ExportResult::Cancelled));
        tasks.push(open_task.map(|_| Message::Noop));

        Task::batch(tasks)
    }

    /// Update export progress.
    fn update_export_progress(&mut self, progress: ExportProgress) {
        // Update both ViewState and dialog window state
        if let ViewState::Export(export_state) = &mut self.state.view {
            if let ExportPhase::Exporting {
                current_domain,
                current_step,
                progress: prog,
                files_written: _,
            } = &mut export_state.phase
            {
                match &progress {
                    ExportProgress::StartingDomain(domain) => {
                        *current_domain = Some(domain.clone());
                    }
                    ExportProgress::Step(step) => {
                        *current_step = step.label().to_string();
                    }
                    ExportProgress::DomainComplete(_domain) => {
                        // Domain done
                    }
                    ExportProgress::OverallProgress(p) => {
                        *prog = *p;
                    }
                }

                // Also update dialog window state
                if let Some((_, ref mut dialog_state)) = self.state.dialog_windows.export_progress {
                    match progress {
                        ExportProgress::StartingDomain(domain) => {
                            dialog_state.current_domain = Some(domain);
                        }
                        ExportProgress::Step(step) => {
                            dialog_state.current_step = step.label().to_string();
                        }
                        ExportProgress::DomainComplete(_) => {}
                        ExportProgress::OverallProgress(p) => {
                            dialog_state.progress = p;
                        }
                    }
                }
            }
        }
    }

    /// Complete the export process.
    fn complete_export(&mut self, result: ExportResult) -> Task<Message> {
        let mut tasks = vec![];

        // Close progress dialog if open
        if let Some((id, _)) = self.state.dialog_windows.export_progress.take() {
            tasks.push(window::close(id));
        }

        // Update view state
        if let ViewState::Export(export_state) = &mut self.state.view {
            export_state.phase = ExportPhase::Complete(result.clone());
        }

        // Open completion dialog
        let settings = window::Settings {
            size: Size::new(450.0, 400.0),
            resizable: false,
            decorations: true,
            level: window::Level::AlwaysOnTop,
            exit_on_close_request: false,
            ..Default::default()
        };
        let (id, open_task) = window::open(settings);
        self.state.dialog_windows.export_complete = Some((id, result));
        tasks.push(open_task.map(|_| Message::Noop));

        Task::batch(tasks)
    }
}
