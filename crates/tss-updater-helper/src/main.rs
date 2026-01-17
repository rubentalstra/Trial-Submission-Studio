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
//! 6. Helper verifies code signature of new bundle
//! 7. Helper performs atomic swap: current → backup, new → current
//! 8. Helper relaunches the application
//! 9. Helper cleans up backup on success

#[cfg(target_os = "macos")]
mod macos {
    use serde::Deserialize;
    use std::fs;
    use std::path::PathBuf;
    use std::process::{Command, ExitCode};
    use std::thread;
    use std::time::Duration;

    /// Configuration passed from the main application via config file
    #[derive(Debug, Deserialize)]
    pub struct HelperConfig {
        /// Path to the new .app bundle (in temp directory)
        pub new_app_path: PathBuf,
        /// Path to the current .app bundle to replace
        pub current_app_path: PathBuf,
        /// PID of the parent process to wait for
        pub parent_pid: u32,
    }

    /// Waits for the parent process to exit
    fn wait_for_parent(pid: u32) {
        eprintln!("[helper] Waiting for parent process {} to exit...", pid);

        // Check if process exists using kill -0
        for _ in 0..300 {
            // Max 30 seconds
            let status = Command::new("kill").args(["-0", &pid.to_string()]).output();

            match status {
                Ok(output) if !output.status.success() => {
                    // Process doesn't exist anymore
                    eprintln!("[helper] Parent process exited");
                    return;
                }
                _ => {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }

        eprintln!("[helper] Warning: Parent process may still be running");
    }

    /// Verifies the code signature of an app bundle
    fn verify_signature(app_path: &PathBuf) -> Result<(), String> {
        eprintln!("[helper] Verifying code signature: {:?}", app_path);

        let output = Command::new("codesign")
            .args(["--verify", "--deep", "--strict"])
            .arg(app_path)
            .output()
            .map_err(|e| format!("Failed to run codesign: {}", e))?;

        if output.status.success() {
            eprintln!("[helper] Code signature valid");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Code signature verification failed: {}", stderr))
        }
    }

    /// Performs the swap of app bundles using ditto (works across filesystems)
    fn swap_bundles(new_app: &PathBuf, current_app: &PathBuf) -> Result<PathBuf, String> {
        eprintln!(
            "[helper] Swapping bundles: {:?} -> {:?}",
            new_app, current_app
        );

        // Create backup path
        let backup_path = current_app.with_extension("app.backup");

        // Remove old backup if exists
        if backup_path.exists() {
            fs::remove_dir_all(&backup_path)
                .map_err(|e| format!("Failed to remove old backup: {}", e))?;
        }

        // Move current to backup (same filesystem, rename works)
        fs::rename(current_app, &backup_path)
            .map_err(|e| format!("Failed to move current app to backup: {}", e))?;
        eprintln!("[helper] Current app moved to backup: {:?}", backup_path);

        // Copy new app using ditto (works across filesystems, preserves metadata)
        let copy_result = Command::new("ditto").arg(new_app).arg(current_app).output();

        match copy_result {
            Ok(output) if output.status.success() => {
                eprintln!("[helper] New app installed: {:?}", current_app);

                // Clean up the source (temp) app bundle
                if let Err(e) = fs::remove_dir_all(new_app) {
                    eprintln!("[helper] Warning: Failed to clean up temp app: {}", e);
                }

                Ok(backup_path)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("[helper] ditto copy failed: {}", stderr);

                // Rollback: restore backup
                eprintln!("[helper] Rolling back...");
                if let Err(restore_err) = fs::rename(&backup_path, current_app) {
                    return Err(format!(
                        "Install failed and rollback failed: ditto: {} / restore: {}",
                        stderr, restore_err
                    ));
                }
                Err(format!("Failed to copy new app with ditto: {}", stderr))
            }
            Err(e) => {
                // Rollback: restore backup
                eprintln!("[helper] ditto command failed: {}", e);
                eprintln!("[helper] Rolling back...");
                if let Err(restore_err) = fs::rename(&backup_path, current_app) {
                    return Err(format!(
                        "Install failed and rollback failed: {} / {}",
                        e, restore_err
                    ));
                }
                Err(format!("Failed to run ditto: {}", e))
            }
        }
    }

    /// Relaunches the application
    fn relaunch(app_path: &PathBuf) -> Result<(), String> {
        eprintln!("[helper] Relaunching application: {:?}", app_path);

        Command::new("open")
            .arg(app_path)
            .spawn()
            .map_err(|e| format!("Failed to relaunch: {}", e))?;

        Ok(())
    }

    /// Cleans up the backup after successful update
    fn cleanup_backup(backup_path: &PathBuf) {
        eprintln!("[helper] Cleaning up backup: {:?}", backup_path);

        // Give the new app a moment to start
        thread::sleep(Duration::from_secs(2));

        if let Err(e) = fs::remove_dir_all(backup_path) {
            eprintln!("[helper] Warning: Failed to clean up backup: {}", e);
        } else {
            eprintln!("[helper] Backup cleaned up successfully");
        }
    }

    pub fn run() -> ExitCode {
        eprintln!("[helper] Trial Submission Studio Update Helper started");

        // Get config file path from command line argument
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 {
            eprintln!("[helper] Usage: {} <config_file>", args[0]);
            return ExitCode::FAILURE;
        }
        let config_path = &args[1];

        // Read config from file (persists after parent exits, avoiding race condition)
        let input = match fs::read_to_string(config_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "[helper] Failed to read config file '{}': {}",
                    config_path, e
                );
                return ExitCode::FAILURE;
            }
        };

        let config: HelperConfig = match serde_json::from_str(&input) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[helper] Failed to parse config: {}", e);
                return ExitCode::FAILURE;
            }
        };

        eprintln!("[helper] Config: {:?}", config);

        // Validate paths exist
        if !config.new_app_path.exists() {
            eprintln!("[helper] New app not found: {:?}", config.new_app_path);
            return ExitCode::FAILURE;
        }
        if !config.current_app_path.exists() {
            eprintln!(
                "[helper] Current app not found: {:?}",
                config.current_app_path
            );
            return ExitCode::FAILURE;
        }

        // Wait for parent to exit
        wait_for_parent(config.parent_pid);

        // Verify code signature
        if let Err(e) = verify_signature(&config.new_app_path) {
            eprintln!("[helper] Signature verification failed: {}", e);
            return ExitCode::FAILURE;
        }

        // Perform the swap
        let backup_path = match swap_bundles(&config.new_app_path, &config.current_app_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[helper] Bundle swap failed: {}", e);
                return ExitCode::FAILURE;
            }
        };

        // Relaunch the application
        if let Err(e) = relaunch(&config.current_app_path) {
            eprintln!("[helper] Relaunch failed: {}", e);
            // Don't fail completely - the update was installed
        }

        // Clean up backup
        cleanup_backup(&backup_path);

        eprintln!("[helper] Update complete!");
        ExitCode::SUCCESS
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
