//! Download functionality with progress reporting.
//!
//! This module provides functions to download release assets from GitHub
//! with progress callbacks for UI updates.

use crate::error::{Result, UpdateError};
use futures_util::StreamExt;
use reqwest::header::{HeaderValue, USER_AGENT};

/// User agent string for download requests.
const USER_AGENT_VALUE: &str = concat!(
    "trial-submission-studio/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/rubentalstra/Trial-Submission-Studio)"
);

/// Progress information during a download operation.
#[derive(Debug, Clone, Copy)]
pub struct DownloadProgress {
    /// Number of bytes downloaded so far.
    pub downloaded: u64,
    /// Total number of bytes to download.
    pub total: u64,
    /// Download progress as a fraction (0.0 to 1.0).
    pub fraction: f32,
}

impl DownloadProgress {
    /// Creates a new progress instance.
    #[must_use]
    pub fn new(downloaded: u64, total: u64) -> Self {
        let fraction = if total > 0 {
            (downloaded as f64 / total as f64) as f32
        } else {
            0.0
        };

        Self {
            downloaded,
            total,
            fraction,
        }
    }

    /// Returns the download progress as a percentage (0-100).
    #[must_use]
    pub fn percentage(&self) -> u8 {
        (self.fraction * 100.0).min(100.0) as u8
    }

    /// Formats the downloaded amount in human-readable form.
    #[must_use]
    pub fn downloaded_display(&self) -> String {
        format_bytes(self.downloaded)
    }

    /// Formats the total size in human-readable form.
    #[must_use]
    pub fn total_display(&self) -> String {
        format_bytes(self.total)
    }
}

/// Formats a byte count in human-readable form.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Downloads a file from the given URL with progress reporting.
///
/// # Arguments
/// * `url` - The URL to download from
/// * `expected_size` - Expected file size (used for progress calculation)
/// * `on_progress` - Callback function called with progress updates
///
/// # Returns
/// The downloaded file contents as a byte vector.
pub async fn download_with_progress<F>(
    url: &str,
    expected_size: u64,
    on_progress: F,
) -> Result<Vec<u8>>
where
    F: Fn(DownloadProgress) + Send + 'static,
{
    tracing::info!("Starting download from {}", url);

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE))
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        return Err(UpdateError::Network(format!(
            "Download failed with status {}",
            status
        )));
    }

    // Get content length from headers, fall back to expected size
    let total_size = response.content_length().unwrap_or(expected_size);

    // Pre-allocate buffer
    let mut data = Vec::with_capacity(total_size as usize);
    let mut downloaded: u64 = 0;

    // Stream the response body
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| UpdateError::Network(e.to_string()))?;

        data.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;

        // Report progress
        let progress = DownloadProgress::new(downloaded, total_size);
        on_progress(progress);

        tracing::trace!(
            "Downloaded {} / {} ({:.1}%)",
            progress.downloaded_display(),
            progress.total_display(),
            progress.fraction * 100.0
        );
    }

    tracing::info!(
        "Download complete: {} bytes",
        format_bytes(data.len() as u64)
    );

    Ok(data)
}

/// Downloads a file without progress reporting.
///
/// This is a convenience function for cases where progress updates are not needed.
pub async fn download(url: &str) -> Result<Vec<u8>> {
    download_with_progress(url, 0, |_| {}).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_progress_new() {
        let progress = DownloadProgress::new(500, 1000);
        assert_eq!(progress.downloaded, 500);
        assert_eq!(progress.total, 1000);
        assert!((progress.fraction - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_download_progress_percentage() {
        let progress = DownloadProgress::new(750, 1000);
        assert_eq!(progress.percentage(), 75);

        let progress = DownloadProgress::new(1000, 1000);
        assert_eq!(progress.percentage(), 100);

        let progress = DownloadProgress::new(0, 1000);
        assert_eq!(progress.percentage(), 0);
    }

    #[test]
    fn test_download_progress_zero_total() {
        let progress = DownloadProgress::new(100, 0);
        assert_eq!(progress.fraction, 0.0);
        assert_eq!(progress.percentage(), 0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(52_428_800), "50.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_progress_display() {
        let progress = DownloadProgress::new(52_428_800, 104_857_600);
        assert_eq!(progress.downloaded_display(), "50.0 MB");
        assert_eq!(progress.total_display(), "100.0 MB");
    }
}
