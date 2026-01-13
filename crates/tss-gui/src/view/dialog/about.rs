//! About dialog view.
//!
//! Displays application information, version, and links.

use iced::widget::{Space, button, column, container, row, svg, text};
use iced::window;
use iced::{Alignment, Border, Element, Length, Padding};

use crate::message::{AboutMessage, DialogMessage, Message};
use crate::theme::{
    GRAY_100, GRAY_200, GRAY_500, GRAY_800, GRAY_900, PRIMARY_500, SPACING_LG, SPACING_MD,
    SPACING_SM, SPACING_XS, WHITE,
};

/// Embedded SVG logo bytes.
const LOGO_SVG: &[u8] = include_bytes!("../../../assets/icon.svg");

/// Application version from Cargo.toml.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Render the About dialog content for a standalone window (multi-window mode).
pub fn view_about_dialog_content<'a>(window_id: window::Id) -> Element<'a, Message> {
    let content = view_dialog_content_inner(Some(window_id));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(WHITE.into()),
            ..Default::default()
        })
        .into()
}

/// Inner dialog content.
fn view_dialog_content_inner<'a>(window_id: Option<window::Id>) -> Element<'a, Message> {
    // Logo on the left
    let logo_handle = svg::Handle::from_memory(LOGO_SVG);
    let logo = svg(logo_handle).width(80).height(80);
    let logo_container = container(logo).padding(SPACING_SM);

    // App name
    let app_title = text("Trial Submission Studio").size(20).color(GRAY_900);

    // Version line
    let version_line = text(format!("Version {}", VERSION))
        .size(13)
        .color(GRAY_500);

    // Target architecture (proper Rust target triple)
    let target_line = text(get_target_triple()).size(13).color(GRAY_500);

    // View on GitHub link
    let github_btn = button(text("View on GitHub").size(13).color(PRIMARY_500))
        .on_press(Message::Dialog(DialogMessage::About(
            AboutMessage::OpenGitHub,
        )))
        .padding(0)
        .style(|_, _| button::Style {
            background: None,
            text_color: PRIMARY_500,
            ..Default::default()
        });

    // Copyright
    let copyright = text("Copyright © 2024-2026 Ruben Talstra")
        .size(12)
        .color(GRAY_500);

    // License
    let license = text("Licensed under the MIT License")
        .size(12)
        .color(GRAY_500);

    // Right side content
    let info_content = column![
        app_title,
        version_line,
        target_line,
        Space::new().height(SPACING_SM),
        github_btn,
        copyright,
        license,
    ]
    .spacing(SPACING_XS);

    // Main layout: logo left, info right
    let main_content = row![logo_container, Space::new().width(SPACING_MD), info_content,]
        .align_y(Alignment::Start)
        .padding(Padding {
            top: SPACING_LG,
            right: SPACING_LG,
            bottom: SPACING_MD,
            left: SPACING_LG,
        });

    // Divider line
    let divider =
        container(Space::new().width(Length::Fill).height(1)).style(|_| container::Style {
            background: Some(GRAY_200.into()),
            ..Default::default()
        });

    // Footer with buttons
    let footer = view_footer(window_id);

    column![main_content, divider, footer,].into()
}

/// Footer with Copy and Close, then Close buttons.
fn view_footer<'a>(window_id: Option<window::Id>) -> Element<'a, Message> {
    let copy_close_btn = button(text("Copy and Close").size(13))
        .on_press(Message::Dialog(DialogMessage::About(
            AboutMessage::CopyAndClose,
        )))
        .padding([SPACING_SM, SPACING_LG])
        .style(|theme, status| {
            let base = button::secondary(theme, status);
            button::Style {
                background: Some(GRAY_100.into()),
                text_color: GRAY_800,
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: GRAY_200,
                },
                ..base
            }
        });

    let close_btn = button(text("Close").size(13))
        .on_press(if let Some(id) = window_id {
            Message::CloseWindow(id)
        } else {
            Message::Dialog(DialogMessage::About(AboutMessage::Close))
        })
        .padding([SPACING_SM, SPACING_LG])
        .style(|theme, status| {
            let base = button::secondary(theme, status);
            button::Style {
                background: Some(GRAY_100.into()),
                text_color: GRAY_800,
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: GRAY_200,
                },
                ..base
            }
        });

    row![
        Space::new().width(Length::Fill),
        copy_close_btn,
        Space::new().width(SPACING_SM),
        close_btn,
    ]
    .padding([SPACING_MD, SPACING_LG])
    .align_y(Alignment::Center)
    .into()
}

/// Generate the system info text for copying to clipboard.
pub fn generate_system_info() -> String {
    format!(
        "Trial Submission Studio\n\
        Version {}\n\
        Target: {}",
        VERSION,
        get_target_triple(),
    )
}

/// Get the Rust target triple (e.g., aarch64-apple-darwin).
fn get_target_triple() -> &'static str {
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    {
        "aarch64-apple-darwin"
    }
    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    {
        "x86_64-apple-darwin"
    }
    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    {
        "x86_64-pc-windows-msvc"
    }
    #[cfg(all(target_arch = "aarch64", target_os = "windows"))]
    {
        "aarch64-pc-windows-msvc"
    }
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        "x86_64-unknown-linux-gnu"
    }
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    {
        "aarch64-unknown-linux-gnu"
    }
    #[cfg(not(any(
        all(target_arch = "aarch64", target_os = "macos"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "x86_64", target_os = "windows"),
        all(target_arch = "aarch64", target_os = "windows"),
        all(target_arch = "x86_64", target_os = "linux"),
        all(target_arch = "aarch64", target_os = "linux"),
    )))]
    {
        "unknown-target"
    }
}
