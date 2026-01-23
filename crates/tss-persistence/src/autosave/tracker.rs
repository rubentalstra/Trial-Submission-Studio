//! Dirty state tracking for auto-save.

use std::time::Instant;

/// Tracks unsaved changes in a project.
///
/// Used to implement debounced auto-save and the "unsaved changes" indicator.
#[derive(Debug, Clone)]
pub struct DirtyTracker {
    /// Whether there are unsaved changes.
    dirty: bool,

    /// When the most recent change was made.
    last_change: Option<Instant>,

    /// When the first unsaved change was made.
    /// Reset when saved.
    first_unsaved_change: Option<Instant>,

    /// Whether a save is currently in progress.
    saving: bool,
}

impl Default for DirtyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DirtyTracker {
    /// Create a new tracker with no unsaved changes.
    pub fn new() -> Self {
        Self {
            dirty: false,
            last_change: None,
            first_unsaved_change: None,
            saving: false,
        }
    }

    /// Check if there are unsaved changes.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Check if a save is in progress.
    #[inline]
    pub fn is_saving(&self) -> bool {
        self.saving
    }

    /// Mark the project as having unsaved changes.
    pub fn mark_dirty(&mut self) {
        let now = Instant::now();
        self.dirty = true;
        self.last_change = Some(now);

        // Only set first_unsaved_change if this is the first change since last save
        if self.first_unsaved_change.is_none() {
            self.first_unsaved_change = Some(now);
        }
    }

    /// Mark that a save has started.
    pub fn start_save(&mut self) {
        self.saving = true;
    }

    /// Mark that a save has completed successfully.
    pub fn save_complete(&mut self) {
        self.dirty = false;
        self.saving = false;
        self.first_unsaved_change = None;
    }

    /// Mark that a save has failed.
    pub fn save_failed(&mut self) {
        self.saving = false;
        // Keep dirty = true since save failed
    }

    /// Get milliseconds since the last change.
    pub fn ms_since_last_change(&self) -> Option<u64> {
        self.last_change.map(|t| t.elapsed().as_millis() as u64)
    }

    /// Get milliseconds since the first unsaved change.
    pub fn ms_since_first_unsaved(&self) -> Option<u64> {
        self.first_unsaved_change
            .map(|t| t.elapsed().as_millis() as u64)
    }

    /// Check if auto-save should trigger based on the config.
    pub fn should_auto_save(&self, config: &super::AutoSaveConfig) -> bool {
        if !self.dirty || self.saving || !config.enabled {
            return false;
        }

        match (self.ms_since_last_change(), self.ms_since_first_unsaved()) {
            (Some(since_last), Some(since_first)) => config.should_save(since_last, since_first),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autosave::AutoSaveConfig;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_new_tracker_is_clean() {
        let tracker = DirtyTracker::new();
        assert!(!tracker.is_dirty());
        assert!(!tracker.is_saving());
    }

    #[test]
    fn test_mark_dirty() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty();
        assert!(tracker.is_dirty());
        assert!(tracker.ms_since_last_change().is_some());
        assert!(tracker.ms_since_first_unsaved().is_some());
    }

    #[test]
    fn test_save_complete() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty();
        tracker.start_save();
        assert!(tracker.is_saving());

        tracker.save_complete();
        assert!(!tracker.is_dirty());
        assert!(!tracker.is_saving());
    }

    #[test]
    fn test_save_failed() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty();
        tracker.start_save();
        tracker.save_failed();

        assert!(tracker.is_dirty()); // Still dirty
        assert!(!tracker.is_saving());
    }

    #[test]
    fn test_should_auto_save_timing() {
        let mut tracker = DirtyTracker::new();
        let mut config = AutoSaveConfig::default();
        config.debounce_ms = 50; // Short debounce for testing

        // Not dirty - shouldn't save
        assert!(!tracker.should_auto_save(&config));

        tracker.mark_dirty();

        // Just marked dirty - shouldn't save yet (debounce)
        assert!(!tracker.should_auto_save(&config));

        // Wait for debounce
        thread::sleep(Duration::from_millis(60));

        // Now should save
        assert!(tracker.should_auto_save(&config));

        // While saving - shouldn't trigger again
        tracker.start_save();
        assert!(!tracker.should_auto_save(&config));
    }
}
