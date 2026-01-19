//! Desktop menu dropdown state.
//!
//! Simple state for tracking which dropdown is currently open.

/// Identifies which dropdown menu is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropdownId {
    File,
    Edit,
    Help,
}

/// State for the desktop menu bar dropdowns.
#[derive(Debug, Clone, Default)]
pub struct MenuDropdownState {
    /// Currently open dropdown (if any).
    pub open: Option<DropdownId>,
}

impl MenuDropdownState {
    /// Toggle a dropdown open/closed.
    pub fn toggle(&mut self, id: DropdownId) {
        if self.open == Some(id) {
            self.open = None;
        } else {
            self.open = Some(id);
        }
    }

    /// Close all dropdowns.
    pub fn close(&mut self) {
        self.open = None;
    }

    /// Check if a specific dropdown is open.
    pub fn is_open(&self, id: DropdownId) -> bool {
        self.open == Some(id)
    }
}
