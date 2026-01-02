//! Theme and styling constants

/// Spacing constants
pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 32.0;
}

/// Common color constants not covered by egui's visuals
pub mod colors {
    use egui::Color32;

    /// Success/positive indicator color (green)
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);
}
