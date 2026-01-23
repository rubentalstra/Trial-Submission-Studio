//! Auto-save configuration.

use serde::{Deserialize, Serialize};

/// Configuration for auto-save behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveConfig {
    /// Whether auto-save is enabled.
    pub enabled: bool,

    /// Debounce delay in milliseconds.
    ///
    /// After a change, the system waits this long before saving.
    /// Additional changes reset the timer.
    pub debounce_ms: u64,

    /// Maximum delay before forcing a save.
    ///
    /// If changes keep coming, save after this many milliseconds
    /// since the first unsaved change.
    pub max_delay_ms: u64,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 2000,    // 2 seconds
            max_delay_ms: 30_000, // 30 seconds max
        }
    }
}

impl AutoSaveConfig {
    /// Create a disabled auto-save config.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Check if auto-save should trigger given the time since last change
    /// and time since first unsaved change.
    pub fn should_save(&self, since_last_change_ms: u64, since_first_unsaved_ms: u64) -> bool {
        if !self.enabled {
            return false;
        }

        // Save if debounce has passed
        if since_last_change_ms >= self.debounce_ms {
            return true;
        }

        // Force save if max delay exceeded
        if since_first_unsaved_ms >= self.max_delay_ms {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AutoSaveConfig::default();
        assert!(config.enabled);
        assert_eq!(config.debounce_ms, 2000);
    }

    #[test]
    fn test_should_save_disabled() {
        let config = AutoSaveConfig::disabled();
        assert!(!config.should_save(10000, 60000));
    }

    #[test]
    fn test_should_save_debounce() {
        let config = AutoSaveConfig::default();

        // Before debounce
        assert!(!config.should_save(1000, 1000));

        // After debounce
        assert!(config.should_save(2500, 2500));
    }

    #[test]
    fn test_should_save_max_delay() {
        let config = AutoSaveConfig::default();

        // Before max delay but within debounce (rapid changes)
        assert!(!config.should_save(500, 25000));

        // After max delay (force save despite recent changes)
        assert!(config.should_save(500, 35000));
    }
}
