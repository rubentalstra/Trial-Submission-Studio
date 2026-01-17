//! Dialog message handlers.
//!
//! Handles:
//! - About dialog
//! - Settings dialog (all categories)
//! - Third-party licenses dialog
//! - Update check dialog (using Task::perform and Task::run for streaming)

use futures_util::StreamExt;
use iced::Task;
use iced::widget::markdown;
use iced::window;

use crate::app::App;
use crate::message::Message;
use crate::message::{
    AboutMessage, DeveloperSettingsMessage, DialogMessage, ExportSettingsMessage,
    GeneralSettingsMessage, SettingsMessage, ThirdPartyMessage, UpdateMessage,
    UpdateSettingsMessage, VerifyResult,
};
use crate::state::Settings;
use crate::view::dialog::update::UpdateState;

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
                    ExportSettingsMessage::SdtmIgVersionChanged(version) => {
                        self.state.settings.export.sdtm_ig_version = version;
                        tracing::info!("SDTM-IG version: {}", version);
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
                    UpdateSettingsMessage::CheckOnStartupToggled(enabled) => {
                        self.state.settings.updates.check_on_startup = enabled;
                        tracing::info!("Check on startup: {}", enabled);
                    }
                    UpdateSettingsMessage::ChannelChanged(channel) => {
                        self.state.settings.updates.channel = channel;
                        tracing::info!("Update channel changed to: {:?}", channel);
                    }
                    UpdateSettingsMessage::ClearSkippedVersion => {
                        self.state.settings.updates.clear_skipped_version();
                        tracing::info!("Cleared skipped version");
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
            ThirdPartyMessage::Close => {
                if let Some((id, _)) = self.state.dialog_windows.third_party.take() {
                    return window::close(id);
                }
                Task::none()
            }
            ThirdPartyMessage::ScrollTo(_position) => {
                // Handle scroll - would need scrollable state
                Task::none()
            }
        }
    }

    /// Handle update dialog messages.
    ///
    /// This handler processes user actions and async operation results.
    fn handle_update_message(&mut self, msg: UpdateMessage) -> Task<Message> {
        match msg {
            // -------------------------------------------------------------------------
            // User Actions
            // -------------------------------------------------------------------------
            UpdateMessage::Open => {
                // Window opening is handled by the app's view routing
                Task::none()
            }

            UpdateMessage::Close => {
                // Close dialog window in multi-window mode
                if let Some((id, _)) = self.state.dialog_windows.update.take() {
                    return window::close(id);
                }
                Task::none()
            }

            UpdateMessage::CheckForUpdates => {
                // Update state to Checking
                if let Some((id, _)) = self.state.dialog_windows.update {
                    self.state.dialog_windows.update = Some((id, UpdateState::Checking));
                }

                // Spawn async check using Task::perform
                let settings = self.state.settings.updates.clone();
                Task::perform(
                    async move {
                        tss_updater::check_for_update(&settings)
                            .await
                            .map_err(|e| e.user_message().to_string())
                    },
                    |result| {
                        Message::Dialog(DialogMessage::Update(UpdateMessage::CheckComplete(result)))
                    },
                )
            }

            UpdateMessage::ConfirmDownload => {
                // Get update info from current state
                let info = if let Some((_, UpdateState::Available { info, .. })) =
                    &self.state.dialog_windows.update
                {
                    info.clone()
                } else if let Some((_, UpdateState::VerificationFailed { info, .. })) =
                    &self.state.dialog_windows.update
                {
                    // Retry download after verification failure
                    info.clone()
                } else {
                    return Task::none();
                };

                // Update state to Downloading with initial progress
                if let Some((id, _)) = self.state.dialog_windows.update {
                    self.state.dialog_windows.update = Some((
                        id,
                        UpdateState::Downloading {
                            info: info.clone(),
                            progress: 0.0,
                            downloaded_bytes: 0,
                            total_bytes: info.asset.size,
                            speed: 0,
                        },
                    ));
                }

                // Use Task::run() for streaming download with progress
                let url = info.asset.download_url.clone();
                let total = info.asset.size;

                // Map stream items to messages (url is passed as owned String)
                let stream = tss_updater::download_with_data(url, total).map(
                    |item: Result<tss_updater::DownloadStreamItem, tss_updater::UpdateError>| {
                        match item {
                            Ok(tss_updater::DownloadStreamItem::Progress(progress)) => {
                                Message::Dialog(DialogMessage::Update(
                                    UpdateMessage::DownloadProgress(progress),
                                ))
                            }
                            Ok(tss_updater::DownloadStreamItem::Complete(result)) => {
                                Message::Dialog(DialogMessage::Update(
                                    UpdateMessage::DownloadComplete(Ok(result)),
                                ))
                            }
                            Err(e) => Message::Dialog(DialogMessage::Update(
                                UpdateMessage::DownloadComplete(Err(e.user_message().to_string())),
                            )),
                        }
                    },
                );

                Task::run(stream, std::convert::identity)
            }

            UpdateMessage::ConfirmInstall => {
                // Get data from current state
                let (info, data) = if let Some((
                    _,
                    UpdateState::ReadyToInstall {
                        info,
                        data,
                        verified: _,
                    },
                )) = &self.state.dialog_windows.update
                {
                    (info.clone(), data.clone())
                } else {
                    return Task::none();
                };

                // Update state to Installing
                if let Some((id, _)) = self.state.dialog_windows.update {
                    self.state.dialog_windows.update = Some((id, UpdateState::Installing));
                }

                // Spawn async installation
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            tss_updater::install_and_restart(&data, &info)
                        })
                        .await
                        .map_err(|e| format!("Installation task failed: {}", e))?
                        .map_err(|e| e.user_message().to_string())
                    },
                    |result| {
                        Message::Dialog(DialogMessage::Update(UpdateMessage::InstallComplete(
                            result,
                        )))
                    },
                )
            }

            UpdateMessage::Restart => {
                // Attempt to restart the application
                if let Err(e) = tss_updater::restart() {
                    tracing::error!("Failed to restart application: {}", e);
                    if let Some((id, _)) = self.state.dialog_windows.update {
                        self.state.dialog_windows.update =
                            Some((id, UpdateState::Error(format!("Failed to restart: {}", e))));
                    }
                }
                Task::none()
            }

            UpdateMessage::SkipVersion(version) => {
                // Save skipped version
                self.state.settings.updates.skip_version(&version);
                let _ = self.state.settings.save();
                tracing::info!("Skipped version: {}", version);

                // Close dialog
                if let Some((id, _)) = self.state.dialog_windows.update.take() {
                    return window::close(id);
                }
                Task::none()
            }

            UpdateMessage::RemindLater => {
                // Just close the dialog
                if let Some((id, _)) = self.state.dialog_windows.update.take() {
                    return window::close(id);
                }
                Task::none()
            }

            UpdateMessage::Cancel => {
                // Cancel current operation and close dialog
                if let Some((id, _)) = self.state.dialog_windows.update.take() {
                    return window::close(id);
                }
                Task::none()
            }

            // -------------------------------------------------------------------------
            // Async Operation Results
            // -------------------------------------------------------------------------
            UpdateMessage::CheckComplete(result) => {
                // Record that we checked for updates
                self.state.settings.updates.record_check();

                match result {
                    Ok(Some(info)) => {
                        // Parse changelog into markdown items
                        let changelog_items: Vec<markdown::Item> =
                            markdown::parse(&info.changelog).collect();

                        if let Some((id, _)) = self.state.dialog_windows.update {
                            self.state.dialog_windows.update = Some((
                                id,
                                UpdateState::Available {
                                    info,
                                    changelog_items,
                                },
                            ));
                        }
                    }
                    Ok(None) => {
                        if let Some((id, _)) = self.state.dialog_windows.update {
                            self.state.dialog_windows.update = Some((id, UpdateState::UpToDate));
                        }
                    }
                    Err(e) => {
                        tracing::error!("Update check failed: {}", e);
                        if let Some((id, _)) = self.state.dialog_windows.update {
                            self.state.dialog_windows.update = Some((id, UpdateState::Error(e)));
                        }
                    }
                }
                Task::none()
            }

            UpdateMessage::DownloadProgress(progress) => {
                // Update download progress in state
                if let Some((id, UpdateState::Downloading { info, .. })) =
                    &self.state.dialog_windows.update
                {
                    self.state.dialog_windows.update = Some((
                        *id,
                        UpdateState::Downloading {
                            info: info.clone(),
                            progress: progress.fraction(),
                            downloaded_bytes: progress.downloaded,
                            total_bytes: progress.total,
                            speed: progress.speed,
                        },
                    ));
                }
                Task::none()
            }

            UpdateMessage::DownloadComplete(result) => {
                match result {
                    Ok(download_result) => {
                        // Get info from current state
                        let info = if let Some((_, UpdateState::Downloading { info, .. })) =
                            &self.state.dialog_windows.update
                        {
                            info.clone()
                        } else {
                            return Task::none();
                        };

                        // Update state to Verifying
                        if let Some((id, _)) = self.state.dialog_windows.update {
                            self.state.dialog_windows.update =
                                Some((id, UpdateState::Verifying { info: info.clone() }));
                        }

                        // Spawn verification task
                        let data = download_result.data;
                        let expected_digest = info.asset.digest.clone();

                        Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    match expected_digest {
                                        Some(expected) => {
                                            match tss_updater::verify_sha256(&data, &expected) {
                                                Ok(hash) => Ok((
                                                    VerifyResult {
                                                        verified: true,
                                                        sha256: Some(hash),
                                                    },
                                                    data,
                                                )),
                                                Err(
                                                    tss_updater::UpdateError::ChecksumMismatch {
                                                        expected,
                                                        actual,
                                                    },
                                                ) => Err(format!(
                                                    "Checksum mismatch: expected {}, got {}",
                                                    &expected[..8.min(expected.len())],
                                                    &actual[..8.min(actual.len())]
                                                )),
                                                Err(e) => Err(e.user_message().to_string()),
                                            }
                                        }
                                        None => {
                                            // No digest available, skip verification
                                            Ok((
                                                VerifyResult {
                                                    verified: false,
                                                    sha256: None,
                                                },
                                                data,
                                            ))
                                        }
                                    }
                                })
                                .await
                                .map_err(|e| format!("Verification task failed: {}", e))?
                            },
                            move |result| match result {
                                Ok((verify_result, data)) => Message::UpdateReadyToInstall {
                                    info,
                                    data,
                                    verified: verify_result.verified,
                                },
                                Err(e) => Message::Dialog(DialogMessage::Update(
                                    UpdateMessage::VerifyComplete(Err(e)),
                                )),
                            },
                        )
                    }
                    Err(e) => {
                        tracing::error!("Download failed: {}", e);
                        if let Some((id, _)) = self.state.dialog_windows.update {
                            self.state.dialog_windows.update = Some((id, UpdateState::Error(e)));
                        }
                        Task::none()
                    }
                }
            }

            UpdateMessage::VerifyComplete(result) => {
                match result {
                    Ok(_verify_result) => {
                        // This path is handled by Message::UpdateReadyToInstall
                        tracing::debug!("Verification complete (handled elsewhere)");
                    }
                    Err(e) => {
                        tracing::error!("Verification failed: {}", e);

                        // Get info for retry option
                        let info = if let Some((_, UpdateState::Verifying { info })) =
                            &self.state.dialog_windows.update
                        {
                            info.clone()
                        } else {
                            if let Some((id, _)) = self.state.dialog_windows.update {
                                self.state.dialog_windows.update =
                                    Some((id, UpdateState::Error(e)));
                            }
                            return Task::none();
                        };

                        if let Some((id, _)) = self.state.dialog_windows.update {
                            self.state.dialog_windows.update = Some((
                                id,
                                UpdateState::VerificationFailed {
                                    info,
                                    expected: "unknown".to_string(),
                                    actual: "unknown".to_string(),
                                },
                            ));
                        }
                    }
                }
                Task::none()
            }

            UpdateMessage::InstallComplete(result) => {
                match result {
                    Ok(()) => {
                        // Get version from previous state
                        let version = if let Some((_, UpdateState::Installing)) =
                            &self.state.dialog_windows.update
                        {
                            // We don't have version here, use a generic message
                            "new version".to_string()
                        } else {
                            "new version".to_string()
                        };

                        if let Some((id, _)) = self.state.dialog_windows.update {
                            self.state.dialog_windows.update =
                                Some((id, UpdateState::InstallComplete { version }));
                        }
                    }
                    Err(e) => {
                        tracing::error!("Installation failed: {}", e);
                        if let Some((id, _)) = self.state.dialog_windows.update {
                            self.state.dialog_windows.update = Some((id, UpdateState::Error(e)));
                        }
                    }
                }
                Task::none()
            }
        }
    }

    /// Set update state to ReadyToInstall (called from download completion).
    pub fn set_update_ready_to_install(
        &mut self,
        info: tss_updater::UpdateInfo,
        data: Vec<u8>,
        verified: bool,
    ) {
        if let Some((id, _)) = self.state.dialog_windows.update {
            self.state.dialog_windows.update = Some((
                id,
                UpdateState::ReadyToInstall {
                    info,
                    data,
                    verified,
                },
            ));
        }
    }
}
