//! About dialog view.
//!
//! Displays application information in RustRover style.

use iced::widget::{Space, button, column, container, row, svg, text};
use iced::window;
use iced::{Alignment, Border, Element, Length, Padding};

use crate::message::{AboutMessage, DialogMessage, Message};
use crate::theme::{SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL, button_primary, colors};

/// Embedded SVG logo bytes.
const LOGO_SVG: &[u8] = include_bytes!("../../../assets/icon.svg");

/// Application version from Cargo.toml.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Render the About dialog content for a standalone window.
pub fn view_about_dialog_content<'a>(window_id: window::Id) -> Element<'a, Message> {
    let c = colors();
    let content = view_dialog_content_inner(Some(window_id));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(c.background_elevated.into()),
            ..Default::default()
        })
        .into()
}

/// Inner dialog content - RustRover style.
fn view_dialog_content_inner<'a>(window_id: Option<window::Id>) -> Element<'a, Message> {
    let c = colors();

    // Logo on the left
    let logo_handle = svg::Handle::from_memory(LOGO_SVG);
    let logo = svg(logo_handle).width(88).height(88);
    let logo_container = container(logo).padding(Padding {
        top: 0.0,
        right: SPACING_MD,
        bottom: 0.0,
        left: 0.0,
    });

    // Title
    let title = text("Trial Submission Studio")
        .size(20)
        .color(c.text_primary);

    // Version
    let version_line = text(format!("Version {}", VERSION))
        .size(13)
        .color(c.text_muted);

    // Build number (derived from version)
    let build_line = text(format!("Build {}", get_build_number()))
        .size(13)
        .color(c.text_muted);

    // Target architecture
    let target_info = text(get_target_triple()).size(13).color(c.text_muted);

    let accent_primary = c.accent_primary;

    // "Powered by open-source software" with link
    let powered_label = text("Powered by ").size(13).color(c.text_secondary);
    let open_source_link = button(text("open-source software").size(13).color(accent_primary))
        .on_press(Message::Dialog(DialogMessage::About(
            AboutMessage::OpenOpenSource,
        )))
        .padding(0)
        .style(move |_, _| button::Style {
            background: None,
            text_color: accent_primary,
            ..Default::default()
        });
    let powered_row = row![powered_label, open_source_link].align_y(Alignment::Center);

    // Copyright with link on author name
    let copyright_label = text("Copyright © 2024–2026 ")
        .size(13)
        .color(c.text_secondary);
    let author_link = button(text("Ruben Talstra").size(13).color(accent_primary))
        .on_press(Message::Dialog(DialogMessage::About(
            AboutMessage::OpenGitHub,
        )))
        .padding(0)
        .style(move |_, _| button::Style {
            background: None,
            text_color: accent_primary,
            ..Default::default()
        });
    let copyright_row = row![copyright_label, author_link].align_y(Alignment::Center);

    // Right side content
    let info_content = column![
        title,
        version_line,
        build_line,
        target_info,
        Space::new().height(SPACING_MD),
        powered_row,
        copyright_row,
    ]
    .spacing(4.0);

    // Main layout: logo left, info right
    let main_content = row![logo_container, info_content].align_y(Alignment::Start);

    // Footer with buttons
    let footer = view_footer(window_id);

    // Full layout
    column![
        container(main_content).padding(Padding {
            top: SPACING_XL,
            right: SPACING_LG,
            bottom: SPACING_LG,
            left: SPACING_LG,
        }),
        Space::new().height(Length::Fill),
        container(footer).padding(Padding {
            top: 0.0,
            right: SPACING_LG,
            bottom: SPACING_MD,
            left: SPACING_LG,
        }),
    ]
    .into()
}

/// Footer with Close and Copy and Close buttons (RustRover style).
fn view_footer<'a>(window_id: Option<window::Id>) -> Element<'a, Message> {
    let c = colors();

    // Close button (secondary/outlined style)
    let close_btn = button(text("Close").size(13))
        .on_press(if let Some(id) = window_id {
            Message::CloseWindow(id)
        } else {
            Message::Dialog(DialogMessage::About(AboutMessage::Close))
        })
        .padding([SPACING_SM, SPACING_LG])
        .style(move |theme, status| {
            let base = button::secondary(theme, status);
            button::Style {
                background: Some(c.background_secondary.into()),
                text_color: c.text_primary,
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: c.border_default,
                },
                ..base
            }
        });

    // Copy and Close button (primary/filled style - like RustRover's blue button)
    let copy_close_btn = button(text("Copy and Close").size(13))
        .on_press(Message::Dialog(DialogMessage::About(
            AboutMessage::CopyAndClose,
        )))
        .padding([SPACING_SM, SPACING_LG])
        .style(button_primary);

    row![
        Space::new().width(Length::Fill),
        close_btn,
        Space::new().width(SPACING_SM),
        copy_close_btn,
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Generate the system info text for copying to clipboard.
pub fn generate_system_info() -> String {
    format!(
        "Trial Submission Studio\n\
        Version {}\n\
        Build {}\n\
        {}\n\
        \n\
        Powered by open-source software\n\
        Copyright © 2024–2026 Ruben Talstra",
        VERSION,
        get_build_number(),
        get_target_triple(),
    )
}

/// Get build number (derived from version string for consistency).
fn get_build_number() -> u32 {
    let mut hash: u32 = 0;
    for byte in VERSION.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    hash % 1000
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
