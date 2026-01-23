//! Dialog message handler.
//!
//! Handles:
//! - About dialog
//! - Settings dialog (all categories)
//! - Third-party licenses dialog
//! - Update check dialog (using Task::perform and Task::run for streaming)

use std::sync::Arc;

use futures_util::StreamExt;
use iced::Task;
use iced::widget::markdown;
use iced::window;

use crate::handler::MessageHandler;
use crate::message::Message;
use crate::message::{
    AboutMessage, DeveloperSettingsMessage, DialogMessage, ExportSettingsMessage,
    GeneralSettingsMessage, SettingsMessage, ThirdPartyMessage, UpdateMessage,
    UpdateSettingsMessage, VerifyOutcome,
};
use crate::state::{AppState, Settings};
use crate::view::dialog::update::{
    DownloadStats, RetryContext, UpdateErrorInfo, UpdateState, parse_changelog,
};

/// Handler for dialog-related messages.
pub struct DialogHandler;

impl MessageHandler<DialogMessage> for DialogHandler {
    fn handle(&self, state: &mut AppState, msg: DialogMessage) -> Task<Message> {
        match msg {
            DialogMessage::About(about_msg) => handle_about_message(state, about_msg),
            DialogMessage::Settings(settings_msg) => handle_settings_message(state, settings_msg),
            DialogMessage::ThirdParty(tp_msg) => handle_third_party_message(state, tp_msg),
            DialogMessage::Update(update_msg) => handle_update_message(state, update_msg),
            DialogMessage::CloseAll => Task::none(),
        }
    }
}

/// Handle about dialog messages.
#[allow(clippy::needless_pass_by_value)]
fn handle_about_message(state: &mut AppState, msg: AboutMessage) -> Task<Message> {
    match msg {
        AboutMessage::Open => Task::none(),
        AboutMessage::Close => {
            // Close dialog window in multi-window mode
            if let Some(id) = state.dialog_windows.about.take() {
                return window::close(id);
            }
            Task::none()
        }
        AboutMessage::CopyAndClose => {
            // Copy system info to clipboard using Iced's clipboard
            let info = crate::view::dialog::about::generate_system_info();
            let copy_task = iced::clipboard::write(info);

            // Close dialog window in multi-window mode
            if let Some(id) = state.dialog_windows.about.take() {
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
fn handle_settings_message(state: &mut AppState, msg: SettingsMessage) -> Task<Message> {
    match msg {
        SettingsMessage::Open => Task::none(),
        SettingsMessage::Close => {
            // Close dialog window in multi-window mode
            if let Some((id, _)) = state.dialog_windows.settings.take() {
                return window::close(id);
            }
            Task::none()
        }
        SettingsMessage::Apply => {
            // Save settings
            let _ = state.settings.save();
            tracing::info!("Settings saved");

            // Close dialog window in multi-window mode
            if let Some((id, _)) = state.dialog_windows.settings.take() {
                return window::close(id);
            }
            Task::none()
        }
        SettingsMessage::ResetToDefaults => {
            state.settings = Settings::default();
            Task::none()
        }
        SettingsMessage::CategorySelected(category) => {
            // Update dialog_windows.settings for multi-window mode
            if let Some((id, _)) = state.dialog_windows.settings {
                state.dialog_windows.settings = Some((id, category));
            }
            Task::none()
        }
        SettingsMessage::General(general_msg) => {
            match general_msg {
                GeneralSettingsMessage::CtVersionChanged(_version) => {
                    // CT version change - would reload terminology
                }
                GeneralSettingsMessage::HeaderRowsChanged(rows) => {
                    state.settings.general.header_rows = rows;
                }
                GeneralSettingsMessage::ConfidenceThresholdChanged(threshold) => {
                    state.settings.general.mapping_confidence_threshold = threshold;
                }
                GeneralSettingsMessage::AssignmentModeChanged(mode) => {
                    state.settings.general.assignment_mode = mode;
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
                    state.settings.developer.bypass_validation = enabled;
                    tracing::info!("Bypass validation: {}", enabled);
                }
                DeveloperSettingsMessage::DeveloperModeToggled(enabled) => {
                    state.settings.developer.developer_mode = enabled;
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
                    state.settings.export.default_format = format;
                }
                ExportSettingsMessage::DefaultXptVersionChanged(version) => {
                    state.settings.export.xpt_version = version;
                }
                ExportSettingsMessage::SdtmIgVersionChanged(version) => {
                    state.settings.export.ig_version = version;
                    tracing::info!("IG version: {}", version);
                }
            }
            Task::none()
        }
        SettingsMessage::Display(display_msg) => {
            use crate::message::DisplaySettingsMessage;
            use crate::theme::ThemeConfig;

            match display_msg {
                DisplaySettingsMessage::PreviewRowsChanged(rows) => {
                    state.settings.display.preview_rows_per_page = rows;
                    tracing::info!("Preview rows per page: {}", rows);
                }
                DisplaySettingsMessage::ThemeModeChanged(mode) => {
                    state.settings.display.theme_mode = mode;
                    state.theme_config =
                        ThemeConfig::new(mode, state.settings.display.accessibility_mode);
                    tracing::info!("Theme mode: {}", mode.label());
                }
                DisplaySettingsMessage::AccessibilityModeChanged(mode) => {
                    state.settings.display.accessibility_mode = mode;
                    state.theme_config = ThemeConfig::new(state.settings.display.theme_mode, mode);
                    tracing::info!("Accessibility mode: {}", mode.label());
                }
            }
            Task::none()
        }
        SettingsMessage::Updates(update_msg) => {
            match update_msg {
                UpdateSettingsMessage::CheckOnStartupToggled(enabled) => {
                    state.settings.updates.check_on_startup = enabled;
                    tracing::info!("Check on startup: {}", enabled);
                }
                UpdateSettingsMessage::ChannelChanged(channel) => {
                    state.settings.updates.channel = channel;
                    tracing::info!("Update channel changed to: {:?}", channel);
                }
                UpdateSettingsMessage::ClearSkippedVersion => {
                    state.settings.updates.clear_skipped_version();
                    tracing::info!("Cleared skipped version");
                }
            }
            Task::none()
        }
    }
}

/// Handle third-party licenses dialog messages.
#[allow(clippy::needless_pass_by_value)]
fn handle_third_party_message(state: &mut AppState, msg: ThirdPartyMessage) -> Task<Message> {
    match msg {
        ThirdPartyMessage::Open => Task::none(),
        ThirdPartyMessage::Close => {
            if let Some((id, _)) = state.dialog_windows.third_party.take() {
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
/// This handler manages the update dialog state machine with the following states:
/// - Checking: Initial state when dialog opens, check runs automatically
/// - Available: Update found with collapsible changelog
/// - UpToDate: No update available
/// - Downloading: Download in progress with stats
/// - Verifying: Verification in progress
/// - ReadyToInstall: Second confirmation before install
/// - Installing: Installation in progress
/// - Complete: Restart required
/// - Error: With retry context
fn handle_update_message(state: &mut AppState, msg: UpdateMessage) -> Task<Message> {
    match msg {
        // =================================================================
        // User Actions
        // =================================================================
        UpdateMessage::Open => {
            // Dialog opens in Checking state and triggers check automatically
            // State is set when window is opened via DialogWindowOpened message
            // Here we just start the check task
            let settings = state.settings.updates.clone();
            Task::perform(
                async move {
                    tss_updater::check_for_update(&settings)
                        .await
                        .map_err(|e| e.user_message().to_string())
                },
                |result| Message::Dialog(DialogMessage::Update(UpdateMessage::CheckResult(result))),
            )
        }

        UpdateMessage::Close => {
            if let Some((id, _)) = state.dialog_windows.update.take() {
                return window::close(id);
            }
            Task::none()
        }

        UpdateMessage::ConfirmDownload => {
            // Get update info from current state
            let info = match &state.dialog_windows.update {
                Some((_, UpdateState::Available { info, .. })) => info.clone(),
                Some((
                    _,
                    UpdateState::Error {
                        retry_context: Some(RetryContext::Download { info }),
                        ..
                    },
                )) => info.clone(),
                _ => return Task::none(),
            };

            // Update state to Downloading
            if let Some((id, _)) = state.dialog_windows.update {
                state.dialog_windows.update = Some((
                    id,
                    UpdateState::Downloading {
                        info: info.clone(),
                        stats: DownloadStats::new(info.asset.size),
                    },
                ));
            }

            // Stream download with progress
            let url = info.asset.download_url.clone();
            let total = info.asset.size;

            let stream = tss_updater::download_with_data(url, total).map(
                |item: Result<tss_updater::DownloadStreamItem, tss_updater::UpdateError>| match item
                {
                    Ok(tss_updater::DownloadStreamItem::Progress(progress)) => Message::Dialog(
                        DialogMessage::Update(UpdateMessage::DownloadProgress(progress)),
                    ),
                    Ok(tss_updater::DownloadStreamItem::Complete(result)) => Message::Dialog(
                        DialogMessage::Update(UpdateMessage::DownloadComplete(Ok(result))),
                    ),
                    Err(e) => Message::Dialog(DialogMessage::Update(
                        UpdateMessage::DownloadComplete(Err(e.user_message().to_string())),
                    )),
                },
            );

            Task::run(stream, std::convert::identity)
        }

        UpdateMessage::ConfirmInstall => {
            // Get data from current state (Arc::clone is cheap for large binaries)
            let (info, data) = match &state.dialog_windows.update {
                Some((_, UpdateState::ReadyToInstall { info, data, .. })) => {
                    (info.clone(), Arc::clone(data))
                }
                Some((
                    _,
                    UpdateState::Error {
                        retry_context: Some(RetryContext::Install { info, data }),
                        ..
                    },
                )) => (info.clone(), Arc::clone(data)),
                _ => return Task::none(),
            };

            // Update state to Installing
            if let Some((id, _)) = state.dialog_windows.update {
                state.dialog_windows.update =
                    Some((id, UpdateState::Installing { info: info.clone() }));
            }

            // Spawn async installation
            let version = info.version.clone();
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        tss_updater::install_and_restart(&data, &info)
                    })
                    .await
                    .map_err(|e| format!("Installation task failed: {}", e))?
                    .map(|_| version)
                    .map_err(|e| e.user_message().to_string())
                },
                |result| {
                    Message::Dialog(DialogMessage::Update(UpdateMessage::InstallResult(result)))
                },
            )
        }

        UpdateMessage::RestartNow => {
            if let Err(e) = tss_updater::restart() {
                tracing::error!("Failed to restart application: {}", e);
                if let Some((id, _)) = state.dialog_windows.update {
                    state.dialog_windows.update = Some((
                        id,
                        UpdateState::Error {
                            error: UpdateErrorInfo::from_message(format!(
                                "Failed to restart: {}",
                                e
                            )),
                            retry_context: None,
                        },
                    ));
                }
            }
            Task::none()
        }

        UpdateMessage::SkipVersion => {
            // Get version from current state and skip it
            if let Some((_, UpdateState::Available { info, .. })) = &state.dialog_windows.update {
                state.settings.updates.skip_version(&info.version);
                let _ = state.settings.save();
                tracing::info!("Skipped version: {}", info.version);
            }

            // Close dialog
            if let Some((id, _)) = state.dialog_windows.update.take() {
                return window::close(id);
            }
            Task::none()
        }

        UpdateMessage::ToggleChangelog => {
            // Toggle changelog expanded state
            if let Some((
                id,
                UpdateState::Available {
                    info,
                    changelog_items,
                    changelog_expanded,
                },
            )) = state.dialog_windows.update.take()
            {
                state.dialog_windows.update = Some((
                    id,
                    UpdateState::Available {
                        info,
                        changelog_items,
                        changelog_expanded: !changelog_expanded,
                    },
                ));
            }
            Task::none()
        }

        UpdateMessage::Retry => {
            // Get retry context from current error state
            let retry_context = match &state.dialog_windows.update {
                Some((_, UpdateState::Error { retry_context, .. })) => retry_context.clone(),
                _ => return Task::none(),
            };

            match retry_context {
                Some(RetryContext::Check) => {
                    // Retry the check
                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((id, UpdateState::Checking));
                    }

                    let settings = state.settings.updates.clone();
                    Task::perform(
                        async move {
                            tss_updater::check_for_update(&settings)
                                .await
                                .map_err(|e| e.user_message().to_string())
                        },
                        |result| {
                            Message::Dialog(DialogMessage::Update(UpdateMessage::CheckResult(
                                result,
                            )))
                        },
                    )
                }
                Some(RetryContext::Download { info }) => {
                    // Retry download - dispatch to ConfirmDownload
                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((
                            id,
                            UpdateState::Available {
                                changelog_items: parse_changelog(&info.changelog),
                                info,
                                changelog_expanded: false,
                            },
                        ));
                    }
                    handle_update_message(state, UpdateMessage::ConfirmDownload)
                }
                Some(RetryContext::Install { info, data }) => {
                    // Retry install - dispatch to ConfirmInstall
                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((
                            id,
                            UpdateState::ReadyToInstall {
                                info,
                                data,
                                verified: false,
                            },
                        ));
                    }
                    handle_update_message(state, UpdateMessage::ConfirmInstall)
                }
                None => Task::none(),
            }
        }

        // =================================================================
        // Async Operation Results
        // =================================================================
        UpdateMessage::CheckResult(result) => {
            // Record that we checked
            state.settings.updates.record_check();

            match result {
                Ok(Some(info)) => {
                    let changelog_items: Vec<markdown::Item> =
                        markdown::parse(&info.changelog).collect();

                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((
                            id,
                            UpdateState::Available {
                                info,
                                changelog_items,
                                changelog_expanded: false,
                            },
                        ));
                    }
                }
                Ok(None) => {
                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((id, UpdateState::UpToDate));
                    }
                }
                Err(e) => {
                    tracing::error!("Update check failed: {}", e);
                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((
                            id,
                            UpdateState::Error {
                                error: UpdateErrorInfo::from_message(e),
                                retry_context: Some(RetryContext::Check),
                            },
                        ));
                    }
                }
            }
            Task::none()
        }

        UpdateMessage::DownloadProgress(progress) => {
            // Update download stats
            if let Some((id, UpdateState::Downloading { info, stats })) =
                &mut state.dialog_windows.update
            {
                stats.update(progress.downloaded, progress.speed);
                // Clone to update state properly
                let id = *id;
                let info = info.clone();
                let mut stats = stats.clone();
                stats.update(progress.downloaded, progress.speed);
                state.dialog_windows.update = Some((id, UpdateState::Downloading { info, stats }));
            }
            Task::none()
        }

        UpdateMessage::DownloadComplete(result) => {
            match result {
                Ok(download_result) => {
                    // Get info from current state
                    let info = match &state.dialog_windows.update {
                        Some((_, UpdateState::Downloading { info, .. })) => info.clone(),
                        _ => return Task::none(),
                    };

                    // Update state to Verifying
                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update =
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
                                            Ok(_) => Ok(VerifyOutcome {
                                                verified: true,
                                                data,
                                            }),
                                            Err(tss_updater::UpdateError::ChecksumMismatch {
                                                expected,
                                                actual,
                                            }) => Err(format!(
                                                "Checksum mismatch: expected {}..., got {}...",
                                                &expected[..8.min(expected.len())],
                                                &actual[..8.min(actual.len())]
                                            )),
                                            Err(e) => Err(e.user_message().to_string()),
                                        }
                                    }
                                    None => {
                                        // No digest available, skip verification
                                        Ok(VerifyOutcome {
                                            verified: false,
                                            data,
                                        })
                                    }
                                }
                            })
                            .await
                            .map_err(|e| format!("Verification task failed: {}", e))?
                        },
                        move |result| match result {
                            Ok(outcome) => Message::Dialog(DialogMessage::Update(
                                UpdateMessage::VerifyResult(Ok(outcome)),
                            )),
                            Err(e) => Message::Dialog(DialogMessage::Update(
                                UpdateMessage::VerifyResult(Err(e)),
                            )),
                        },
                    )
                }
                Err(e) => {
                    tracing::error!("Download failed: {}", e);

                    // Get info for retry
                    let info = match &state.dialog_windows.update {
                        Some((_, UpdateState::Downloading { info, .. })) => Some(info.clone()),
                        _ => None,
                    };

                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((
                            id,
                            UpdateState::Error {
                                error: UpdateErrorInfo::from_message(e),
                                retry_context: info.map(|i| RetryContext::Download { info: i }),
                            },
                        ));
                    }
                    Task::none()
                }
            }
        }

        UpdateMessage::VerifyResult(result) => {
            // Get info from Verifying state
            let info = match &state.dialog_windows.update {
                Some((_, UpdateState::Verifying { info })) => info.clone(),
                _ => return Task::none(),
            };

            match result {
                Ok(outcome) => {
                    // Move to ReadyToInstall (wrap data in Arc for cheap cloning)
                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((
                            id,
                            UpdateState::ReadyToInstall {
                                info,
                                data: Arc::new(outcome.data),
                                verified: outcome.verified,
                            },
                        ));
                    }
                }
                Err(e) => {
                    tracing::error!("Verification failed: {}", e);
                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((
                            id,
                            UpdateState::Error {
                                error: UpdateErrorInfo::from_message(e),
                                retry_context: Some(RetryContext::Download { info }),
                            },
                        ));
                    }
                }
            }
            Task::none()
        }

        UpdateMessage::InstallResult(result) => {
            match result {
                Ok(version) => {
                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((id, UpdateState::Complete { version }));
                    }
                }
                Err(e) => {
                    tracing::error!("Installation failed: {}", e);

                    // Get info for retry if available (data is lost at this point)
                    let retry_context = match &state.dialog_windows.update {
                        Some((_, UpdateState::Installing { info })) => {
                            Some(RetryContext::Install {
                                info: info.clone(),
                                data: Arc::new(vec![]), // Data is not available after install attempt
                            })
                        }
                        _ => None,
                    };

                    if let Some((id, _)) = state.dialog_windows.update {
                        state.dialog_windows.update = Some((
                            id,
                            UpdateState::Error {
                                error: UpdateErrorInfo::from_message(e),
                                retry_context,
                            },
                        ));
                    }
                }
            }
            Task::none()
        }
    }
}
