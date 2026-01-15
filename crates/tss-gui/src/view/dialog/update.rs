//! Update dialog view.
//!
//! Check for and install application updates with SHA256 verification.
//!
//! This module provides multi-window dialog views for the update process:
//! - Check for updates
//! - Download with progress
//! - SHA256 verification
//! - Installation and restart

use iced::widget::{
    Space, button, column, container, markdown, progress_bar, row, scrollable, text,
};
use iced::window;
use iced::{Alignment, Element, Length, Theme};
use iced_fonts::lucide;

use crate::message::{DialogMessage, Message, UpdateMessage};
use crate::theme::{
    GRAY_100, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900, PRIMARY_500, SPACING_LG,
    SPACING_MD, SPACING_SM, SPACING_XS, SUCCESS, WARNING, button_primary,
};

/// Current application version.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Update check and installation state.
#[derive(Debug, Clone, Default)]
pub enum UpdateState {
    /// Initial state, no check performed.
    #[default]
    Idle,

    /// Checking for updates.
    Checking,

    /// Update available with parsed changelog.
    Available {
        /// Update information from tss_updater.
        info: tss_updater::UpdateInfo,
        /// Pre-parsed markdown items for rendering.
        changelog_items: Vec<markdown::Item>,
    },

    /// No update available.
    UpToDate,

    /// Downloading update with progress.
    Downloading {
        /// Update information.
        info: tss_updater::UpdateInfo,
        /// Progress fraction (0.0 to 1.0).
        progress: f32,
        /// Bytes downloaded so far.
        downloaded_bytes: u64,
        /// Total bytes to download.
        total_bytes: u64,
    },

    /// Verifying SHA256 hash.
    Verifying {
        /// Update information.
        info: tss_updater::UpdateInfo,
    },

    /// SHA256 verification failed.
    VerificationFailed {
        /// Update information.
        info: tss_updater::UpdateInfo,
        /// Expected hash.
        expected: String,
        /// Actual hash.
        actual: String,
    },

    /// Download complete, ready to install.
    ReadyToInstall {
        /// Update information.
        info: tss_updater::UpdateInfo,
        /// Downloaded binary data.
        data: Vec<u8>,
        /// Whether verification passed.
        verified: bool,
    },

    /// Installing update.
    Installing,

    /// Installation complete, restart required.
    InstallComplete {
        /// Version that was installed.
        version: String,
    },

    /// Error occurred.
    Error(String),
}

/// Render the Update dialog content for a standalone window (multi-window mode).
///
/// This is the content that appears in a separate dialog window.
pub fn view_update_dialog_content(
    state: &UpdateState,
    window_id: window::Id,
) -> Element<'_, Message> {
    let content = match state {
        UpdateState::Idle => view_idle_state(window_id),
        UpdateState::Checking => view_checking_state(),
        UpdateState::Available {
            info,
            changelog_items,
        } => view_available_state(info, changelog_items, window_id),
        UpdateState::UpToDate => view_up_to_date_state(window_id),
        UpdateState::Downloading {
            info,
            progress,
            downloaded_bytes,
            total_bytes,
        } => view_downloading_state(info, *progress, *downloaded_bytes, *total_bytes, window_id),
        UpdateState::Verifying { info } => view_verifying_state(info),
        UpdateState::VerificationFailed {
            info,
            expected,
            actual,
        } => view_verification_failed_state(info, expected, actual, window_id),
        UpdateState::ReadyToInstall {
            info,
            verified,
            data: _,
        } => view_ready_to_install_state(info, *verified, window_id),
        UpdateState::Installing => view_installing_state(),
        UpdateState::InstallComplete { version } => view_install_complete_state(version, window_id),
        UpdateState::Error(msg) => view_error_state(msg, window_id),
    };

    // Wrap in a styled container for the window
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| container::Style {
            background: Some(GRAY_100.into()),
            ..Default::default()
        })
        .into()
}

/// Idle state - prompt to check for updates.
fn view_idle_state(window_id: window::Id) -> Element<'static, Message> {
    let icon = lucide::refresh_cw().size(32).color(PRIMARY_500);

    let current_version = text(format!("Current version: {}", CURRENT_VERSION))
        .size(12)
        .color(GRAY_500);

    let check_btn = button(
        row![
            lucide::search().size(14),
            Space::new().width(SPACING_XS),
            text("Check for Updates"),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Dialog(DialogMessage::Update(
        UpdateMessage::CheckForUpdates,
    )))
    .padding([SPACING_SM, SPACING_MD])
    .style(button_primary);

    let close_btn = button(text("Close").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD]);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Check for Updates").size(18).color(GRAY_900),
        Space::new().height(SPACING_XS),
        current_version,
        Space::new().height(SPACING_LG),
        check_btn,
        Space::new().height(SPACING_SM),
        close_btn,
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Checking state - spinner/loading.
fn view_checking_state() -> Element<'static, Message> {
    let icon = lucide::loader().size(32).color(PRIMARY_500);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Checking for Updates...").size(16).color(GRAY_800),
        Space::new().height(SPACING_XS),
        text("Please wait").size(13).color(GRAY_500),
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Update available state with changelog.
fn view_available_state<'a>(
    info: &'a tss_updater::UpdateInfo,
    changelog_items: &'a [markdown::Item],
    window_id: window::Id,
) -> Element<'a, Message> {
    let icon = lucide::download().size(32).color(SUCCESS);

    let header = text(format!("Version {} Available", info.version_display()))
        .size(18)
        .color(GRAY_900);

    let current = text(format!("Current: {}", CURRENT_VERSION))
        .size(12)
        .color(GRAY_500);

    // Changelog section with markdown rendering
    let changelog_header = text("Release Notes").size(14).color(GRAY_700);

    // Render changelog using iced's markdown widget in scrollable container
    // The markdown widget returns Element<String> (for URL clicks), we map to our Message type
    let markdown_content: Element<'_, Message> =
        markdown::view(changelog_items, Theme::Light).map(|url| {
            // Open URLs in browser when clicked
            let _ = open::that(&url);
            Message::Noop
        });

    let changelog_view = scrollable(container(markdown_content).padding(SPACING_SM))
        .height(Length::Fixed(200.0))
        .width(Length::Fill);

    let changelog_container =
        container(changelog_view)
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Color::WHITE.into()),
                border: iced::Border {
                    color: GRAY_500,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

    // Show file size on download button
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

    let skip_btn = button(text("Skip Version").size(13))
        .on_press(Message::Dialog(DialogMessage::Update(
            UpdateMessage::SkipVersion(info.version.clone()),
        )))
        .padding([SPACING_SM, SPACING_MD]);

    let later_btn = button(text("Remind Later").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD]);

    column![
        Space::new().height(SPACING_MD),
        icon,
        Space::new().height(SPACING_MD),
        header,
        Space::new().height(SPACING_XS),
        current,
        Space::new().height(SPACING_MD),
        changelog_header,
        Space::new().height(SPACING_SM),
        changelog_container,
        Space::new().height(SPACING_MD),
        row![download_btn, skip_btn, later_btn].spacing(SPACING_SM),
        Space::new().height(SPACING_MD),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .width(Length::Fill)
    .into()
}

/// Up to date state.
fn view_up_to_date_state(window_id: window::Id) -> Element<'static, Message> {
    let icon = lucide::circle_check().size(32).color(SUCCESS);

    let close_btn = button(text("Close").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_LG]);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("You're Up to Date!").size(18).color(GRAY_900),
        Space::new().height(SPACING_XS),
        text(format!("Version {} is the latest", CURRENT_VERSION))
            .size(13)
            .color(GRAY_500),
        Space::new().height(SPACING_LG),
        close_btn,
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Downloading state with progress.
fn view_downloading_state(
    info: &tss_updater::UpdateInfo,
    progress: f32,
    downloaded_bytes: u64,
    total_bytes: u64,
    window_id: window::Id,
) -> Element<'_, Message> {
    let icon = lucide::download().size(32).color(PRIMARY_500);

    // Format bytes for display
    let downloaded_mb = downloaded_bytes as f64 / 1_048_576.0;
    let total_mb = total_bytes as f64 / 1_048_576.0;

    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD]);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text(format!("Downloading version {}...", info.version_display()))
            .size(16)
            .color(GRAY_800),
        Space::new().height(SPACING_MD),
        container(progress_bar(0.0..=1.0, progress)).width(300),
        Space::new().height(SPACING_XS),
        text(format!(
            "{:.1} MB / {:.1} MB ({:.0}%)",
            downloaded_mb,
            total_mb,
            progress * 100.0
        ))
        .size(12)
        .color(GRAY_500),
        Space::new().height(SPACING_LG),
        cancel_btn,
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Verifying state.
fn view_verifying_state(info: &tss_updater::UpdateInfo) -> Element<'_, Message> {
    let icon = lucide::shield_check().size(32).color(PRIMARY_500);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Verifying Download...").size(16).color(GRAY_800),
        Space::new().height(SPACING_XS),
        text(format!(
            "Checking SHA256 hash for version {}",
            info.version_display()
        ))
        .size(13)
        .color(GRAY_500),
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Verification failed state.
fn view_verification_failed_state<'a>(
    _info: &'a tss_updater::UpdateInfo,
    expected: &'a str,
    actual: &'a str,
    window_id: window::Id,
) -> Element<'a, Message> {
    let icon = lucide::shield_x().size(32).color(WARNING);

    let retry_btn = button(text("Retry Download").size(13))
        .on_press(Message::Dialog(DialogMessage::Update(
            UpdateMessage::ConfirmDownload,
        )))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_primary);

    let close_btn = button(text("Close").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD]);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Verification Failed").size(18).color(GRAY_900),
        Space::new().height(SPACING_SM),
        text("The downloaded file's SHA256 hash does not match.")
            .size(13)
            .color(GRAY_600),
        Space::new().height(SPACING_XS),
        text("This could indicate a corrupted download or tampering.")
            .size(12)
            .color(GRAY_500),
        Space::new().height(SPACING_MD),
        container(
            column![
                text(format!("Expected: {}", truncate_hash(expected)))
                    .size(10)
                    .color(GRAY_500),
                text(format!("Actual: {}", truncate_hash(actual)))
                    .size(10)
                    .color(GRAY_500),
            ]
            .spacing(2)
        )
        .padding(SPACING_SM),
        Space::new().height(SPACING_MD),
        row![retry_btn, Space::new().width(SPACING_SM), close_btn,].align_y(Alignment::Center),
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Ready to install state.
fn view_ready_to_install_state(
    info: &tss_updater::UpdateInfo,
    verified: bool,
    window_id: window::Id,
) -> Element<'_, Message> {
    let (icon, verification_text) = if verified {
        (
            lucide::shield_check().size(32).color(SUCCESS),
            text("SHA256 verification passed").size(12).color(SUCCESS),
        )
    } else {
        (
            lucide::shield_alert().size(32).color(WARNING),
            text("No verification available (digest not provided)")
                .size(12)
                .color(WARNING),
        )
    };

    let install_btn = button(
        row![
            lucide::package().size(14),
            Space::new().width(SPACING_XS),
            text("Install Update"),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Dialog(DialogMessage::Update(
        UpdateMessage::ConfirmInstall,
    )))
    .padding([SPACING_SM, SPACING_MD])
    .style(button_primary);

    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD]);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Ready to Install").size(18).color(GRAY_900),
        Space::new().height(SPACING_XS),
        text(format!("Version {}", info.version_display()))
            .size(14)
            .color(GRAY_700),
        Space::new().height(SPACING_XS),
        verification_text,
        Space::new().height(SPACING_LG),
        row![install_btn, Space::new().width(SPACING_SM), cancel_btn,].align_y(Alignment::Center),
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Installing state.
fn view_installing_state() -> Element<'static, Message> {
    let icon = lucide::loader().size(32).color(PRIMARY_500);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Installing Update...").size(16).color(GRAY_800),
        Space::new().height(SPACING_XS),
        text("Please wait, do not close the application")
            .size(13)
            .color(GRAY_500),
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Installation complete state.
fn view_install_complete_state(version: &str, window_id: window::Id) -> Element<'_, Message> {
    let icon = lucide::circle_check().size(32).color(SUCCESS);

    let restart_btn = button(
        row![
            lucide::refresh_cw().size(14),
            Space::new().width(SPACING_XS),
            text("Restart Now"),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Dialog(DialogMessage::Update(
        UpdateMessage::RestartApp,
    )))
    .padding([SPACING_SM, SPACING_MD])
    .style(button_primary);

    let later_btn = button(text("Restart Later").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD]);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Update Installed!").size(18).color(GRAY_900),
        Space::new().height(SPACING_XS),
        text(format!(
            "Version {} has been installed successfully.",
            version
        ))
        .size(13)
        .color(GRAY_500),
        Space::new().height(SPACING_XS),
        text("Restart the application to use the new version.")
            .size(12)
            .color(GRAY_500),
        Space::new().height(SPACING_LG),
        row![restart_btn, Space::new().width(SPACING_SM), later_btn,].align_y(Alignment::Center),
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Error state.
fn view_error_state(message: &str, window_id: window::Id) -> Element<'_, Message> {
    let icon = lucide::circle_x().size(32).color(GRAY_600);

    let retry_btn = button(text("Retry").size(13))
        .on_press(Message::Dialog(DialogMessage::Update(
            UpdateMessage::CheckForUpdates,
        )))
        .padding([SPACING_SM, SPACING_MD]);

    let close_btn = button(text("Close").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD]);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Update Check Failed").size(18).color(GRAY_900),
        Space::new().height(SPACING_SM),
        text(message).size(13).color(GRAY_600),
        Space::new().height(SPACING_LG),
        row![retry_btn, Space::new().width(SPACING_SM), close_btn,].align_y(Alignment::Center),
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Truncate a hash for display (show first and last 8 characters).
fn truncate_hash(hash: &str) -> String {
    if hash.len() > 20 {
        format!("{}...{}", &hash[..8], &hash[hash.len() - 8..])
    } else {
        hash.to_string()
    }
}

/// Parse changelog into markdown items for rendering.
///
/// This should be called when transitioning to the `Available` state.
pub fn parse_changelog(changelog: &str) -> Vec<markdown::Item> {
    markdown::parse(changelog).collect()
}
