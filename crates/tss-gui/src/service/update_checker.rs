//! Update checking service.
//!
//! Provides async functions for checking, downloading, and installing
//! application updates using Iced's `Task::perform` pattern.

use iced::Task;

use crate::message::{DialogMessage, Message, UpdateMessage, VerificationResult};

/// Check for available updates.
///
/// Returns a Task that will produce a `CheckComplete` message.
pub fn check_for_updates(settings: tss_updater::UpdateSettings) -> Task<Message> {
    Task::perform(
        async move {
            tss_updater::UpdateService::check_for_update(&settings)
                .await
                .map_err(|e| e.user_message().to_string())
        },
        |result| Message::Dialog(DialogMessage::Update(UpdateMessage::CheckComplete(result))),
    )
}

/// Download an update with progress reporting.
///
/// Returns a Task that will produce a `DownloadComplete` message.
pub fn download_update(info: tss_updater::UpdateInfo) -> Task<Message> {
    Task::perform(
        async move {
            tss_updater::UpdateService::download_update(&info, |_progress| {
                // Progress updates are handled via separate mechanism
                // For now, we don't send progress updates
            })
            .await
            .map_err(|e| e.user_message().to_string())
        },
        |result| {
            Message::Dialog(DialogMessage::Update(UpdateMessage::DownloadComplete(
                result,
            )))
        },
    )
}

/// Verify downloaded data and transition to ready-to-install state.
///
/// Returns a Task that will update the state based on verification result.
pub fn verify_update(data: Vec<u8>, info: tss_updater::UpdateInfo) -> Task<Message> {
    Task::perform(
        async move {
            // Perform verification
            let status = tss_updater::UpdateService::verify_download(&data, &info);

            match status {
                tss_updater::VerificationStatus::Verified => {
                    // Return data and info for installation
                    Ok((data, info, true))
                }
                tss_updater::VerificationStatus::Failed { expected, actual } => {
                    // Verification failed
                    Err(VerificationResult::Failed { expected, actual })
                }
                tss_updater::VerificationStatus::Unavailable => {
                    // No digest available, allow with warning
                    Ok((data, info, false))
                }
            }
        },
        |result| match result {
            Ok((data, info, verified)) => {
                // Need to update state to ReadyToInstall
                // This is a bit awkward - we need to pass data through
                Message::UpdateReadyToInstall {
                    info,
                    data,
                    verified,
                }
            }
            Err(verification_result) => Message::Dialog(DialogMessage::Update(
                UpdateMessage::VerificationStatus(verification_result),
            )),
        },
    )
}

/// Install the downloaded update and restart the application.
///
/// This function handles all platform-specific details:
/// - On macOS: Spawns a helper to swap app bundles, then exits
/// - On Windows/Linux: Replaces the binary and restarts
///
/// On success, this function does not return (the app exits/restarts).
/// Returns a Task that will produce an `InstallFailed` message only on error.
pub fn install_and_restart(data: Vec<u8>, info: tss_updater::UpdateInfo) -> Task<Message> {
    Task::perform(
        async move {
            // Run installation in blocking task
            tokio::task::spawn_blocking(move || {
                tss_updater::UpdateService::install_and_restart(&data, &info)
            })
            .await
            .map_err(|e| format!("Installation task failed: {}", e))?
            .map_err(|e| e.user_message().to_string())
        },
        |result| {
            // On macOS: never reached (process exits)
            // On Windows/Linux: never reached (process restarts)
            // Only reached on error
            match result {
                Ok(()) => {
                    // Should not reach here, but handle gracefully
                    Message::Dialog(DialogMessage::Update(UpdateMessage::InstallComplete(
                        Ok(()),
                    )))
                }
                Err(e) => Message::Dialog(DialogMessage::Update(UpdateMessage::InstallComplete(
                    Err(e),
                ))),
            }
        },
    )
}

/// Install the downloaded update (legacy method, kept for compatibility).
///
/// Prefer using `install_and_restart` for new code.
///
/// Returns a Task that will produce an `InstallComplete` message.
pub fn install_update(data: Vec<u8>, info: tss_updater::UpdateInfo) -> Task<Message> {
    Task::perform(
        async move {
            // Run installation in blocking task
            tokio::task::spawn_blocking(move || {
                tss_updater::UpdateService::install_update(&data, &info)
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_version_comparison() {
        // Simple helper for testing
        let is_newer = |new: &str, current: &str| -> bool {
            let parse = |v: &str| -> (u32, u32, u32) {
                let v = v.trim_start_matches('v');
                let parts: Vec<&str> = v.split('.').collect();
                let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                (major, minor, patch)
            };

            let (new_major, new_minor, new_patch) = parse(new);
            let (cur_major, cur_minor, cur_patch) = parse(current);

            (new_major, new_minor, new_patch) > (cur_major, cur_minor, cur_patch)
        };

        assert!(is_newer("1.0.0", "0.9.0"));
        assert!(is_newer("0.10.0", "0.9.0"));
        assert!(is_newer("0.0.2", "0.0.1"));
        assert!(!is_newer("0.0.1", "0.0.2"));
        assert!(!is_newer("1.0.0", "1.0.0"));
    }
}
