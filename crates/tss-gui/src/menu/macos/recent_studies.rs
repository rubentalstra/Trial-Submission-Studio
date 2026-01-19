//! Recent studies submenu management for macOS.
//!
//! Handles dynamic updates to the Recent Studies submenu without recreating the menu.

use muda::{MenuItem, PredefinedMenuItem, Submenu};
use uuid::Uuid;

use super::super::common::ids;

/// Manages the Recent Studies submenu.
///
/// This struct holds a reference to the submenu and provides methods
/// to update its contents.
pub struct RecentStudiesSubmenu {
    submenu: Submenu,
}

impl RecentStudiesSubmenu {
    /// Create a new Recent Studies submenu with a placeholder item.
    pub fn new() -> Self {
        let submenu = Submenu::new("Recent Studies", true);

        // Add placeholder for empty state
        submenu
            .append(&MenuItem::with_id(
                ids::NO_RECENT_STUDIES,
                "No Recent Studies",
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

    /// Update the submenu with a list of recent studies.
    ///
    /// Each study needs an ID (UUID) and a display name.
    pub fn update(&self, studies: &[RecentStudyInfo]) {
        // Clear existing items
        while self.submenu.remove_at(0).is_some() {}

        if studies.is_empty() {
            self.submenu
                .append(&MenuItem::with_id(
                    ids::NO_RECENT_STUDIES,
                    "No Recent Studies",
                    false, // disabled
                    None,
                ))
                .expect("Failed to add placeholder");
        } else {
            // Add study items (max 10 for consistency)
            for study in studies.iter().take(10) {
                let id = format!("{}{}", ids::RECENT_STUDY_PREFIX, study.id);
                self.submenu
                    .append(&MenuItem::with_id(&id, &study.display_name, true, None))
                    .expect("Failed to add recent study item");
            }

            // Add separator and clear option
            self.submenu
                .append(&PredefinedMenuItem::separator())
                .expect("Failed to add separator");
            self.submenu
                .append(&MenuItem::with_id(
                    ids::CLEAR_RECENT,
                    "Clear Recent Studies",
                    true,
                    None,
                ))
                .expect("Failed to add Clear Recent Studies item");
        }
    }
}

impl Default for RecentStudiesSubmenu {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal info needed to display a recent study in the menu.
#[derive(Debug, Clone)]
pub struct RecentStudyInfo {
    /// Unique identifier for the study
    pub id: Uuid,
    /// Display name shown in the menu
    pub display_name: String,
}

impl RecentStudyInfo {
    /// Create a new recent study info.
    pub fn new(id: Uuid, display_name: String) -> Self {
        Self { id, display_name }
    }
}

/// Global function to update the recent studies menu.
///
/// This uses a thread-local to access the submenu from anywhere.
/// The submenu reference is stored when the menu bar is created.
use std::cell::RefCell;

thread_local! {
    static RECENT_SUBMENU: RefCell<Option<RecentStudiesSubmenu>> = const { RefCell::new(None) };
}

/// Store the recent submenu reference for global access.
pub fn set_recent_submenu(submenu: RecentStudiesSubmenu) {
    RECENT_SUBMENU.with(|cell| {
        *cell.borrow_mut() = Some(submenu);
    });
}

/// Update the recent studies menu with the given studies.
///
/// This is called when:
/// - A study is opened (add to recent)
/// - A study is removed from recent
/// - Recent studies are cleared
pub fn update_recent_studies_menu(studies: &[RecentStudyInfo]) {
    RECENT_SUBMENU.with(|cell| {
        if let Some(ref submenu) = *cell.borrow() {
            submenu.update(studies);
        }
    });
}
