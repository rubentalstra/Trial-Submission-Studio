//! Update dialog view.
//!
//! Check for and install application updates.

use iced::widget::{Space, button, column, container, progress_bar, row, text};
use iced::{Alignment, Border, Color, Element, Length};
use iced_fonts::lucide;

use crate::message::{DialogMessage, Message, UpdateInfo, UpdateMessage};
use crate::theme::{
    BORDER_RADIUS_LG, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900, PRIMARY_500, SPACING_LG,
    SPACING_MD, SPACING_SM, SPACING_XS, SUCCESS, WHITE, button_primary,
};

/// Current application version.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Update check state.
#[derive(Debug, Clone, Default)]
pub enum UpdateState {
    /// Initial state, no check performed.
    #[default]
    Idle,
    /// Checking for updates.
    Checking,
    /// Update available.
    Available(UpdateInfo),
    /// No update available.
    UpToDate,
    /// Error checking for updates.
    Error(String),
    /// Downloading/installing update.
    Installing { progress: f32 },
    /// Installation complete.
    InstallComplete,
}

/// Render the Update dialog.
pub fn view_update_dialog(state: &UpdateState) -> Element<Message> {
    let backdrop = container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
            ..Default::default()
        });

    let dialog_content = view_dialog_content(state);

    let dialog = container(dialog_content)
        .width(450)
        .style(|_| container::Style {
            background: Some(WHITE.into()),
            border: Border {
                radius: BORDER_RADIUS_LG.into(),
                ..Default::default()
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        });

    let centered_dialog = container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Shrink)
        .center_y(Length::Shrink);

    iced::widget::stack![backdrop, centered_dialog].into()
}

/// Dialog content based on state.
fn view_dialog_content(state: &UpdateState) -> Element<Message> {
    match state {
        UpdateState::Idle => view_idle_state(),
        UpdateState::Checking => view_checking_state(),
        UpdateState::Available(info) => view_available_state(info),
        UpdateState::UpToDate => view_up_to_date_state(),
        UpdateState::Error(msg) => view_error_state(msg),
        UpdateState::Installing { progress } => view_installing_state(*progress),
        UpdateState::InstallComplete => view_install_complete_state(),
    }
}

/// Idle state - prompt to check for updates.
fn view_idle_state<'a>() -> Element<'a, Message> {
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
        .on_press(Message::Dialog(DialogMessage::Update(UpdateMessage::Close)))
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
fn view_checking_state<'a>() -> Element<'a, Message> {
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

/// Update available state.
fn view_available_state(info: &UpdateInfo) -> Element<Message> {
    let icon = lucide::download().size(32).color(SUCCESS);

    let install_btn = button(
        row![
            lucide::download().size(14),
            Space::new().width(SPACING_XS),
            text("Install Update"),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Dialog(DialogMessage::Update(
        UpdateMessage::StartInstall,
    )))
    .padding([SPACING_SM, SPACING_MD])
    .style(button_primary);

    let later_btn = button(text("Later").size(13))
        .on_press(Message::Dialog(DialogMessage::Update(UpdateMessage::Close)))
        .padding([SPACING_SM, SPACING_MD]);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Update Available!").size(18).color(GRAY_900),
        Space::new().height(SPACING_SM),
        text(format!("Version {} is available", info.version))
            .size(14)
            .color(GRAY_700),
        Space::new().height(SPACING_XS),
        text(format!("Current: {}", CURRENT_VERSION))
            .size(12)
            .color(GRAY_500),
        Space::new().height(SPACING_LG),
        install_btn,
        Space::new().height(SPACING_SM),
        later_btn,
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Up to date state.
fn view_up_to_date_state<'a>() -> Element<'a, Message> {
    let icon = lucide::circle_check().size(32).color(SUCCESS);

    let close_btn = button(text("Close").size(13))
        .on_press(Message::Dialog(DialogMessage::Update(UpdateMessage::Close)))
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

/// Error state.
fn view_error_state(message: &str) -> Element<Message> {
    let icon = lucide::circle_x().size(32).color(GRAY_600);

    let retry_btn = button(text("Retry").size(13))
        .on_press(Message::Dialog(DialogMessage::Update(
            UpdateMessage::CheckForUpdates,
        )))
        .padding([SPACING_SM, SPACING_MD]);

    let close_btn = button(text("Close").size(13))
        .on_press(Message::Dialog(DialogMessage::Update(UpdateMessage::Close)))
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

/// Installing state with progress.
fn view_installing_state<'a>(progress: f32) -> Element<'a, Message> {
    let icon = lucide::download().size(32).color(PRIMARY_500);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Installing Update...").size(18).color(GRAY_900),
        Space::new().height(SPACING_MD),
        container(progress_bar(0.0..=1.0, progress)).width(300),
        Space::new().height(SPACING_XS),
        text(format!("{}%", (progress * 100.0) as u32))
            .size(12)
            .color(GRAY_500),
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Installation complete state.
fn view_install_complete_state<'a>() -> Element<'a, Message> {
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

    let later_btn = button(text("Later").size(13))
        .on_press(Message::Dialog(DialogMessage::Update(UpdateMessage::Close)))
        .padding([SPACING_SM, SPACING_MD]);

    column![
        Space::new().height(SPACING_LG),
        icon,
        Space::new().height(SPACING_MD),
        text("Update Ready!").size(18).color(GRAY_900),
        Space::new().height(SPACING_XS),
        text("Restart to complete the installation")
            .size(13)
            .color(GRAY_500),
        Space::new().height(SPACING_LG),
        restart_btn,
        Space::new().height(SPACING_SM),
        later_btn,
        Space::new().height(SPACING_LG),
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}
