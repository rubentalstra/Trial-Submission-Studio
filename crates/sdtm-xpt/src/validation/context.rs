//! Validation context providing shared state during validation.

use crate::version::XptVersion;
use super::ValidationMode;

/// Context passed to validation rules during execution.
///
/// The context provides access to version-specific limits and
/// validation mode settings that rules need to perform their checks.
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// The XPT version being validated against.
    version: XptVersion,
    /// The validation mode (basic, FDA, custom).
    mode: ValidationMode,
}

impl ValidationContext {
    /// Create a new validation context.
    #[must_use]
    pub fn new(version: XptVersion, mode: ValidationMode) -> Self {
        Self { version, mode }
    }

    /// Get the XPT version.
    #[must_use]
    pub fn version(&self) -> XptVersion {
        self.version
    }

    /// Get the validation mode.
    #[must_use]
    pub fn mode(&self) -> ValidationMode {
        self.mode
    }

    /// Check if FDA compliance mode is enabled.
    #[must_use]
    pub fn is_fda_compliant(&self) -> bool {
        self.mode == ValidationMode::FdaCompliant
    }

    // === Version-specific limits ===

    /// Maximum length for dataset names.
    #[must_use]
    pub fn dataset_name_limit(&self) -> usize {
        self.version.dataset_name_limit()
    }

    /// Maximum length for variable names.
    #[must_use]
    pub fn variable_name_limit(&self) -> usize {
        self.version.variable_name_limit()
    }

    /// Maximum length for variable labels.
    #[must_use]
    pub fn variable_label_limit(&self) -> usize {
        self.version.variable_label_limit()
    }

    /// Maximum length for format names.
    #[must_use]
    pub fn format_name_limit(&self) -> usize {
        self.version.format_name_limit()
    }

    /// Dataset label limit (always 40 for both V5 and V8).
    #[must_use]
    pub fn dataset_label_limit(&self) -> usize {
        40
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_v5_limits() {
        let ctx = ValidationContext::new(XptVersion::V5, ValidationMode::Basic);
        assert_eq!(ctx.dataset_name_limit(), 8);
        assert_eq!(ctx.variable_name_limit(), 8);
        assert_eq!(ctx.variable_label_limit(), 40);
        assert_eq!(ctx.format_name_limit(), 8);
    }

    #[test]
    fn test_context_v8_limits() {
        let ctx = ValidationContext::new(XptVersion::V8, ValidationMode::Basic);
        assert_eq!(ctx.dataset_name_limit(), 32);
        assert_eq!(ctx.variable_name_limit(), 32);
        assert_eq!(ctx.variable_label_limit(), 256);
        assert_eq!(ctx.format_name_limit(), 32);
    }

    #[test]
    fn test_context_fda_mode() {
        let ctx = ValidationContext::new(XptVersion::V5, ValidationMode::FdaCompliant);
        assert!(ctx.is_fda_compliant());

        let ctx_basic = ValidationContext::new(XptVersion::V5, ValidationMode::Basic);
        assert!(!ctx_basic.is_fda_compliant());
    }
}
