//! In-app menu bar for Windows and Linux.
//!
//! On macOS, the native menu bar is used instead (via muda).
//! This module provides an Iced-based menu bar rendered inside the application window.

use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding, Theme};
use iced_fonts::lucide;

use crate::message::{MenuBarMenuId, MenuMessage, Message};
use crate::theme::{GRAY_200, GRAY_600, GRAY_800, SPACING_SM, SPACING_XS};

/// Re-export MenuId as an alias for MenuBarMenuId for convenience.
pub type MenuId = MenuBarMenuId;

/// Menu bar state for tracking open menus.
#[derive(Debug, Clone, Default)]
pub struct MenuBarState {
    /// Currently open menu (if any).
    pub open_menu: Option<MenuId>,
}

impl MenuBarState {
    /// Create a new menu bar state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle a menu open/closed.
    pub fn toggle(&mut self, menu: MenuId) {
        if self.open_menu == Some(menu) {
            self.open_menu = None;
        } else {
            self.open_menu = Some(menu);
        }
    }

    /// Close all menus.
    pub fn close(&mut self) {
        self.open_menu = None;
    }

    /// Check if a specific menu is open.
    pub fn is_open(&self, menu: MenuId) -> bool {
        self.open_menu == Some(menu)
    }
}

/// Render the in-app menu bar.
///
/// This is only used on Windows and Linux. On macOS, the native menu bar is used.
#[cfg(not(target_os = "macos"))]
pub fn view_menu_bar<'a>(state: &MenuBarState, has_study: bool) -> Element<'a, Message> {
    let file_menu = view_menu_button("File", MenuBarMenuId::File, state);
    let edit_menu = view_menu_button("Edit", MenuBarMenuId::Edit, state);
    let help_menu = view_menu_button("Help", MenuBarMenuId::Help, state);

    let bar = row![
        file_menu,
        edit_menu,
        Space::new().width(Length::Fill),
        help_menu,
    ]
    .spacing(SPACING_XS)
    .align_y(Alignment::Center)
    .padding(Padding::from([SPACING_XS, SPACING_SM]));

    let bar_container =
        container(bar)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(GRAY_100.into()),
                border: Border {
                    color: GRAY_200,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            });

    // If a menu is open, render the dropdown
    match state.open_menu {
        Some(MenuBarMenuId::File) => {
            iced::widget::stack![bar_container, view_file_dropdown(has_study)].into()
        }
        Some(MenuBarMenuId::Edit) => {
            iced::widget::stack![bar_container, view_edit_dropdown()].into()
        }
        Some(MenuBarMenuId::Help) => {
            iced::widget::stack![bar_container, view_help_dropdown()].into()
        }
        None => bar_container.into(),
    }
}

/// Render a menu button in the menu bar.
fn view_menu_button<'a>(
    label: &'a str,
    menu_id: MenuBarMenuId,
    state: &MenuBarState,
) -> Element<'a, Message> {
    let is_active = state.is_open(menu_id);

    let style = move |_theme: &Theme, _status: button::Status| {
        if is_active {
            button::Style {
                background: Some(GRAY_200.into()),
                text_color: GRAY_800,
                border: Border::default(),
                ..Default::default()
            }
        } else {
            button::Style {
                background: None,
                text_color: GRAY_600,
                border: Border::default(),
                ..Default::default()
            }
        }
    };

    button(text(label).size(13))
        .on_press(Message::MenuBarToggle(menu_id))
        .padding([SPACING_XS, SPACING_SM])
        .style(style)
        .into()
}

/// Render the File menu dropdown.
fn view_file_dropdown<'a>(has_study: bool) -> Element<'a, Message> {
    use iced::widget::column;

    let open_item = view_menu_item(
        lucide::folder_open().size(14).into(),
        "Open Study...",
        Some("Ctrl+O"),
        Some(Message::Menu(MenuMessage::OpenStudy)),
    );

    let close_item = view_menu_item(
        lucide::folder_closed().size(14).into(),
        "Close Study",
        Some("Ctrl+W"),
        if has_study {
            Some(Message::Menu(MenuMessage::CloseStudy))
        } else {
            None
        },
    );

    let settings_item = view_menu_item(
        lucide::settings().size(14).into(),
        "Settings...",
        Some("Ctrl+,"),
        Some(Message::Menu(MenuMessage::Settings)),
    );

    let exit_item = view_menu_item(
        lucide::log_out().size(14).into(),
        "Exit",
        None,
        Some(Message::Menu(MenuMessage::Quit)),
    );

    let dropdown = column![
        open_item,
        close_item,
        view_separator(),
        settings_item,
        view_separator(),
        exit_item,
    ]
    .width(220);

    view_dropdown_container(dropdown, 0.0)
}

/// Render the Edit menu dropdown.
fn view_edit_dropdown<'a>() -> Element<'a, Message> {
    use iced::widget::column;

    let undo_item = view_menu_item(
        lucide::undo().size(14).into(),
        "Undo",
        Some("Ctrl+Z"),
        Some(Message::Menu(MenuMessage::Undo)),
    );

    let redo_item = view_menu_item(
        lucide::redo().size(14).into(),
        "Redo",
        Some("Ctrl+Y"),
        Some(Message::Menu(MenuMessage::Redo)),
    );

    let cut_item = view_menu_item(
        lucide::scissors().size(14).into(),
        "Cut",
        Some("Ctrl+X"),
        Some(Message::Menu(MenuMessage::Cut)),
    );

    let copy_item = view_menu_item(
        lucide::copy().size(14).into(),
        "Copy",
        Some("Ctrl+C"),
        Some(Message::Menu(MenuMessage::Copy)),
    );

    let paste_item = view_menu_item(
        lucide::clipboard().size(14).into(),
        "Paste",
        Some("Ctrl+V"),
        Some(Message::Menu(MenuMessage::Paste)),
    );

    let select_all_item = view_menu_item(
        lucide::square_check_big().size(14).into(),
        "Select All",
        Some("Ctrl+A"),
        Some(Message::Menu(MenuMessage::SelectAll)),
    );

    let dropdown = column![
        undo_item,
        redo_item,
        view_separator(),
        cut_item,
        copy_item,
        paste_item,
        view_separator(),
        select_all_item,
    ]
    .width(200);

    // Position after File menu button
    view_dropdown_container(dropdown, 50.0)
}

/// Render the Help menu dropdown.
fn view_help_dropdown<'a>() -> Element<'a, Message> {
    use iced::widget::column;

    let docs_item = view_menu_item(
        lucide::book_open().size(14).into(),
        "Documentation",
        None,
        Some(Message::Menu(MenuMessage::Documentation)),
    );

    let release_notes_item = view_menu_item(
        lucide::scroll_text().size(14).into(),
        "Release Notes",
        None,
        Some(Message::Menu(MenuMessage::ReleaseNotes)),
    );

    let github_item = view_menu_item(
        lucide::github().size(14).into(),
        "View on GitHub",
        None,
        Some(Message::Menu(MenuMessage::ViewOnGitHub)),
    );

    let report_item = view_menu_item(
        lucide::bug().size(14).into(),
        "Report Issue...",
        None,
        Some(Message::Menu(MenuMessage::ReportIssue)),
    );

    let license_item = view_menu_item(
        lucide::scale().size(14).into(),
        "View License",
        None,
        Some(Message::Menu(MenuMessage::ViewLicense)),
    );

    let third_party_item = view_menu_item(
        lucide::file_text().size(14).into(),
        "Third-Party Licenses",
        None,
        Some(Message::Menu(MenuMessage::ThirdPartyLicenses)),
    );

    let updates_item = view_menu_item(
        lucide::download().size(14).into(),
        "Check for Updates...",
        None,
        Some(Message::Menu(MenuMessage::CheckUpdates)),
    );

    let about_item = view_menu_item(
        lucide::info().size(14).into(),
        "About",
        None,
        Some(Message::Menu(MenuMessage::About)),
    );

    let dropdown = column![
        docs_item,
        release_notes_item,
        view_separator(),
        github_item,
        report_item,
        view_separator(),
        license_item,
        third_party_item,
        view_separator(),
        updates_item,
        about_item,
    ]
    .width(220);

    // Position at right side (will need adjustment based on actual layout)
    view_dropdown_container(dropdown, 0.0) // Positioned via stack alignment
}

/// Render a menu item.
fn view_menu_item<'a>(
    icon: Element<'a, Message>,
    label: &'a str,
    shortcut: Option<&'a str>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let is_enabled = on_press.is_some();
    let text_color = if is_enabled { GRAY_800 } else { GRAY_600 };

    let content = row![
        container(icon).width(20),
        Space::new().width(SPACING_XS),
        text(label).size(13).color(text_color),
        Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let content = if let Some(shortcut) = shortcut {
        row![content, text(shortcut).size(11).color(GRAY_600),].align_y(Alignment::Center)
    } else {
        content
    };

    let btn = button(content)
        .padding([SPACING_XS, SPACING_SM])
        .width(Length::Fill)
        .style(menu_item_style);

    if let Some(msg) = on_press {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}

/// Render a menu separator.
fn view_separator<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fill).height(1))
        .style(|_theme: &Theme| container::Style {
            background: Some(GRAY_200.into()),
            ..Default::default()
        })
        .padding(Padding::from([SPACING_XS, 0.0]))
        .into()
}

/// Wrap a dropdown in a positioned container.
fn view_dropdown_container<'a>(
    content: impl Into<Element<'a, Message>>,
    _left_offset: f32,
) -> Element<'a, Message> {
    use crate::theme::{BORDER_RADIUS_MD, WHITE};

    // Dropdown with shadow and border
    container(content)
        .style(|_theme: &Theme| container::Style {
            background: Some(WHITE.into()),
            border: Border {
                color: GRAY_200,
                width: 1.0,
                radius: BORDER_RADIUS_MD.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        })
        .padding(SPACING_XS)
        .into()
}

/// Style for menu items.
fn menu_item_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: GRAY_800,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// No-op view for macOS (uses native menu).
#[cfg(target_os = "macos")]
pub fn view_menu_bar<'a>(_state: &MenuBarState, _has_study: bool) -> Element<'a, Message> {
    Space::new().width(0).height(0).into()
}
