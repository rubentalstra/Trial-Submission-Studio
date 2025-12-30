use std::collections::HashMap;

/// Mode for controlled terminology matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CtMatchingMode {
    /// Require exact or synonym matches only.
    Strict,
    /// Allow lenient matching (case-insensitive, ignoring non-alphanumeric).
    #[default]
    Lenient,
}

/// Options for normalization.
#[derive(Debug, Clone, Default)]
pub struct NormalizationOptions {
    /// Matching strictness (Strict vs Lenient)
    pub matching_mode: CtMatchingMode,
    
    /// Allow mapping invalid non-extensible values to "OTHER" if available.
    /// Default: false (Safety first).
    pub enable_other_fallback: bool,
    
    /// Allow mapping unknown-like values to "UNKNOWN" if available.
    /// Default: true.
    pub enable_unknown_fallback: bool,
    
    /// Custom value mappings (Raw -> Submission Value).
    /// Key: Raw value (normalized to uppercase/trimmed for lookup).
    /// Value: Submission Value.
    pub custom_maps: HashMap<String, String>,
}

impl NormalizationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_other_fallback(mut self, enable: bool) -> Self {
        self.enable_other_fallback = enable;
        self
    }

    pub fn with_custom_map(mut self, map: HashMap<String, String>) -> Self {
        self.custom_maps = map;
        self
    }
}
