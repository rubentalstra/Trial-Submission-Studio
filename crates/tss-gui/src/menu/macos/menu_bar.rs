//! Native macOS menu bar implementation.
//!
//! Creates the native NSMenu via muda with proper ownership.

use muda::{
    Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};
use std::cell::RefCell;

use super::super::MenuAction;
use super::super::common::ids;
use super::recent_studies::{RecentStudiesSubmenu, set_recent_submenu};

// Thread-local storage for the menu bar to keep it alive
thread_local! {
    static MENU_BAR: RefCell<Option<MenuBarStorage>> = const { RefCell::new(None) };
}

/// Storage for menu bar components that need to stay alive.
struct MenuBarStorage {
    #[allow(dead_code)]
    menu: Menu,
    #[allow(dead_code)]
    window_menu: Submenu,
}

/// Create and initialize the native macOS menu bar.
///
/// This should be called once during app startup. The menu bar is stored
/// in thread-local storage to keep it alive.
pub fn create_menu() -> Menu {
    let menu = Menu::new();

    // Create all menus
    create_app_menu(&menu);
    let recent_submenu = create_file_menu(&menu);
    create_edit_menu(&menu);
    let window_menu = create_window_menu(&menu);
    create_help_menu(&menu);

    // Store the recent submenu for later updates
    set_recent_submenu(recent_submenu);

    // Initialize for NSApp
    menu.init_for_nsapp();

    // Set window menu for proper macOS window management
    window_menu.set_as_windows_menu_for_nsapp();

    // Store in thread-local to keep alive
    let menu_clone = menu.clone();
    MENU_BAR.with(|cell| {
        *cell.borrow_mut() = Some(MenuBarStorage {
            menu: menu_clone,
            window_menu,
        });
    });

    menu
}

/// Poll for a menu event and convert to MenuAction.
///
/// This uses muda's global event receiver with try_recv for non-blocking polling.
pub fn poll_menu_event() -> Option<MenuAction> {
    let receiver = MenuEvent::receiver();

    match receiver.try_recv() {
        Ok(event) => {
            let id = event.id().0.as_str();
            menu_id_to_action(id)
        }
        Err(_) => None,
    }
}

/// Convert a menu item ID to a MenuAction.
fn menu_id_to_action(id: &str) -> Option<MenuAction> {
    // Check for recent study click (UUID-based)
    if let Some(uuid_str) = id.strip_prefix(ids::RECENT_STUDY_PREFIX)
        && let Ok(uuid) = uuid::Uuid::parse_str(uuid_str)
    {
        return Some(MenuAction::OpenRecentStudy(uuid));
    }

    match id {
        // File menu
        ids::OPEN_STUDY => Some(MenuAction::OpenStudy),
        ids::CLOSE_STUDY => Some(MenuAction::CloseStudy),
        ids::CLEAR_RECENT => Some(MenuAction::ClearRecentStudies),
        ids::SETTINGS => Some(MenuAction::Settings),
        ids::EXIT => Some(MenuAction::Quit),

        // Edit menu (stubs - predefined items don't generate events)
        ids::UNDO => Some(MenuAction::Undo),
        ids::REDO => Some(MenuAction::Redo),
        ids::CUT => Some(MenuAction::Cut),
        ids::COPY => Some(MenuAction::Copy),
        ids::PASTE => Some(MenuAction::Paste),
        ids::SELECT_ALL => Some(MenuAction::SelectAll),

        // Help menu
        ids::DOCUMENTATION => Some(MenuAction::Documentation),
        ids::RELEASE_NOTES => Some(MenuAction::ReleaseNotes),
        ids::VIEW_ON_GITHUB => Some(MenuAction::ViewOnGitHub),
        ids::REPORT_ISSUE => Some(MenuAction::ReportIssue),
        ids::VIEW_LICENSE => Some(MenuAction::ViewLicense),
        ids::THIRD_PARTY_LICENSES => Some(MenuAction::ThirdPartyLicenses),
        ids::CHECK_UPDATES => Some(MenuAction::CheckUpdates),
        ids::ABOUT => Some(MenuAction::About),

        // Unknown or predefined items (handled by system)
        _ => None,
    }
}

/// Create the macOS app menu (Trial Submission Studio menu).
fn create_app_menu(menu: &Menu) {
    let app_menu = Submenu::new("Trial Submission Studio", true);

    // About (custom, not predefined - so it triggers our dialog)
    app_menu
        .append(&MenuItem::with_id(
            ids::ABOUT,
            "About Trial Submission Studio",
            true,
            None,
        ))
        .expect("Failed to add About menu item");

    // Check for Updates
    app_menu
        .append(&MenuItem::with_id(
            ids::CHECK_UPDATES,
            "Check for Updates...",
            true,
            None,
        ))
        .expect("Failed to add Check for Updates menu item");

    app_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    // Settings (Cmd+,)
    app_menu
        .append(&MenuItem::with_id(
            ids::SETTINGS,
            "Settings...",
            true,
            Some(Accelerator::new(Some(Modifiers::META), Code::Comma)),
        ))
        .expect("Failed to add Settings menu item");

    app_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    // Hide, Hide Others, Show All (system predefined)
    app_menu
        .append(&PredefinedMenuItem::hide(None))
        .expect("Failed to add Hide menu item");
    app_menu
        .append(&PredefinedMenuItem::hide_others(None))
        .expect("Failed to add Hide Others menu item");
    app_menu
        .append(&PredefinedMenuItem::show_all(None))
        .expect("Failed to add Show All menu item");

    app_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    // Quit (system predefined)
    app_menu
        .append(&PredefinedMenuItem::quit(None))
        .expect("Failed to add Quit menu item");

    menu.append(&app_menu).expect("Failed to add app menu");
}

/// Create the File menu.
fn create_file_menu(menu: &Menu) -> RecentStudiesSubmenu {
    let file_menu = Submenu::new("File", true);

    // Open Study (Cmd+O)
    file_menu
        .append(&MenuItem::with_id(
            ids::OPEN_STUDY,
            "Open Study...",
            true,
            Some(Accelerator::new(Some(Modifiers::META), Code::KeyO)),
        ))
        .expect("Failed to add Open Study menu item");

    // Create Recent Studies submenu
    let recent_submenu = RecentStudiesSubmenu::new();
    file_menu
        .append(recent_submenu.submenu())
        .expect("Failed to add Recent Studies submenu");

    file_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    // Close Study (Cmd+W)
    file_menu
        .append(&MenuItem::with_id(
            ids::CLOSE_STUDY,
            "Close Study",
            true,
            Some(Accelerator::new(Some(Modifiers::META), Code::KeyW)),
        ))
        .expect("Failed to add Close Study menu item");

    menu.append(&file_menu).expect("Failed to add File menu");

    recent_submenu
}

/// Create the Edit menu with predefined system items.
fn create_edit_menu(menu: &Menu) {
    let edit_menu = Submenu::new("Edit", true);

    // Use predefined items for system integration
    edit_menu
        .append(&PredefinedMenuItem::undo(None))
        .expect("Failed to add Undo menu item");
    edit_menu
        .append(&PredefinedMenuItem::redo(None))
        .expect("Failed to add Redo menu item");

    edit_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    edit_menu
        .append(&PredefinedMenuItem::cut(None))
        .expect("Failed to add Cut menu item");
    edit_menu
        .append(&PredefinedMenuItem::copy(None))
        .expect("Failed to add Copy menu item");
    edit_menu
        .append(&PredefinedMenuItem::paste(None))
        .expect("Failed to add Paste menu item");

    edit_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    edit_menu
        .append(&PredefinedMenuItem::select_all(None))
        .expect("Failed to add Select All menu item");

    menu.append(&edit_menu).expect("Failed to add Edit menu");
}

/// Create the Window menu (macOS standard).
fn create_window_menu(menu: &Menu) -> Submenu {
    let window_menu = Submenu::new("Window", true);

    // Minimize (Cmd+M)
    window_menu
        .append(&PredefinedMenuItem::minimize(None))
        .expect("Failed to add Minimize menu item");

    // Zoom (macOS calls maximize "Zoom")
    window_menu
        .append(&PredefinedMenuItem::maximize(None))
        .expect("Failed to add Zoom menu item");

    // Full Screen
    window_menu
        .append(&PredefinedMenuItem::fullscreen(None))
        .expect("Failed to add Full Screen menu item");

    window_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    // Bring All to Front
    window_menu
        .append(&PredefinedMenuItem::bring_all_to_front(None))
        .expect("Failed to add Bring All to Front menu item");

    menu.append(&window_menu)
        .expect("Failed to add Window menu");

    window_menu
}

/// Create the Help menu.
fn create_help_menu(menu: &Menu) {
    let help_menu = Submenu::new("Help", true);

    // Documentation
    help_menu
        .append(&MenuItem::with_id(
            ids::DOCUMENTATION,
            "Documentation",
            true,
            None,
        ))
        .expect("Failed to add Documentation menu item");

    // Release Notes
    help_menu
        .append(&MenuItem::with_id(
            ids::RELEASE_NOTES,
            "Release Notes",
            true,
            None,
        ))
        .expect("Failed to add Release Notes menu item");

    help_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    // View on GitHub
    help_menu
        .append(&MenuItem::with_id(
            ids::VIEW_ON_GITHUB,
            "View on GitHub",
            true,
            None,
        ))
        .expect("Failed to add View on GitHub menu item");

    // Report Issue
    help_menu
        .append(&MenuItem::with_id(
            ids::REPORT_ISSUE,
            "Report Issue...",
            true,
            None,
        ))
        .expect("Failed to add Report Issue menu item");

    help_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    // View License
    help_menu
        .append(&MenuItem::with_id(
            ids::VIEW_LICENSE,
            "View License",
            true,
            None,
        ))
        .expect("Failed to add View License menu item");

    // Third-Party Licenses
    help_menu
        .append(&MenuItem::with_id(
            ids::THIRD_PARTY_LICENSES,
            "Third-Party Licenses",
            true,
            None,
        ))
        .expect("Failed to add Third-Party Licenses menu item");

    menu.append(&help_menu).expect("Failed to add Help menu");
}
