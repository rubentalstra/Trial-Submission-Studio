//! Dialog message handlers.
//!
//! Handles:
//! - About dialog
//! - Settings dialog (all categories)
//! - Third-party licenses dialog
//! - Update check dialog

use iced::Task;
use iced::window;

use crate::app::App;
use crate::message::Message;
use crate::message::{
    AboutMessage, DeveloperSettingsMessage, DialogMessage, ExportSettingsMessage,
    GeneralSettingsMessage, SettingsMessage, ThirdPartyMessage, UpdateMessage,
    UpdateSettingsMessage,
};
use crate::state::Settings;

impl App {
    /// Handle dialog messages.
    pub fn handle_dialog_message(&mut self, msg: DialogMessage) -> Task<Message> {
        match msg {
            DialogMessage::About(about_msg) => self.handle_about_message(about_msg),
            DialogMessage::Settings(settings_msg) => self.handle_settings_message(settings_msg),
            DialogMessage::ThirdParty(tp_msg) => self.handle_third_party_message(tp_msg),
            DialogMessage::Update(update_msg) => self.handle_update_message(update_msg),
            DialogMessage::CloseAll => Task::none(),
        }
    }

    /// Handle about dialog messages.
    fn handle_about_message(&mut self, msg: AboutMessage) -> Task<Message> {
        match msg {
            AboutMessage::Open => Task::none(),
            AboutMessage::Close => {
                // Close dialog window in multi-window mode
                if let Some(id) = self.state.dialog_windows.about.take() {
                    return window::close(id);
                }
                Task::none()
            }
            AboutMessage::CopyAndClose => {
                // Copy system info to clipboard using Iced's clipboard
                let info = crate::view::dialog::about::generate_system_info();
                let copy_task = iced::clipboard::write(info);

                // Close dialog window in multi-window mode
                if let Some(id) = self.state.dialog_windows.about.take() {
                    return Task::batch([copy_task, window::close(id)]);
                }
                copy_task
            }
            AboutMessage::OpenWebsite => {
                let _ = open::that("https://trialsubmissionstudio.com");
                Task::none()
            }
            AboutMessage::OpenGitHub => {
                let _ = open::that("https://github.com/rubentalstra/trial-submission-studio");
                Task::none()
            }
            AboutMessage::OpenOpenSource => {
                // Open the third-party licenses or open source page
                let _ = open::that(
                    "https://github.com/rubentalstra/trial-submission-studio/blob/main/THIRD_PARTY_LICENSES.md",
                );
                Task::none()
            }
        }
    }

    /// Handle settings dialog messages.
    fn handle_settings_message(&mut self, msg: SettingsMessage) -> Task<Message> {
        match msg {
            SettingsMessage::Open => Task::none(),
            SettingsMessage::Close => {
                // Close dialog window in multi-window mode
                if let Some((id, _)) = self.state.dialog_windows.settings.take() {
                    return window::close(id);
                }
                Task::none()
            }
            SettingsMessage::Apply => {
                // Save settings
                let _ = self.state.settings.save();
                tracing::info!("Settings saved");

                // Close dialog window in multi-window mode
                if let Some((id, _)) = self.state.dialog_windows.settings.take() {
                    return window::close(id);
                }
                Task::none()
            }
            SettingsMessage::ResetToDefaults => {
                self.state.settings = Settings::default();
                Task::none()
            }
            SettingsMessage::CategorySelected(category) => {
                // Update dialog_windows.settings for multi-window mode
                if let Some((id, _)) = self.state.dialog_windows.settings {
                    self.state.dialog_windows.settings = Some((id, category));
                }
                Task::none()
            }
            SettingsMessage::General(general_msg) => {
                match general_msg {
                    GeneralSettingsMessage::CtVersionChanged(_version) => {
                        // CT version change - would reload terminology
                    }
                    GeneralSettingsMessage::HeaderRowsChanged(rows) => {
                        self.state.settings.general.header_rows = rows;
                    }
                    GeneralSettingsMessage::ConfidenceThresholdChanged(threshold) => {
                        self.state.settings.general.mapping_confidence_threshold = threshold;
                    }
                }
                Task::none()
            }
            SettingsMessage::Validation(_val_msg) => {
                // Handle validation settings
                Task::none()
            }
            SettingsMessage::Developer(dev_msg) => {
                match dev_msg {
                    DeveloperSettingsMessage::BypassValidationToggled(enabled) => {
                        self.state.settings.developer.bypass_validation = enabled;
                        tracing::info!("Bypass validation: {}", enabled);
                    }
                    DeveloperSettingsMessage::DeveloperModeToggled(enabled) => {
                        self.state.settings.developer.developer_mode = enabled;
                        tracing::info!("Developer mode: {}", enabled);
                    }
                }
                Task::none()
            }
            SettingsMessage::Export(export_msg) => {
                match export_msg {
                    ExportSettingsMessage::DefaultOutputDirChanged(_dir) => {
                        // Handle output dir change
                    }
                    ExportSettingsMessage::DefaultFormatChanged(format) => {
                        self.state.settings.export.default_format = format;
                    }
                    ExportSettingsMessage::DefaultXptVersionChanged(version) => {
                        self.state.settings.export.xpt_version = version;
                    }
                }
                Task::none()
            }
            SettingsMessage::Display(_display_msg) => {
                // Handle display settings
                Task::none()
            }
            SettingsMessage::Updates(update_msg) => {
                match update_msg {
                    UpdateSettingsMessage::AutoCheckToggled(enabled) => {
                        self.state.settings.general.auto_check_updates = enabled;
                    }
                    UpdateSettingsMessage::CheckFrequencyChanged(_freq) => {
                        // Handle frequency change
                    }
                }
                Task::none()
            }
        }
    }

    /// Handle third-party licenses dialog messages.
    fn handle_third_party_message(&mut self, msg: ThirdPartyMessage) -> Task<Message> {
        match msg {
            ThirdPartyMessage::Open => Task::none(),
            ThirdPartyMessage::Close => Task::none(),
            ThirdPartyMessage::ScrollTo(_position) => {
                // Handle scroll - would need scrollable state
                Task::none()
            }
        }
    }

    /// Handle update dialog messages.
    fn handle_update_message(&mut self, msg: UpdateMessage) -> Task<Message> {
        match msg {
            UpdateMessage::Open => Task::none(),
            UpdateMessage::Close => Task::none(),
            UpdateMessage::CheckForUpdates => {
                // TODO: Start actual update check
                // For now, simulate up-to-date
                Task::done(Message::Dialog(DialogMessage::Update(
                    UpdateMessage::CheckResult(Ok(None)),
                )))
            }
            UpdateMessage::CheckResult(_result) => {
                // TODO: Handle update check result via dialog window
                Task::none()
            }
            UpdateMessage::StartInstall => {
                // TODO: Start actual installation
                Task::none()
            }
            UpdateMessage::InstallProgress(_progress) => {
                // TODO: Update progress via dialog window
                Task::none()
            }
            UpdateMessage::InstallComplete(_result) => {
                // TODO: Handle completion via dialog window
                Task::none()
            }
            UpdateMessage::RestartApp => {
                // Would restart the application
                Task::none()
            }
        }
    }
}
