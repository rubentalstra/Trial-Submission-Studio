//! Native menu bar implementation using the `muda` crate.
//!
//! Platform-specific behavior:
//! - macOS: App menu with About, Settings, Hide, Quit + Edit/Window menus
//! - Windows/Linux: File menu with Open Study, Settings, Exit + Edit menu

use muda::{
    Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};

/// Menu action identifiers.
pub mod ids {
    // File menu
    pub const OPEN_STUDY: &str = "open_study";
    #[allow(dead_code)] // Planned for future "Open Recent" submenu
    pub const CLEAR_RECENT: &str = "clear_recent";

    // App/Settings
    pub const SETTINGS: &str = "settings";
    pub const CHECK_UPDATES: &str = "check_updates";
    pub const ABOUT: &str = "about";
    pub const EXIT: &str = "exit";

    // Help menu
    pub const DOCUMENTATION: &str = "documentation";
    pub const RELEASE_NOTES: &str = "release_notes";
    pub const VIEW_ON_GITHUB: &str = "view_on_github";
    pub const REPORT_ISSUE: &str = "report_issue";
    pub const VIEW_LICENSE: &str = "view_license";
    pub const THIRD_PARTY_LICENSES: &str = "third_party_licenses";
}

/// Create the native menu bar.
///
/// On macOS, this creates the app menu but does NOT call `init_for_nsapp()`.
/// Call `init_menu_for_nsapp()` after the application event loop has started.
///
/// Returns the menu. Use `menu_event_receiver()` to get menu events.
pub fn create_menu() -> Menu {
    let menu = Menu::new();

    // macOS: App menu with app name
    #[cfg(target_os = "macos")]
    {
        create_macos_app_menu(&menu);
    }

    // File menu (all platforms)
    create_file_menu(&menu);

    // Edit menu (all platforms)
    create_edit_menu(&menu);

    // Window menu (macOS only)
    #[cfg(target_os = "macos")]
    {
        create_window_menu(&menu);
    }

    // Help menu (all platforms)
    create_help_menu(&menu);

    menu
}

/// Create the macOS app menu.
#[cfg(target_os = "macos")]
fn create_macos_app_menu(menu: &Menu) {
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

    // Check for Updates (no separator before)
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

    // Hide, Hide Others, Show All
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

    // Quit
    app_menu
        .append(&PredefinedMenuItem::quit(None))
        .expect("Failed to add Quit menu item");

    menu.append(&app_menu).expect("Failed to add app menu");
}

/// Create the File menu (all platforms).
fn create_file_menu(menu: &Menu) {
    let file_menu = Submenu::new("File", true);

    // Open Study (Cmd/Ctrl+O)
    let open_accel = if cfg!(target_os = "macos") {
        Accelerator::new(Some(Modifiers::META), Code::KeyO)
    } else {
        Accelerator::new(Some(Modifiers::CONTROL), Code::KeyO)
    };

    file_menu
        .append(&MenuItem::with_id(
            ids::OPEN_STUDY,
            "Open Study...",
            true,
            Some(open_accel),
        ))
        .expect("Failed to add Open Study menu item");

    // TODO: Add "Open Recent" submenu with recent files
    // This requires tracking recent files in settings

    file_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    // Windows/Linux only: Settings and Exit in File menu
    #[cfg(not(target_os = "macos"))]
    {
        // Settings (Ctrl+,)
        file_menu
            .append(&MenuItem::with_id(
                ids::SETTINGS,
                "Settings...",
                true,
                Some(Accelerator::new(Some(Modifiers::CONTROL), Code::Comma)),
            ))
            .expect("Failed to add Settings menu item");

        file_menu
            .append(&PredefinedMenuItem::separator())
            .expect("Failed to add separator");

        // Exit
        file_menu
            .append(&MenuItem::with_id(ids::EXIT, "Exit", true, None))
            .expect("Failed to add Exit menu item");
    }

    menu.append(&file_menu).expect("Failed to add File menu");
}

/// Create the Edit menu (all platforms).
fn create_edit_menu(menu: &Menu) {
    let edit_menu = Submenu::new("Edit", true);

    // Undo
    edit_menu
        .append(&PredefinedMenuItem::undo(None))
        .expect("Failed to add Undo menu item");

    // Redo
    edit_menu
        .append(&PredefinedMenuItem::redo(None))
        .expect("Failed to add Redo menu item");

    edit_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    // Cut
    edit_menu
        .append(&PredefinedMenuItem::cut(None))
        .expect("Failed to add Cut menu item");

    // Copy
    edit_menu
        .append(&PredefinedMenuItem::copy(None))
        .expect("Failed to add Copy menu item");

    // Paste
    edit_menu
        .append(&PredefinedMenuItem::paste(None))
        .expect("Failed to add Paste menu item");

    edit_menu
        .append(&PredefinedMenuItem::separator())
        .expect("Failed to add separator");

    // Select All
    edit_menu
        .append(&PredefinedMenuItem::select_all(None))
        .expect("Failed to add Select All menu item");

    menu.append(&edit_menu).expect("Failed to add Edit menu");
}

/// Create the Window menu (macOS only).
#[cfg(target_os = "macos")]
fn create_window_menu(menu: &Menu) {
    let window_menu = Submenu::new("Window", true);

    // Minimize
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
}

/// Create the Help menu (all platforms).
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

    // Windows/Linux only: Check for Updates and About
    #[cfg(not(target_os = "macos"))]
    {
        help_menu
            .append(&PredefinedMenuItem::separator())
            .expect("Failed to add separator");

        help_menu
            .append(&MenuItem::with_id(
                ids::CHECK_UPDATES,
                "Check for Updates...",
                true,
                None,
            ))
            .expect("Failed to add Check for Updates menu item");

        help_menu
            .append(&MenuItem::with_id(ids::ABOUT, "About", true, None))
            .expect("Failed to add About menu item");
    }

    menu.append(&help_menu).expect("Failed to add Help menu");
}

/// Initialize the menu for macOS NSApp.
///
/// This must be called after the application event loop has started.
/// On non-macOS platforms, this is a no-op.
#[cfg(target_os = "macos")]
pub fn init_menu_for_nsapp(menu: &Menu) {
    menu.init_for_nsapp();
}

/// Initialize the menu for macOS NSApp (no-op on other platforms).
#[cfg(not(target_os = "macos"))]
pub fn init_menu_for_nsapp(_menu: &Menu) {
    // No-op on non-macOS platforms
}

/// Get the menu event receiver.
///
/// Muda uses crossbeam_channel internally. Call `try_recv()` to poll for events.
pub fn menu_event_receiver() -> crossbeam_channel::Receiver<MenuEvent> {
    MenuEvent::receiver().clone()
}
