//! Update checking service.
//!
//! Provides async functions for checking application updates using
//! Iced's `Task::perform` pattern.

use crate::message::UpdateInfo;

/// Check for available updates.
///
/// Returns `Ok(Some(UpdateInfo))` if an update is available,
/// `Ok(None)` if the application is up to date,
/// or `Err(message)` if the check failed.
pub async fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    // Run the check in a blocking task to avoid blocking the async runtime
    tokio::task::spawn_blocking(check_for_updates_sync)
        .await
        .map_err(|e| format!("Update check task failed: {}", e))?
}

/// Synchronous update check implementation.
fn check_for_updates_sync() -> Result<Option<UpdateInfo>, String> {
    // Create default update settings
    let settings = tss_updater::UpdateSettings::default();

    // Check for updates using tss_updater
    match tss_updater::UpdateService::check_for_update(&settings) {
        Ok(Some(info)) => {
            // Convert tss_updater::UpdateInfo to crate::message::UpdateInfo

            //  TODO: look at tss_updater to really integrate this properly.
            Ok(Some(UpdateInfo {
                version: info.version_display().to_string(),
                changelog: info.changelog,
                download_url: String::new(), // Not provided by tss_updater
            }))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
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
