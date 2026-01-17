//! Download update with streaming progress.

use async_stream::stream;
use futures_util::{Stream, StreamExt};
use reqwest::header::{HeaderValue, USER_AGENT};

use crate::error::{Result, UpdateError};

/// User agent string for download requests.
const USER_AGENT_VALUE: &str = concat!(
    "trial-submission-studio/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/rubentalstra/Trial-Submission-Studio)"
);

/// Progress update interval in milliseconds.
const PROGRESS_UPDATE_INTERVAL_MS: u64 = 100;

/// Download progress event.
///
/// This struct is yielded by `download_stream()` to report progress.
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Bytes downloaded so far.
    pub downloaded: u64,
    /// Total bytes to download.
    pub total: u64,
    /// Current download speed in bytes per second (smoothed average).
    pub speed: u64,
}

impl DownloadProgress {
    /// Returns the progress as a fraction (0.0 to 1.0).
    #[must_use]
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.downloaded as f64 / self.total as f64) as f32
    }

    /// Returns the progress as a percentage (0 to 100).
    #[must_use]
    pub fn percentage(&self) -> u8 {
        (self.fraction() * 100.0).min(100.0) as u8
    }
}

/// Download progress tracker with smoothed speed calculation.
struct ProgressTracker {
    downloaded: u64,
    total: u64,
    samples: Vec<(u64, u64)>, // (timestamp_ms, bytes)
    last_emit: u64,
    max_samples: usize,
}

impl ProgressTracker {
    fn new(total: u64) -> Self {
        Self {
            downloaded: 0,
            total,
            samples: Vec::with_capacity(20),
            last_emit: 0,
            max_samples: 20, // ~2 seconds at 100ms updates
        }
    }

    fn update(&mut self, downloaded: u64) {
        self.downloaded = downloaded;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        self.samples.push((now, downloaded));

        if self.samples.len() > self.max_samples {
            self.samples.remove(0);
        }
    }

    fn speed(&self) -> u64 {
        if self.samples.len() < 2 {
            return 0;
        }

        let first = &self.samples[0];
        let last = self.samples.last().unwrap();

        let time_diff_ms = last.0.saturating_sub(first.0);
        let bytes_diff = last.1.saturating_sub(first.1);

        if time_diff_ms == 0 {
            return 0;
        }

        (bytes_diff as f64 * 1000.0 / time_diff_ms as f64) as u64
    }

    fn should_emit(&mut self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        if now.saturating_sub(self.last_emit) >= PROGRESS_UPDATE_INTERVAL_MS {
            self.last_emit = now;
            true
        } else {
            false
        }
    }

    fn to_progress(&self) -> DownloadProgress {
        DownloadProgress {
            downloaded: self.downloaded,
            total: self.total,
            speed: self.speed(),
        }
    }
}

/// Download result containing the data and final progress.
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// The downloaded data.
    pub data: Vec<u8>,
    /// Final progress information.
    pub progress: DownloadProgress,
}

/// Downloads a file and returns a stream of progress events.
///
/// The stream yields `DownloadProgress` events approximately every 100ms.
/// When the download completes, the stream ends and you should call
/// `download_complete()` to get the actual data.
///
/// # Usage with Iced
///
/// This is designed to work with `Task::sip()` for streaming progress:
///
/// ```ignore
/// Task::sip(
///     download_stream(&url, total),
///     |progress| Message::DownloadProgress(progress),
///     |result| Message::DownloadComplete(result),
/// )
/// ```
pub fn download_stream(
    url: &str,
    expected_size: u64,
) -> impl Stream<Item = Result<DownloadProgress>> {
    let url = url.to_string();

    stream! {
        tracing::info!("Starting streaming download from {}", url);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE))
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                yield Err(e.into());
                return;
            }
        };

        let status = response.status();
        if !status.is_success() {
            yield Err(UpdateError::Network(format!(
                "Download failed with status {}",
                status
            )));
            return;
        }

        // Get content length from headers, fall back to expected size
        let total_size = response.content_length().unwrap_or(expected_size);
        let mut tracker = ProgressTracker::new(total_size);

        // Stream the response body
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(e) => {
                    yield Err(UpdateError::Network(e.to_string()));
                    return;
                }
            };

            tracker.update(tracker.downloaded + chunk.len() as u64);

            // Emit progress event at throttled interval
            if tracker.should_emit() {
                yield Ok(tracker.to_progress());
            }
        }

        // Emit final progress event
        yield Ok(tracker.to_progress());
    }
}

/// Downloads a file with progress events, returning both progress stream and final data.
///
/// This version accumulates the data and returns it in the final result.
/// Use this when you need the downloaded data after the stream completes.
///
/// Note: Takes owned String to allow the stream to be 'static.
pub fn download_with_data(
    url: String,
    expected_size: u64,
) -> impl Stream<Item = Result<DownloadStreamItem>> + Send + 'static {
    stream! {
        tracing::info!("Starting download with data from {}", url);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE))
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                yield Err(e.into());
                return;
            }
        };

        let status = response.status();
        if !status.is_success() {
            yield Err(UpdateError::Network(format!(
                "Download failed with status {}",
                status
            )));
            return;
        }

        // Get content length from headers, fall back to expected size
        let total_size = response.content_length().unwrap_or(expected_size);
        let mut data = Vec::with_capacity(total_size as usize);
        let mut tracker = ProgressTracker::new(total_size);

        // Stream the response body
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(e) => {
                    yield Err(UpdateError::Network(e.to_string()));
                    return;
                }
            };

            data.extend_from_slice(&chunk);
            tracker.update(data.len() as u64);

            // Emit progress event at throttled interval
            if tracker.should_emit() {
                yield Ok(DownloadStreamItem::Progress(tracker.to_progress()));
            }
        }

        tracing::info!(
            "Download complete: {} bytes",
            format_bytes(data.len() as u64)
        );

        // Yield the complete data
        yield Ok(DownloadStreamItem::Complete(DownloadResult {
            data,
            progress: tracker.to_progress(),
        }));
    }
}

/// Item yielded by `download_with_data()` stream.
#[derive(Debug, Clone)]
pub enum DownloadStreamItem {
    /// Progress update during download.
    Progress(DownloadProgress),
    /// Download complete with data.
    Complete(DownloadResult),
}

/// Downloads a file without progress reporting.
///
/// This is a simpler version for cases where progress updates are not needed.
pub async fn download_simple(url: &str) -> Result<Vec<u8>> {
    tracing::info!("Starting simple download from {}", url);

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

    let data = response
        .bytes()
        .await
        .map_err(|e| UpdateError::Network(e.to_string()))?;

    Ok(data.to_vec())
}

/// Format bytes as a human-readable string.
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
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

/// Format speed as a human-readable string.
#[must_use]
pub fn format_speed(bytes_per_sec: u64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent() {
        assert!(USER_AGENT_VALUE.starts_with("trial-submission-studio/"));
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
    fn test_format_speed() {
        assert_eq!(format_speed(1024), "1.0 KB/s");
        assert_eq!(format_speed(1024 * 1024), "1.0 MB/s");
    }

    #[test]
    fn test_download_progress_fraction() {
        let progress = DownloadProgress {
            downloaded: 250,
            total: 1000,
            speed: 0,
        };

        assert!((progress.fraction() - 0.25).abs() < 0.001);
        assert_eq!(progress.percentage(), 25);
    }
}
