//! Native menu bar implementation using the `muda` crate.
//!
//! Platform-specific behavior:
//! - macOS: App menu with About, Settings, Hide, Quit in the app name menu
//! - Windows/Linux: File menu with Open Study, Settings, Exit

use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};

/// Menu action identifiers.
pub mod ids {
    pub const OPEN_STUDY: &str = "open_study";
    pub const SETTINGS: &str = "settings";
    pub const ABOUT: &str = "about";
    pub const EXIT: &str = "exit";
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
        let app_menu = Submenu::new("CDISC Transpiler", true);

        // About
        app_menu
            .append(&PredefinedMenuItem::about(
                Some("About CDISC Transpiler"),
                None,
            ))
            .expect("Failed to add About menu item");

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

    // File menu (all platforms)
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

    // TODO: Add Recent Studies submenu

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

    // Help menu
    let help_menu = Submenu::new("Help", true);

    #[cfg(not(target_os = "macos"))]
    {
        help_menu
            .append(&MenuItem::with_id(ids::ABOUT, "About", true, None))
            .expect("Failed to add About menu item");
    }

    // On macOS, just add a placeholder or additional help items
    #[cfg(target_os = "macos")]
    {
        help_menu
            .append(&MenuItem::with_id(
                "help_docs",
                "Documentation",
                true,
                None,
            ))
            .expect("Failed to add Documentation menu item");
    }

    menu.append(&help_menu).expect("Failed to add Help menu");

    menu
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
