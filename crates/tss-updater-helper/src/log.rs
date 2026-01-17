//! File-based logging for the update helper.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Local;

/// Global log file handle.
static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

/// Path to the current log file.
static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Gets the log directory path.
pub fn log_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("Library/Logs/Trial Submission Studio"))
}

/// Gets the path to the current log file, creating it if necessary.
pub fn init_logging() -> Result<PathBuf, String> {
    let log_dir = log_dir().ok_or_else(|| "Could not determine log directory".to_string())?;

    // Create log directory if it doesn't exist
    fs::create_dir_all(&log_dir).map_err(|e| format!("Failed to create log directory: {}", e))?;

    // Create log file with timestamp
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let log_path = log_dir.join(format!("update-helper-{}.log", timestamp));

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .map_err(|e| format!("Failed to create log file: {}", e))?;

    // Store the file handle and path
    *LOG_FILE.lock().unwrap() = Some(file);
    *LOG_PATH.lock().unwrap() = Some(log_path.clone());

    // Write initial log header
    log(&format!(
        "=== Trial Submission Studio Update Helper ===\nStarted: {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    Ok(log_path)
}

/// Logs a message to the log file.
pub fn log(message: &str) {
    let timestamp = Local::now().format("%H:%M:%S%.3f");
    let log_line = format!("[{}] {}\n", timestamp, message);

    // Write to stderr for console output
    eprint!("{}", log_line);

    // Write to log file if available
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *guard {
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }
    }
}

/// Gets the path to the current log file.
pub fn get_log_path() -> Option<PathBuf> {
    LOG_PATH.lock().ok().and_then(|g| g.clone())
}

/// Logs an error and returns a formatted error string.
pub fn log_error(context: &str, error: &str) -> String {
    let msg = format!("ERROR: {} - {}", context, error);
    log(&msg);
    msg
}
