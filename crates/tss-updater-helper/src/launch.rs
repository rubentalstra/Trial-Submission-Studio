//! Application relaunch functionality.

use std::path::Path;
use std::process::Command;

/// Relaunches the application using the `open` command.
pub fn relaunch(app_path: &Path) -> Result<(), String> {
    eprintln!("[helper] Relaunching application: {:?}", app_path);

    Command::new("open")
        .arg(app_path)
        .spawn()
        .map_err(|e| format!("Failed to relaunch: {}", e))?;

    eprintln!("[helper] Application relaunch command sent");
    Ok(())
}

/// Waits for the parent process to exit.
pub fn wait_for_parent(pid: u32) {
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
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }

    eprintln!("[helper] Warning: Parent process may still be running");
}
