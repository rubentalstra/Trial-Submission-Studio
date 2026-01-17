//! macOS Update Helper for Trial Submission Studio
//!
//! This is a minimal helper binary that performs the actual app bundle swap on macOS.
//! It is spawned by the main application after downloading an update.
//!
//! Process:
//! 1. Parent (main app) downloads new .app bundle to temp directory
//! 2. Parent writes JSON config to a file and spawns this helper with the file path
//! 3. Parent exits
//! 4. Helper reads config from file (avoids race condition with stdin)
//! 5. Helper waits for parent to exit
//! 6. Helper removes quarantine attribute from new bundle
//! 7. Helper verifies code signature of new bundle
//! 8. Helper performs atomic swap: current → backup, new → current
//! 9. Helper writes status file for post-update feedback
//! 10. Helper relaunches the application
//! 11. Helper cleans up backup on success

#[cfg(target_os = "macos")]
mod config;
#[cfg(target_os = "macos")]
mod launch;
#[cfg(target_os = "macos")]
mod log;
#[cfg(target_os = "macos")]
mod quarantine;
#[cfg(target_os = "macos")]
mod signature;
#[cfg(target_os = "macos")]
mod status;
#[cfg(target_os = "macos")]
mod swap;

#[cfg(target_os = "macos")]
mod macos {
    use crate::config::HelperConfig;
    use crate::launch::{relaunch, wait_for_parent};
    use crate::log::{get_log_path, init_logging, log, log_error};
    use crate::quarantine::remove_quarantine;
    use crate::signature::{get_team_id, verify_signature};
    use crate::status::UpdateStatus;
    use crate::swap::{cleanup_backup, swap_bundles};
    use std::path::PathBuf;
    use std::process::ExitCode;

    pub fn run() -> ExitCode {
        // Initialize logging first
        let log_path = match init_logging() {
            Ok(path) => {
                log(&format!("Log file: {:?}", path));
                path
            }
            Err(e) => {
                eprintln!("[helper] Failed to initialize logging: {}", e);
                // Continue without file logging
                PathBuf::from("/dev/null")
            }
        };

        log("Trial Submission Studio Update Helper started");

        // Get config file path from command line argument
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 {
            log_error("Usage", &format!("{} <config_file>", args[0]));
            return ExitCode::FAILURE;
        }
        let config_path = &args[1];
        log(&format!("Config file: {}", config_path));

        // Read and parse config
        let config = match HelperConfig::from_file(config_path) {
            Ok(c) => {
                log(&format!("Config loaded: {:?}", c));
                c
            }
            Err(e) => {
                let error_msg = log_error("Config", &e);
                write_failure_status("", "", &error_msg, &log_path);
                return ExitCode::FAILURE;
            }
        };

        // Validate paths exist
        if !config.new_app_path.exists() {
            let error_msg = log_error(
                "Validation",
                &format!("New app not found: {:?}", config.new_app_path),
            );
            write_failure_status(
                &config.version,
                &config.previous_version,
                &error_msg,
                &log_path,
            );
            return ExitCode::FAILURE;
        }
        if !config.current_app_path.exists() {
            let error_msg = log_error(
                "Validation",
                &format!("Current app not found: {:?}", config.current_app_path),
            );
            write_failure_status(
                &config.version,
                &config.previous_version,
                &error_msg,
                &log_path,
            );
            return ExitCode::FAILURE;
        }
        log("Paths validated");

        // Wait for parent to exit
        wait_for_parent(config.parent_pid);

        // Remove quarantine attribute
        if let Err(e) = remove_quarantine(&config.new_app_path) {
            log_error("Quarantine", &e);
            // Non-fatal - continue with installation
        }

        // Verify code signature
        if let Err(e) = verify_signature(&config.new_app_path) {
            let error_msg = log_error("Signature verification", &e);
            write_failure_status(
                &config.version,
                &config.previous_version,
                &error_msg,
                &log_path,
            );
            return ExitCode::FAILURE;
        }

        // Log team ID if available
        if let Some(team_id) = get_team_id(&config.new_app_path) {
            log(&format!("Code signature valid, Team ID: {}", team_id));
        } else {
            log("Code signature valid");
        }

        // Perform the bundle swap
        let swap_result = match swap_bundles(&config.new_app_path, &config.current_app_path) {
            Ok(result) => {
                log(&format!(
                    "Bundle swap complete, backup at: {:?}",
                    result.backup_path
                ));
                result
            }
            Err(e) => {
                let error_msg = log_error("Bundle swap", &e);
                write_failure_status(
                    &config.version,
                    &config.previous_version,
                    &error_msg,
                    &log_path,
                );
                return ExitCode::FAILURE;
            }
        };

        // Write success status file
        let final_log_path = get_log_path().unwrap_or(log_path);
        let status = UpdateStatus::success(
            config.version.clone(),
            config.previous_version.clone(),
            final_log_path,
        );
        if let Err(e) = status.write() {
            log_error("Status file", &e);
            // Non-fatal - continue with relaunch
        } else {
            log("Status file written");
        }

        // Relaunch the application
        if let Err(e) = relaunch(&config.current_app_path) {
            log_error("Relaunch", &e);
            // Don't fail completely - the update was installed
        } else {
            log("Application relaunch command sent");
        }

        // Clean up backup
        cleanup_backup(&swap_result.backup_path);

        log("Update complete!");
        ExitCode::SUCCESS
    }

    /// Writes a failure status file.
    fn write_failure_status(
        version: &str,
        previous_version: &str,
        error: &str,
        log_path: &PathBuf,
    ) {
        let status = UpdateStatus::failure(
            version.to_string(),
            previous_version.to_string(),
            error.to_string(),
            log_path.clone(),
        );
        if let Err(e) = status.write() {
            log_error("Failed to write failure status", &e);
        }
    }
}

#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    macos::run()
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("tss-updater-helper is only supported on macOS");
    std::process::exit(1);
}
