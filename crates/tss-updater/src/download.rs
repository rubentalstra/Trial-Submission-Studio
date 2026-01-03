//! Download functionality with progress reporting.
//!
//! Handles downloading release assets with progress callbacks for UI updates.

use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use tracing::{debug, info};

use crate::error::{Result, UpdateError};
use crate::release::UpdateInfo;

/// Progress information during download.
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Bytes downloaded so far.
    pub downloaded: u64,
    /// Total bytes to download.
    pub total: u64,
    /// Current download speed in bytes per second.
    pub speed_bps: u64,
    /// Estimated time remaining in seconds.
    pub eta_secs: Option<u64>,
}

impl DownloadProgress {
    /// Get the progress as a fraction (0.0 to 1.0).
    #[must_use]
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.downloaded as f32 / self.total as f32
        }
    }

    /// Get the progress as a percentage (0 to 100).
    #[must_use]
    pub fn percentage(&self) -> u8 {
        (self.fraction() * 100.0) as u8
    }

    /// Get human-readable downloaded size.
    #[must_use]
    pub fn human_downloaded(&self) -> String {
        human_bytes(self.downloaded)
    }

    /// Get human-readable total size.
    #[must_use]
    pub fn human_total(&self) -> String {
        human_bytes(self.total)
    }

    /// Get human-readable download speed.
    #[must_use]
    pub fn human_speed(&self) -> String {
        format!("{}/s", human_bytes(self.speed_bps))
    }

    /// Get human-readable ETA.
    #[must_use]
    pub fn human_eta(&self) -> Option<String> {
        self.eta_secs.map(|secs| {
            if secs < 60 {
                format!("{secs}s")
            } else if secs < 3600 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
            }
        })
    }
}

/// Convert bytes to human-readable format.
fn human_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Downloader for release assets.
pub struct Downloader {
    /// HTTP client.
    client: Client,
    /// Cancellation flag.
    cancelled: Arc<AtomicBool>,
}

impl Downloader {
    /// Create a new downloader.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minute timeout for large downloads
            .build()
            .map_err(UpdateError::Network)?;

        Ok(Self {
            client,
            cancelled: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Get a cancellation handle that can be used to cancel the download.
    #[must_use]
    pub fn cancellation_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancelled)
    }

    /// Cancel any ongoing download.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Reset the cancellation flag for a new download.
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }

    /// Download an update to a temporary location.
    ///
    /// Returns the path to the downloaded file.
    pub fn download<F>(
        &self,
        update_info: &UpdateInfo,
        progress_callback: F,
    ) -> Result<PathBuf>
    where
        F: Fn(DownloadProgress),
    {
        self.reset();

        let url = update_info.download_url();
        let total_size = update_info.download_size();

        info!("Downloading update from: {}", url);
        debug!("Expected size: {} bytes", total_size);

        // Create temp directory for download
        let temp_dir = std::env::temp_dir().join("tss-updates");
        std::fs::create_dir_all(&temp_dir).map_err(UpdateError::Io)?;

        let file_name = &update_info.asset.name;
        let dest_path = temp_dir.join(file_name);

        // Download the file
        self.download_to_file(url, &dest_path, total_size, progress_callback)?;

        info!("Download complete: {}", dest_path.display());
        Ok(dest_path)
    }

    /// Download a URL to a file with progress reporting.
    fn download_to_file<F>(
        &self,
        url: &str,
        dest: &Path,
        total_size: u64,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(DownloadProgress),
    {
        let mut response = self
            .client
            .get(url)
            .header(USER_AGENT, format!("Trial-Submission-Studio/{}", env!("CARGO_PKG_VERSION")))
            .send()
            .map_err(UpdateError::Network)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().unwrap_or_else(|_| "Download failed".to_string());
            return Err(UpdateError::GitHubApi { status, message });
        }

        // Get content length if available
        let total = response
            .content_length()
            .unwrap_or(total_size);

        let file = File::create(dest).map_err(UpdateError::Io)?;
        let mut writer = BufWriter::new(file);

        let mut downloaded: u64 = 0;
        let start_time = Instant::now();
        let mut last_update = Instant::now();

        let mut buffer = [0u8; 8192];

        loop {
            // Check for cancellation
            if self.cancelled.load(Ordering::SeqCst) {
                // Clean up partial download
                drop(writer);
                let _ = std::fs::remove_file(dest);
                return Err(UpdateError::Cancelled);
            }

            // Read chunk using the Read trait
            let bytes_read = response.read(&mut buffer).map_err(UpdateError::Io)?;
            if bytes_read == 0 {
                break;
            }

            writer
                .write_all(&buffer[..bytes_read])
                .map_err(UpdateError::Io)?;
            downloaded += bytes_read as u64;

            // Update progress at most every 100ms
            if last_update.elapsed() >= Duration::from_millis(100) {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed_bps = if elapsed > 0.0 {
                    (downloaded as f64 / elapsed) as u64
                } else {
                    0
                };

                let remaining = total.saturating_sub(downloaded);
                let eta_secs = if speed_bps > 0 {
                    Some(remaining / speed_bps)
                } else {
                    None
                };

                progress_callback(DownloadProgress {
                    downloaded,
                    total,
                    speed_bps,
                    eta_secs,
                });

                last_update = Instant::now();
            }
        }

        writer.flush().map_err(UpdateError::Io)?;

        // Final progress update
        progress_callback(DownloadProgress {
            downloaded,
            total,
            speed_bps: 0,
            eta_secs: Some(0),
        });

        Ok(())
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_bytes() {
        assert_eq!(human_bytes(500), "500 B");
        assert_eq!(human_bytes(1536), "1.50 KB");
        assert_eq!(human_bytes(2_621_440), "2.50 MB");
        assert_eq!(human_bytes(1_610_612_736), "1.50 GB");
    }

    #[test]
    fn test_progress_fraction() {
        let progress = DownloadProgress {
            downloaded: 50,
            total: 100,
            speed_bps: 1000,
            eta_secs: Some(50),
        };
        assert!((progress.fraction() - 0.5).abs() < 0.001);
        assert_eq!(progress.percentage(), 50);
    }

    #[test]
    fn test_human_eta() {
        let progress = |secs| DownloadProgress {
            downloaded: 0,
            total: 100,
            speed_bps: 0,
            eta_secs: Some(secs),
        };

        assert_eq!(progress(30).human_eta(), Some("30s".to_string()));
        assert_eq!(progress(90).human_eta(), Some("1m 30s".to_string()));
        assert_eq!(progress(3700).human_eta(), Some("1h 1m".to_string()));
    }
}
