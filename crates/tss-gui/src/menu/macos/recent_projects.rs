//! Recent projects submenu management for macOS.
//!
//! Handles dynamic updates to the Recent Projects submenu without recreating the menu.

use muda::{MenuItem, PredefinedMenuItem, Submenu};
use uuid::Uuid;

use super::super::common::ids;

/// Manages the Recent Projects submenu.
///
/// This struct holds a reference to the submenu and provides methods
/// to update its contents.
pub struct RecentProjectsSubmenu {
    submenu: Submenu,
}

impl RecentProjectsSubmenu {
    /// Create a new Recent Projects submenu with a placeholder item.
    pub fn new() -> Self {
        let submenu = Submenu::new("Recent Projects", true);

        // Add placeholder for empty state
        submenu
            .append(&MenuItem::with_id(
                ids::NO_RECENT_PROJECTS,
                "No Recent Projects",
                false, // disabled
                None,
            ))
            .expect("Failed to add placeholder");

        Self { submenu }
    }

    /// Get a reference to the underlying submenu for appending to parent menu.
    pub fn submenu(&self) -> &Submenu {
        &self.submenu
    }

    /// Update the submenu with a list of recent projects.
    ///
    /// Each project needs an ID (UUID) and a display name.
    pub fn update(&self, projects: &[RecentProjectInfo]) {
        // Clear existing items
        while self.submenu.remove_at(0).is_some() {}

        if projects.is_empty() {
            self.submenu
                .append(&MenuItem::with_id(
                    ids::NO_RECENT_PROJECTS,
                    "No Recent Projects",
                    false, // disabled
                    None,
                ))
                .expect("Failed to add placeholder");
        } else {
            // Add project items (max 10 for consistency)
            for project in projects.iter().take(10) {
                let id = format!("{}{}", ids::RECENT_PROJECT_PREFIX, project.id);
                self.submenu
                    .append(&MenuItem::with_id(&id, &project.display_name, true, None))
                    .expect("Failed to add recent project item");
            }

            // Add separator and clear option
            self.submenu
                .append(&PredefinedMenuItem::separator())
                .expect("Failed to add separator");
            self.submenu
                .append(&MenuItem::with_id(
                    ids::CLEAR_RECENT,
                    "Clear Recent Projects",
                    true,
                    None,
                ))
                .expect("Failed to add Clear Recent Projects item");
        }
    }
}

impl Default for RecentProjectsSubmenu {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal info needed to display a recent project in the menu.
#[derive(Debug, Clone)]
pub struct RecentProjectInfo {
    /// Unique identifier for the project
    pub id: Uuid,
    /// Display name shown in the menu
    pub display_name: String,
}

impl RecentProjectInfo {
    /// Create a new recent project info.
    pub fn new(id: Uuid, display_name: String) -> Self {
        Self { id, display_name }
    }
}

/// Global function to update the recent projects menu.
///
/// This uses a thread-local to access the submenu from anywhere.
/// The submenu reference is stored when the menu bar is created.
use std::cell::RefCell;

thread_local! {
    static RECENT_SUBMENU: RefCell<Option<RecentProjectsSubmenu>> = const { RefCell::new(None) };
}

/// Store the recent submenu reference for global access.
pub fn set_recent_submenu(submenu: RecentProjectsSubmenu) {
    RECENT_SUBMENU.with(|cell| {
        *cell.borrow_mut() = Some(submenu);
    });
}

/// Update the recent projects menu with the given projects.
///
/// This is called when:
/// - A project is opened (add to recent)
/// - A project is removed from recent
/// - Recent projects are cleared
pub fn update_recent_projects_menu(projects: &[RecentProjectInfo]) {
    RECENT_SUBMENU.with(|cell| {
        if let Some(ref submenu) = *cell.borrow() {
            submenu.update(projects);
        }
    });
}
