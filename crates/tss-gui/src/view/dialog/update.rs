//! Update dialog view.
//!
//! Modern update dialog with:
//! - Immediate checking when opened (no idle state)
//! - Enhanced progress display with speed and ETA
//! - Collapsible changelog
//! - Two-step confirmation (download + install)
//! - Inline error handling with suggestions

use std::sync::Arc;
use std::time::Instant;

use iced::widget::{
    Space, button, center, column, container, markdown, progress_bar, row, scrollable, text,
};
use iced::window;
use iced::{Alignment, Element, Length, Theme};
use iced_fonts::lucide;

use crate::message::{DialogMessage, Message, UpdateMessage};
use crate::theme::{
    BORDER_RADIUS_SM, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, button_ghost, button_primary,
    button_secondary, colors,
};

/// Current application version.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// =============================================================================
// STATE TYPES
// =============================================================================

/// Download statistics for enhanced progress display.
#[derive(Debug, Clone, Default)]
pub struct DownloadStats {
    /// Bytes downloaded so far.
    pub downloaded: u64,
    /// Total bytes to download.
    pub total: u64,
    /// Current download speed in bytes per second.
    pub speed: u64,
    /// When the download started.
    pub started_at: Option<Instant>,
}

impl DownloadStats {
    /// Create new download stats.
    pub fn new(total: u64) -> Self {
        Self {
            downloaded: 0,
            total,
            speed: 0,
            started_at: Some(Instant::now()),
        }
    }

    /// Update from progress data.
    pub fn update(&mut self, downloaded: u64, speed: u64) {
        self.downloaded = downloaded;
        self.speed = speed;
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }
    }

    /// Progress fraction (0.0 to 1.0).
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.downloaded as f32 / self.total as f32).min(1.0)
        }
    }

    /// Percentage (0 to 100).
    pub fn percentage(&self) -> u32 {
        (self.fraction() * 100.0) as u32
    }

    /// Estimated time remaining in seconds.
    pub fn eta_seconds(&self) -> Option<u64> {
        if self.speed == 0 || self.downloaded >= self.total {
            return None;
        }
        let remaining = self.total.saturating_sub(self.downloaded);
        Some(remaining / self.speed)
    }

    /// Human-readable ETA string.
    pub fn eta_display(&self) -> String {
        match self.eta_seconds() {
            Some(secs) if secs >= 60 => {
                let mins = secs / 60;
                let secs = secs % 60;
                format!("{}m {}s remaining", mins, secs)
            }
            Some(secs) => format!("{}s remaining", secs),
            None => String::new(),
        }
    }

    /// Human-readable speed string.
    pub fn speed_display(&self) -> String {
        tss_updater::format_speed(self.speed)
    }

    /// Human-readable size progress string.
    pub fn size_display(&self) -> String {
        let downloaded_mb = self.downloaded as f64 / 1_048_576.0;
        let total_mb = self.total as f64 / 1_048_576.0;
        format!("{:.1} / {:.1} MB", downloaded_mb, total_mb)
    }
}

/// Error information with user-friendly details and suggestions.
#[derive(Debug, Clone)]
pub struct UpdateErrorInfo {
    /// User-friendly error message.
    pub message: String,
    /// Suggested action to resolve the error.
    pub suggestion: Option<String>,
    /// Whether the error can be retried.
    pub can_retry: bool,
    /// URL for manual download (if applicable).
    pub manual_download_url: Option<String>,
}

impl UpdateErrorInfo {
    /// Create error info from a string message.
    pub fn from_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            suggestion: Some("Please try again.".to_string()),
            can_retry: true,
            manual_download_url: None,
        }
    }
}

impl From<tss_updater::UpdateError> for UpdateErrorInfo {
    fn from(err: tss_updater::UpdateError) -> Self {
        let suggestion = match err.suggested_action() {
            tss_updater::SuggestedAction::None => None,
            action => Some(action.description().to_string()),
        };

        let manual_download_url = match &err {
            tss_updater::UpdateError::NoAssetFound(_)
            | tss_updater::UpdateError::NoCompatibleAsset => Some(format!(
                "https://github.com/{}/{}/releases/latest",
                tss_updater::REPO_OWNER,
                tss_updater::REPO_NAME
            )),
            _ => None,
        };

        Self {
            message: err.user_message().to_string(),
            suggestion,
            can_retry: err.is_retryable(),
            manual_download_url,
        }
    }
}

/// Context for retry operations.
#[derive(Debug, Clone)]
pub enum RetryContext {
    /// Retry the update check.
    Check,
    /// Retry downloading.
    Download {
        /// Update info for retry.
        info: tss_updater::UpdateInfo,
    },
    /// Retry installation.
    Install {
        /// Update info.
        info: tss_updater::UpdateInfo,
        /// Downloaded data (Arc for cheap cloning of large binaries).
        data: Arc<Vec<u8>>,
    },
}

/// Update check and installation state.
///
/// Clean 8-state design with no idle state - dialog opens directly into checking.
#[derive(Debug, Clone, Default)]
pub enum UpdateState {
    /// Checking for updates (initial state when dialog opens).
    #[default]
    Checking,

    /// Update available with info and collapsible changelog.
    Available {
        /// Update information from tss_updater.
        info: tss_updater::UpdateInfo,
        /// Pre-parsed markdown items for rendering.
        changelog_items: Vec<markdown::Item>,
        /// Whether changelog is expanded.
        changelog_expanded: bool,
    },

    /// No update available - already on latest version.
    UpToDate,

    /// Downloading update with enhanced progress.
    Downloading {
        /// Update information.
        info: tss_updater::UpdateInfo,
        /// Download statistics.
        stats: DownloadStats,
    },

    /// Verifying SHA256 hash (brief state).
    Verifying {
        /// Update information.
        info: tss_updater::UpdateInfo,
    },

    /// Download complete, ready for second confirmation.
    ReadyToInstall {
        /// Update information.
        info: tss_updater::UpdateInfo,
        /// Downloaded binary data (Arc for cheap cloning of large binaries).
        data: Arc<Vec<u8>>,
        /// Whether verification passed.
        verified: bool,
    },

    /// Installing update.
    Installing {
        /// Update information.
        info: tss_updater::UpdateInfo,
    },

    /// Installation complete, restart required.
    Complete {
        /// Version that was installed.
        version: String,
    },

    /// Error occurred with recovery options.
    Error {
        /// Error details.
        error: UpdateErrorInfo,
        /// Context for retry (if applicable).
        retry_context: Option<RetryContext>,
    },
}

// =============================================================================
// MAIN VIEW FUNCTION
// =============================================================================

/// Render the update dialog content for a standalone window.
pub fn view_update_dialog_content<'a>(
    state: &'a UpdateState,
    window_id: window::Id,
) -> Element<'a, Message> {
    let c = colors();
    let bg_elevated = c.background_elevated;

    let content: Element<'a, Message> = match state {
        UpdateState::Checking => view_checking(window_id),
        UpdateState::Available {
            info,
            changelog_items,
            changelog_expanded,
        } => view_available(info, changelog_items, *changelog_expanded, window_id),
        UpdateState::UpToDate => view_up_to_date(window_id),
        UpdateState::Downloading { info, stats } => view_downloading(info, stats, window_id),
        UpdateState::Verifying { info } => view_ready_to_install(info, None, window_id),
        UpdateState::ReadyToInstall {
            info,
            verified,
            data: _,
        } => view_ready_to_install(info, Some(*verified), window_id),
        UpdateState::Installing { info } => view_installing(info),
        UpdateState::Complete { version } => view_complete(version),
        UpdateState::Error {
            error,
            retry_context,
        } => view_error(error, retry_context.is_some(), window_id),
    };

    // Dialog container with white background (content fills window directly)
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(SPACING_LG)
        .style(move |_| container::Style {
            background: Some(bg_elevated.into()),
            ..Default::default()
        })
        .into()
}

// =============================================================================
// VIEW FUNCTIONS FOR EACH STATE
// =============================================================================

/// Checking state - initial state with spinner.
fn view_checking(window_id: window::Id) -> Element<'static, Message> {
    let c = colors();

    let spinner = lucide::loader().size(48).color(c.accent_primary);

    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_secondary);

    let content = column![
        Space::new().height(SPACING_LG),
        center(spinner).width(Length::Fill),
        Space::new().height(SPACING_MD),
        text("Checking for Updates...")
            .size(18)
            .color(c.text_primary)
            .center(),
        Space::new().height(SPACING_XS),
        text(format!("Current version: {}", CURRENT_VERSION))
            .size(13)
            .color(c.text_muted)
            .center(),
        Space::new().height(SPACING_LG),
        Space::new().height(SPACING_LG),
        row![Space::new().width(Length::Fill), cancel_btn],
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .width(Length::Fill);

    content.into()
}

/// Available state - update available with collapsible changelog.
fn view_available<'a>(
    info: &'a tss_updater::UpdateInfo,
    changelog_items: &'a [markdown::Item],
    changelog_expanded: bool,
    window_id: window::Id,
) -> Element<'a, Message> {
    let c = colors();

    let header = column![
        text(format!("Version {} Available", info.version_display()))
            .size(20)
            .color(c.text_primary),
        Space::new().height(SPACING_XS),
        text(format!("You have version {}", CURRENT_VERSION))
            .size(13)
            .color(c.text_muted),
    ]
    .align_x(Alignment::Center);

    // Collapsible changelog section
    let changelog_section = view_collapsible_changelog(changelog_items, changelog_expanded);

    // Action buttons
    let size_display = info.asset.size_display();
    let download_btn = button(
        row![
            lucide::download().size(14),
            Space::new().width(SPACING_XS),
            text(format!("Download ({})", size_display)),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Dialog(DialogMessage::Update(
        UpdateMessage::ConfirmDownload,
    )))
    .padding([SPACING_SM, SPACING_MD])
    .style(button_primary);

    let later_btn = button(text("Later").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_secondary);

    let skip_btn = button(text("Skip Version").size(13))
        .on_press(Message::Dialog(DialogMessage::Update(
            UpdateMessage::SkipVersion,
        )))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_ghost);

    let buttons = row![
        skip_btn,
        Space::new().width(Length::Fill),
        later_btn,
        download_btn
    ]
    .spacing(SPACING_SM)
    .align_y(Alignment::Center);

    let content = column![
        Space::new().height(SPACING_MD),
        header,
        Space::new().height(SPACING_MD),
        changelog_section,
        Space::new().height(SPACING_MD),
        buttons,
    ]
    .padding(SPACING_LG)
    .width(Length::Fill);

    content.into()
}

/// Collapsible changelog component.
fn view_collapsible_changelog<'a>(
    changelog_items: &'a [markdown::Item],
    expanded: bool,
) -> Element<'a, Message> {
    let c = colors();
    let text_secondary = c.text_secondary;
    let bg_secondary = c.background_secondary;
    let border_default = c.border_default;

    let chevron = if expanded {
        lucide::chevron_up().size(16).color(text_secondary)
    } else {
        lucide::chevron_down().size(16).color(text_secondary)
    };

    let header_btn = button(
        row![
            text("Release Notes").size(14).color(text_secondary),
            Space::new().width(Length::Fill),
            chevron,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .on_press(Message::Dialog(DialogMessage::Update(
        UpdateMessage::ToggleChangelog,
    )))
    .padding([SPACING_SM, SPACING_MD])
    .width(Length::Fill)
    .style(move |_theme: &Theme, status| {
        let mut style = button_secondary(_theme, status);
        style.border.radius = BORDER_RADIUS_SM.into();
        style
    });

    if expanded {
        // Render changelog using iced's markdown widget
        let markdown_content: Element<'_, Message> = markdown::view(changelog_items, Theme::Light)
            .map(|url| Message::OpenUrl(url.to_string()));

        let changelog_view = scrollable(
            container(markdown_content)
                .padding(SPACING_MD)
                .width(Length::Fill),
        )
        .height(Length::Fixed(220.0))
        .width(Length::Fill);

        let changelog_container = container(changelog_view)
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(bg_secondary.into()),
                border: iced::Border {
                    color: border_default,
                    width: 1.0,
                    radius: BORDER_RADIUS_SM.into(),
                },
                ..Default::default()
            });

        column![header_btn, changelog_container]
            .spacing(0)
            .width(Length::Fill)
            .into()
    } else {
        header_btn.into()
    }
}

/// Up to date state.
fn view_up_to_date(window_id: window::Id) -> Element<'static, Message> {
    let c = colors();

    let icon = lucide::circle_check().size(48).color(c.status_success);

    let close_btn = button(text("Close").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_LG])
        .style(button_secondary);

    let content = column![
        Space::new().height(SPACING_LG),
        center(icon).width(Length::Fill),
        Space::new().height(SPACING_MD),
        text("You're Up to Date")
            .size(20)
            .color(c.text_primary)
            .center(),
        Space::new().height(SPACING_XS),
        text(format!(
            "Version {} is the latest version.",
            CURRENT_VERSION
        ))
        .size(13)
        .color(c.text_muted)
        .center(),
        Space::new().height(SPACING_LG),
        Space::new().height(SPACING_LG),
        row![Space::new().width(Length::Fill), close_btn],
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .width(Length::Fill);

    content.into()
}

/// Downloading state with enhanced progress display.
fn view_downloading<'a>(
    info: &'a tss_updater::UpdateInfo,
    stats: &DownloadStats,
    window_id: window::Id,
) -> Element<'a, Message> {
    let c = colors();
    let accent_primary = c.accent_primary;
    let border_default = c.border_default;

    let percentage = stats.percentage();

    let percentage_text = text(format!("{}%", percentage))
        .size(32)
        .color(accent_primary)
        .center();

    let title = text(format!("Downloading {}", info.version_display()))
        .size(18)
        .color(c.text_primary)
        .center();

    // Progress bar
    let progress = progress_bar(0.0..=1.0, stats.fraction()).style(move |_| progress_bar::Style {
        background: border_default.into(),
        bar: accent_primary.into(),
        border: iced::Border {
            radius: 4.0.into(),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
    });

    // Stats row
    let stats_row = row![
        text(stats.size_display()).size(12).color(c.text_muted),
        Space::new().width(Length::Fill),
        text(stats.speed_display()).size(12).color(c.text_muted),
    ]
    .width(Length::Fill);

    // ETA row
    let eta = stats.eta_display();
    let eta_row = if !eta.is_empty() {
        row![
            Space::new().width(Length::Fill),
            text(eta).size(12).color(c.text_muted),
        ]
        .width(Length::Fill)
    } else {
        row![].width(Length::Fill)
    };

    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_secondary);

    let content = column![
        Space::new().height(SPACING_MD),
        percentage_text,
        Space::new().height(SPACING_SM),
        title,
        Space::new().height(SPACING_MD),
        container(progress).width(Length::Fill),
        Space::new().height(SPACING_XS),
        stats_row,
        eta_row,
        Space::new().height(SPACING_LG),
        row![Space::new().width(Length::Fill), cancel_btn],
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .width(Length::Fill);

    content.into()
}

/// Ready to install state - second confirmation with prominent verification status.
///
/// `verified` is:
/// - `None` = verification in progress (shows spinner)
/// - `Some(true)` = verified successfully
/// - `Some(false)` = unverified (no digest available)
fn view_ready_to_install<'a>(
    info: &'a tss_updater::UpdateInfo,
    verified: Option<bool>,
    window_id: window::Id,
) -> Element<'a, Message> {
    let c = colors();
    let accent_primary = c.accent_primary;
    let status_success = c.status_success;
    let status_success_light = c.status_success_light;
    let status_warning = c.status_warning;
    let status_warning_light = c.status_warning_light;
    let status_info_light = c.status_info_light;
    let text_primary = c.text_primary;
    let text_secondary = c.text_secondary;
    let text_muted = c.text_muted;

    // Large icon based on verification status
    let icon: Element<'a, Message> = match verified {
        None => lucide::shield_check().size(48).color(accent_primary).into(), // Verifying
        Some(true) => lucide::shield_check().size(48).color(status_success).into(), // Verified
        Some(false) => lucide::shield_alert().size(48).color(status_warning).into(), // Unverified
    };

    // Verification status badge with styled container
    let verification_badge: Element<'a, Message> = match verified {
        None => {
            // Verifying in progress
            container(
                row![
                    lucide::loader().size(16).color(accent_primary),
                    Space::new().width(SPACING_XS),
                    text("Verifying Download...").size(14).color(accent_primary),
                ]
                .align_y(Alignment::Center),
            )
            .padding([SPACING_SM, SPACING_MD])
            .style(move |_| container::Style {
                background: Some(status_info_light.into()),
                border: iced::Border {
                    color: accent_primary,
                    width: 1.0,
                    radius: BORDER_RADIUS_SM.into(),
                },
                ..Default::default()
            })
            .into()
        }
        Some(is_verified) => {
            let badge_text = if is_verified {
                "Download Verified"
            } else {
                "Unverified Download"
            };
            let badge_color = if is_verified {
                status_success
            } else {
                status_warning
            };
            let badge_bg = if is_verified {
                status_success_light
            } else {
                status_warning_light
            };

            let badge_icon: Element<'a, Message> = if is_verified {
                lucide::shield_check().size(16).color(badge_color).into()
            } else {
                lucide::shield_alert().size(16).color(badge_color).into()
            };

            container(
                row![
                    badge_icon,
                    Space::new().width(SPACING_XS),
                    text(badge_text).size(14).color(badge_color),
                ]
                .align_y(Alignment::Center),
            )
            .padding([SPACING_SM, SPACING_MD])
            .style(move |_| container::Style {
                background: Some(badge_bg.into()),
                border: iced::Border {
                    color: badge_color,
                    width: 1.0,
                    radius: BORDER_RADIUS_SM.into(),
                },
                ..Default::default()
            })
            .into()
        }
    };

    // Install button - disabled while verifying
    let install_btn_content = row![
        lucide::package().size(14),
        Space::new().width(SPACING_XS),
        text("Install & Restart"),
    ]
    .align_y(Alignment::Center);

    let install_btn = if verified.is_some() {
        button(install_btn_content)
            .on_press(Message::Dialog(DialogMessage::Update(
                UpdateMessage::ConfirmInstall,
            )))
            .padding([SPACING_SM, SPACING_MD])
            .style(button_primary)
    } else {
        // Disabled while verifying
        button(install_btn_content)
            .padding([SPACING_SM, SPACING_MD])
            .style(button_secondary)
    };

    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_secondary);

    let content = column![
        Space::new().height(SPACING_LG),
        center(icon).width(Length::Fill),
        Space::new().height(SPACING_MD),
        text("Ready to Install")
            .size(20)
            .color(text_primary)
            .center(),
        Space::new().height(SPACING_XS),
        text(format!("Version {}", info.version_display()))
            .size(14)
            .color(text_secondary)
            .center(),
        Space::new().height(SPACING_MD),
        center(verification_badge).width(Length::Fill),
        Space::new().height(SPACING_MD),
        text("The application will restart after installation.")
            .size(13)
            .color(text_muted)
            .center(),
        Space::new().height(SPACING_LG),
        row![Space::new().width(Length::Fill), cancel_btn, install_btn].spacing(SPACING_SM),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .width(Length::Fill);

    content.into()
}

/// Installing state - brief indicator.
fn view_installing<'a>(info: &'a tss_updater::UpdateInfo) -> Element<'a, Message> {
    let c = colors();

    let icon = lucide::loader().size(48).color(c.accent_primary);

    let content = column![
        Space::new().height(SPACING_LG),
        center(icon).width(Length::Fill),
        Space::new().height(SPACING_MD),
        text("Installing Update...")
            .size(18)
            .color(c.text_primary)
            .center(),
        Space::new().height(SPACING_XS),
        text(format!("Installing version {}", info.version_display()))
            .size(13)
            .color(c.text_muted)
            .center(),
        Space::new().height(SPACING_XS),
        text("Please wait, do not close the application.")
            .size(12)
            .color(c.text_disabled)
            .center(),
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .width(Length::Fill);

    content.into()
}

/// Complete state - restart required.
fn view_complete(version: &str) -> Element<'static, Message> {
    let c = colors();

    let icon = lucide::circle_check().size(48).color(c.status_success);

    let restart_btn = button(
        row![
            lucide::refresh_cw().size(14),
            Space::new().width(SPACING_XS),
            text("Restart Now"),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Dialog(DialogMessage::Update(
        UpdateMessage::RestartNow,
    )))
    .padding([SPACING_SM, SPACING_LG])
    .style(button_primary);

    let content = column![
        Space::new().height(SPACING_LG),
        center(icon).width(Length::Fill),
        Space::new().height(SPACING_MD),
        text("Update Installed")
            .size(20)
            .color(c.text_primary)
            .center(),
        Space::new().height(SPACING_SM),
        text(format!("Version {} installed successfully.", version))
            .size(13)
            .color(c.text_muted)
            .center(),
        Space::new().height(SPACING_XS),
        text("Restart to start using it.")
            .size(13)
            .color(c.text_muted)
            .center(),
        Space::new().height(SPACING_LG),
        center(restart_btn).width(Length::Fill),
        Space::new().height(SPACING_MD),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .width(Length::Fill);

    content.into()
}

/// Error state with suggestions and retry.
fn view_error(
    error: &UpdateErrorInfo,
    can_retry: bool,
    window_id: window::Id,
) -> Element<'static, Message> {
    let c = colors();
    let status_error = c.status_error;
    let status_warning = c.status_warning;
    let status_warning_light = c.status_warning_light;
    let accent_primary = c.accent_primary;
    let text_primary = c.text_primary;
    let text_secondary = c.text_secondary;

    let icon = lucide::circle_x().size(48).color(status_error);

    let error_message = error.message.clone();
    let mut content_items: Vec<Element<'static, Message>> = vec![
        Space::new().height(SPACING_LG).into(),
        center(icon).width(Length::Fill).into(),
        Space::new().height(SPACING_MD).into(),
        text("Update Failed")
            .size(18)
            .color(text_primary)
            .center()
            .into(),
        Space::new().height(SPACING_SM).into(),
        text(error_message)
            .size(13)
            .color(text_secondary)
            .center()
            .into(),
    ];

    // Suggestion box
    if let Some(suggestion) = &error.suggestion {
        content_items.push(Space::new().height(SPACING_MD).into());
        let suggestion_text = suggestion.clone();
        let suggestion_box = container(
            row![
                lucide::lightbulb().size(14).color(status_warning),
                Space::new().width(SPACING_SM),
                text(suggestion_text).size(12).color(text_secondary),
            ]
            .align_y(Alignment::Center),
        )
        .padding(SPACING_SM)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(status_warning_light.into()),
            border: iced::Border {
                color: status_warning,
                width: 1.0,
                radius: BORDER_RADIUS_SM.into(),
            },
            ..Default::default()
        });
        content_items.push(suggestion_box.into());
    }

    // Manual download link
    if let Some(url) = &error.manual_download_url {
        content_items.push(Space::new().height(SPACING_SM).into());
        let link_btn = button(
            row![
                lucide::external_link().size(12).color(accent_primary),
                Space::new().width(4),
                text("Download manually").size(12).color(accent_primary),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::OpenUrl(url.clone()))
        .padding([4, 8])
        .style(button_ghost);
        content_items.push(center(link_btn).width(Length::Fill).into());
    }

    // Buttons
    content_items.push(Space::new().height(SPACING_LG).into());

    let close_btn = button(text("Close").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_secondary);

    let buttons = if can_retry && error.can_retry {
        let retry_btn = button(text("Retry").size(13))
            .on_press(Message::Dialog(DialogMessage::Update(UpdateMessage::Retry)))
            .padding([SPACING_SM, SPACING_MD])
            .style(button_primary);
        row![Space::new().width(Length::Fill), close_btn, retry_btn].spacing(SPACING_SM)
    } else {
        row![Space::new().width(Length::Fill), close_btn]
    };

    content_items.push(buttons.into());

    let content = column(content_items)
        .align_x(Alignment::Center)
        .padding(SPACING_LG)
        .width(Length::Fill);

    content.into()
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Parse changelog into markdown items for rendering.
pub fn parse_changelog(changelog: &str) -> Vec<markdown::Item> {
    markdown::parse(changelog).collect()
}
