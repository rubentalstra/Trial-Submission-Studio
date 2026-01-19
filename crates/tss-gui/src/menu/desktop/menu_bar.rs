//! Desktop menu bar view implementation.
//!
//! Renders the full in-app menu bar for Windows and Linux.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding, Theme};
use iced_fonts::lucide;

use super::components::{
    view_dropdown_container, view_menu_item, view_menu_item_disabled, view_menu_label,
    view_separator,
};
use super::state::{DropdownId, MenuDropdownState};
use crate::message::{HomeMessage, Message};
use crate::state::AppState;
use crate::theme::{GRAY_100, GRAY_200, GRAY_600, GRAY_800, SPACING_SM, SPACING_XS};

use super::super::MenuAction;

/// Render the desktop in-app menu bar.
pub fn view_menu_bar<'a>(
    state: &MenuDropdownState,
    has_study: bool,
    app_state: &'a AppState,
) -> Element<'a, Message> {
    let file_menu = view_menu_button("File", DropdownId::File, state);
    let edit_menu = view_menu_button("Edit", DropdownId::Edit, state);
    let help_menu = view_menu_button("Help", DropdownId::Help, state);

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
    match state.open {
        Some(DropdownId::File) => {
            iced::widget::stack![bar_container, view_file_dropdown(has_study, app_state)].into()
        }
        Some(DropdownId::Edit) => iced::widget::stack![bar_container, view_edit_dropdown()].into(),
        Some(DropdownId::Help) => iced::widget::stack![bar_container, view_help_dropdown()].into(),
        None => bar_container.into(),
    }
}

/// Render a menu button in the menu bar.
fn view_menu_button<'a>(
    label: &'a str,
    menu_id: DropdownId,
    state: &MenuDropdownState,
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
        .on_press(Message::MenuAction(MenuAction::from(menu_id)))
        .padding([SPACING_XS, SPACING_SM])
        .style(style)
        .into()
}

/// Render the File menu dropdown.
fn view_file_dropdown<'a>(has_study: bool, app_state: &'a AppState) -> Element<'a, Message> {
    let open_item = view_menu_item(
        lucide::folder_open().size(14).into(),
        "Open Study...",
        Some("Ctrl+O"),
        Some(Message::MenuAction(MenuAction::OpenStudy)),
    );

    let close_item = view_menu_item(
        lucide::folder_closed().size(14).into(),
        "Close Study",
        Some("Ctrl+W"),
        if has_study {
            Some(Message::MenuAction(MenuAction::CloseStudy))
        } else {
            None
        },
    );

    // Generate recent study items
    let recent_studies = app_state.settings.general.recent_sorted();
    let mut dropdown_items: Vec<Element<'a, Message>> =
        vec![open_item, close_item, view_separator()];

    // Add "Recent Studies" label
    dropdown_items.push(view_menu_label("Recent Studies"));

    if recent_studies.is_empty() {
        dropdown_items.push(view_menu_item_disabled(
            lucide::history().size(14).into(),
            "No Recent Studies",
        ));
    } else {
        // Show up to 10 recent studies (consistent with macOS)
        for study in recent_studies.iter().take(10) {
            let path = study.path.clone();
            dropdown_items.push(view_menu_item(
                lucide::folder().size(14).into(),
                &study.display_name,
                None,
                Some(Message::Home(HomeMessage::RecentStudyClicked(path))),
            ));
        }
    }

    dropdown_items.push(view_separator());

    // Clear Recent Studies
    dropdown_items.push(view_menu_item(
        lucide::trash().size(14).into(),
        "Clear Recent Studies",
        None,
        if recent_studies.is_empty() {
            None
        } else {
            Some(Message::MenuAction(MenuAction::ClearRecentStudies))
        },
    ));

    dropdown_items.push(view_separator());

    let settings_item = view_menu_item(
        lucide::settings().size(14).into(),
        "Settings...",
        Some("Ctrl+,"),
        Some(Message::MenuAction(MenuAction::Settings)),
    );

    let exit_item = view_menu_item(
        lucide::log_out().size(14).into(),
        "Exit",
        None,
        Some(Message::MenuAction(MenuAction::Quit)),
    );

    dropdown_items.push(settings_item);
    dropdown_items.push(view_separator());
    dropdown_items.push(exit_item);

    let dropdown = column(dropdown_items).width(220);

    view_dropdown_container(dropdown, 0.0)
}

/// Render the Edit menu dropdown.
fn view_edit_dropdown<'a>() -> Element<'a, Message> {
    let undo_item = view_menu_item(
        lucide::undo().size(14).into(),
        "Undo",
        Some("Ctrl+Z"),
        Some(Message::MenuAction(MenuAction::Undo)),
    );

    let redo_item = view_menu_item(
        lucide::redo().size(14).into(),
        "Redo",
        Some("Ctrl+Y"),
        Some(Message::MenuAction(MenuAction::Redo)),
    );

    let cut_item = view_menu_item(
        lucide::scissors().size(14).into(),
        "Cut",
        Some("Ctrl+X"),
        Some(Message::MenuAction(MenuAction::Cut)),
    );

    let copy_item = view_menu_item(
        lucide::copy().size(14).into(),
        "Copy",
        Some("Ctrl+C"),
        Some(Message::MenuAction(MenuAction::Copy)),
    );

    let paste_item = view_menu_item(
        lucide::clipboard().size(14).into(),
        "Paste",
        Some("Ctrl+V"),
        Some(Message::MenuAction(MenuAction::Paste)),
    );

    let select_all_item = view_menu_item(
        lucide::square_check_big().size(14).into(),
        "Select All",
        Some("Ctrl+A"),
        Some(Message::MenuAction(MenuAction::SelectAll)),
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
    let docs_item = view_menu_item(
        lucide::book_open().size(14).into(),
        "Documentation",
        None,
        Some(Message::MenuAction(MenuAction::Documentation)),
    );

    let release_notes_item = view_menu_item(
        lucide::scroll_text().size(14).into(),
        "Release Notes",
        None,
        Some(Message::MenuAction(MenuAction::ReleaseNotes)),
    );

    let github_item = view_menu_item(
        lucide::github().size(14).into(),
        "View on GitHub",
        None,
        Some(Message::MenuAction(MenuAction::ViewOnGitHub)),
    );

    let report_item = view_menu_item(
        lucide::bug().size(14).into(),
        "Report Issue...",
        None,
        Some(Message::MenuAction(MenuAction::ReportIssue)),
    );

    let license_item = view_menu_item(
        lucide::scale().size(14).into(),
        "View License",
        None,
        Some(Message::MenuAction(MenuAction::ViewLicense)),
    );

    let third_party_item = view_menu_item(
        lucide::file_text().size(14).into(),
        "Third-Party Licenses",
        None,
        Some(Message::MenuAction(MenuAction::ThirdPartyLicenses)),
    );

    let updates_item = view_menu_item(
        lucide::download().size(14).into(),
        "Check for Updates...",
        None,
        Some(Message::MenuAction(MenuAction::CheckUpdates)),
    );

    let about_item = view_menu_item(
        lucide::info().size(14).into(),
        "About",
        None,
        Some(Message::MenuAction(MenuAction::About)),
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
    view_dropdown_container(dropdown, 0.0)
}
