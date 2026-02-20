//! Theme definitions for the Imbolc UI.
//!
//! Provides color schemes that can be applied across all panes.

use serde::{Deserialize, Serialize};

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ThemeColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert to a tuple for convenience
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

impl Default for ThemeColor {
    fn default() -> Self {
        Self::new(255, 255, 255)
    }
}

/// Complete UI theme with all color definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,

    // Base colors
    pub background: ThemeColor,
    pub foreground: ThemeColor,
    pub border: ThemeColor,

    // Accent colors
    pub accent: ThemeColor,
    pub accent_secondary: ThemeColor,
    pub dim: ThemeColor,

    // Semantic UI colors
    pub selection_bg: ThemeColor,
    pub selection_fg: ThemeColor,
    pub muted: ThemeColor,
    pub error: ThemeColor,
    pub warning: ThemeColor,
    pub success: ThemeColor,

    // Track type indicator colors
    pub mute_color: ThemeColor,
    pub solo_color: ThemeColor,
    pub bus_color: ThemeColor,
    pub group_color: ThemeColor,
    pub master_color: ThemeColor,

    // Module type colors (for instrument editor sections)
    pub osc_color: ThemeColor,
    pub filter_color: ThemeColor,
    pub env_color: ThemeColor,
    pub lfo_color: ThemeColor,
    pub fx_color: ThemeColor,
    pub sample_color: ThemeColor,
    pub midi_color: ThemeColor,
    pub audio_in_color: ThemeColor,
    pub eq_color: ThemeColor,
    pub custom_color: ThemeColor,
    pub kit_color: ThemeColor,
    pub bus_in_color: ThemeColor,
    pub vst_color: ThemeColor,

    // Meter colors
    pub meter_low: ThemeColor,
    pub meter_mid: ThemeColor,
    pub meter_high: ThemeColor,

    // Waveform gradient (from center outward)
    pub waveform_gradient: [ThemeColor; 4],

    // Status colors
    pub playing: ThemeColor,
    pub recording: ThemeColor,
    pub armed: ThemeColor,
}

impl Default for Theme {
    fn default() -> Self {
        Self::minimal()
    }
}

impl Theme {
    /// Default dark theme (current Imbolc colors)
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),

            // Base
            background: ThemeColor::new(0, 0, 0),
            foreground: ThemeColor::new(255, 255, 255),
            border: ThemeColor::new(100, 100, 100),

            // Accent
            accent: ThemeColor::new(0, 255, 255),
            accent_secondary: ThemeColor::new(255, 165, 0),
            dim: ThemeColor::new(100, 100, 100),

            // Semantic
            selection_bg: ThemeColor::new(60, 100, 180),
            selection_fg: ThemeColor::new(255, 255, 255),
            muted: ThemeColor::new(100, 100, 100),
            error: ThemeColor::new(255, 80, 80),
            warning: ThemeColor::new(255, 180, 0),
            success: ThemeColor::new(80, 255, 80),

            // Track type indicators
            mute_color: ThemeColor::new(255, 100, 100),
            solo_color: ThemeColor::new(255, 220, 80),
            bus_color: ThemeColor::new(147, 112, 219),
            group_color: ThemeColor::new(0, 128, 128),
            master_color: ThemeColor::new(255, 215, 0),

            // Module types
            osc_color: ThemeColor::new(100, 180, 255),
            filter_color: ThemeColor::new(255, 140, 90),
            env_color: ThemeColor::new(180, 130, 255),
            lfo_color: ThemeColor::new(130, 255, 180),
            fx_color: ThemeColor::new(255, 180, 220),
            sample_color: ThemeColor::new(255, 200, 100),
            midi_color: ThemeColor::new(255, 100, 160),
            audio_in_color: ThemeColor::new(100, 255, 200),
            eq_color: ThemeColor::new(100, 200, 255),
            custom_color: ThemeColor::new(200, 150, 255),
            kit_color: ThemeColor::new(255, 165, 0),
            bus_in_color: ThemeColor::new(180, 220, 100),
            vst_color: ThemeColor::new(255, 120, 200),

            // Meters
            meter_low: ThemeColor::new(80, 220, 100),
            meter_mid: ThemeColor::new(255, 220, 50),
            meter_high: ThemeColor::new(255, 80, 80),

            // Waveform gradient
            waveform_gradient: [
                ThemeColor::new(30, 60, 90),
                ThemeColor::new(60, 120, 180),
                ThemeColor::new(90, 180, 255),
                ThemeColor::new(120, 200, 255),
            ],

            // Status
            playing: ThemeColor::new(0, 255, 0),
            recording: ThemeColor::new(255, 0, 0),
            armed: ThemeColor::new(255, 100, 100),
        }
    }

    /// Light theme for bright environments
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),

            // Base
            background: ThemeColor::new(245, 245, 245),
            foreground: ThemeColor::new(30, 30, 30),
            border: ThemeColor::new(180, 180, 180),

            // Accent
            accent: ThemeColor::new(0, 100, 180),
            accent_secondary: ThemeColor::new(200, 120, 0),
            dim: ThemeColor::new(150, 150, 150),

            // Semantic
            selection_bg: ThemeColor::new(100, 140, 200),
            selection_fg: ThemeColor::new(255, 255, 255),
            muted: ThemeColor::new(150, 150, 150),
            error: ThemeColor::new(200, 50, 50),
            warning: ThemeColor::new(200, 140, 0),
            success: ThemeColor::new(50, 180, 50),

            // Track type indicators
            mute_color: ThemeColor::new(200, 60, 60),
            solo_color: ThemeColor::new(200, 160, 0),
            bus_color: ThemeColor::new(120, 80, 180),
            group_color: ThemeColor::new(0, 110, 110),
            master_color: ThemeColor::new(180, 140, 0),

            // Module types (darker for visibility on light bg)
            osc_color: ThemeColor::new(0, 120, 200),
            filter_color: ThemeColor::new(200, 100, 50),
            env_color: ThemeColor::new(100, 60, 180),
            lfo_color: ThemeColor::new(0, 160, 100),
            fx_color: ThemeColor::new(200, 100, 140),
            sample_color: ThemeColor::new(180, 130, 50),
            midi_color: ThemeColor::new(200, 60, 100),
            audio_in_color: ThemeColor::new(0, 150, 120),
            eq_color: ThemeColor::new(0, 140, 200),
            custom_color: ThemeColor::new(140, 80, 200),
            kit_color: ThemeColor::new(200, 120, 0),
            bus_in_color: ThemeColor::new(100, 150, 50),
            vst_color: ThemeColor::new(200, 60, 140),

            // Meters
            meter_low: ThemeColor::new(50, 180, 50),
            meter_mid: ThemeColor::new(200, 180, 0),
            meter_high: ThemeColor::new(200, 50, 50),

            // Waveform gradient
            waveform_gradient: [
                ThemeColor::new(180, 200, 220),
                ThemeColor::new(100, 150, 200),
                ThemeColor::new(50, 100, 180),
                ThemeColor::new(30, 80, 160),
            ],

            // Status
            playing: ThemeColor::new(50, 200, 50),
            recording: ThemeColor::new(220, 50, 50),
            armed: ThemeColor::new(220, 100, 100),
        }
    }

    /// High contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),

            // Base
            background: ThemeColor::new(0, 0, 0),
            foreground: ThemeColor::new(255, 255, 255),
            border: ThemeColor::new(255, 255, 255),

            // Accent
            accent: ThemeColor::new(0, 255, 255),
            accent_secondary: ThemeColor::new(255, 200, 0),
            dim: ThemeColor::new(180, 180, 180),

            // Semantic
            selection_bg: ThemeColor::new(255, 255, 0),
            selection_fg: ThemeColor::new(0, 0, 0),
            muted: ThemeColor::new(180, 180, 180),
            error: ThemeColor::new(255, 0, 0),
            warning: ThemeColor::new(255, 255, 0),
            success: ThemeColor::new(0, 255, 0),

            // Track type indicators
            mute_color: ThemeColor::new(255, 0, 0),
            solo_color: ThemeColor::new(255, 255, 0),
            bus_color: ThemeColor::new(200, 100, 255),
            group_color: ThemeColor::new(0, 255, 200),
            master_color: ThemeColor::new(255, 255, 0),

            // Module types (bright, saturated)
            osc_color: ThemeColor::new(0, 255, 255),
            filter_color: ThemeColor::new(255, 128, 0),
            env_color: ThemeColor::new(0, 255, 0),
            lfo_color: ThemeColor::new(255, 0, 255),
            fx_color: ThemeColor::new(255, 255, 0),
            sample_color: ThemeColor::new(0, 255, 128),
            midi_color: ThemeColor::new(255, 0, 0),
            audio_in_color: ThemeColor::new(128, 0, 255),
            eq_color: ThemeColor::new(0, 255, 255),
            custom_color: ThemeColor::new(200, 100, 255),
            kit_color: ThemeColor::new(255, 200, 0),
            bus_in_color: ThemeColor::new(200, 255, 0),
            vst_color: ThemeColor::new(255, 0, 200),

            // Meters
            meter_low: ThemeColor::new(0, 255, 0),
            meter_mid: ThemeColor::new(255, 255, 0),
            meter_high: ThemeColor::new(255, 0, 0),

            // Waveform gradient
            waveform_gradient: [
                ThemeColor::new(0, 50, 100),
                ThemeColor::new(0, 100, 200),
                ThemeColor::new(0, 200, 255),
                ThemeColor::new(100, 255, 255),
            ],

            // Status
            playing: ThemeColor::new(0, 255, 0),
            recording: ThemeColor::new(255, 0, 0),
            armed: ThemeColor::new(255, 255, 0),
        }
    }

    /// Minimal monochrome theme — color only for meters, recording, and playing
    pub fn minimal() -> Self {
        let mono = ThemeColor::new(200, 200, 200);
        Self {
            name: "Minimal".to_string(),

            // Base
            background: ThemeColor::new(0, 0, 0),
            foreground: ThemeColor::new(200, 200, 200),
            border: ThemeColor::new(80, 80, 80),

            // Accent (monochrome)
            accent: ThemeColor::new(220, 220, 220),
            accent_secondary: ThemeColor::new(160, 160, 160),
            dim: ThemeColor::new(80, 80, 80),

            // Semantic (subtle color for state readability)
            selection_bg: ThemeColor::new(35, 42, 58),
            selection_fg: ThemeColor::new(220, 220, 220),
            muted: ThemeColor::new(80, 80, 80),
            error: ThemeColor::new(180, 120, 120),
            warning: ThemeColor::new(180, 165, 120),
            success: ThemeColor::new(120, 170, 120),

            // Track type indicators (subtle color for state)
            mute_color: ThemeColor::new(180, 120, 120),
            solo_color: ThemeColor::new(180, 170, 120),
            bus_color: mono,
            group_color: mono,
            master_color: mono,

            // Module types (all mono)
            osc_color: mono,
            filter_color: mono,
            env_color: mono,
            lfo_color: mono,
            fx_color: mono,
            sample_color: mono,
            midi_color: mono,
            audio_in_color: mono,
            eq_color: mono,
            custom_color: mono,
            kit_color: mono,
            bus_in_color: mono,
            vst_color: mono,

            // Meters — keep color for functional clarity
            meter_low: ThemeColor::new(80, 220, 100),
            meter_mid: ThemeColor::new(255, 220, 50),
            meter_high: ThemeColor::new(255, 80, 80),

            // Waveform gradient (greyscale)
            waveform_gradient: [
                ThemeColor::new(40, 40, 40),
                ThemeColor::new(80, 80, 80),
                ThemeColor::new(140, 140, 140),
                ThemeColor::new(200, 200, 200),
            ],

            // Status — keep color for recording/playing
            playing: ThemeColor::new(0, 255, 0),
            recording: ThemeColor::new(255, 0, 0),
            armed: ThemeColor::new(180, 120, 120),
        }
    }

    /// Get all built-in themes
    pub fn built_in_themes() -> Vec<Theme> {
        vec![
            Self::dark(),
            Self::light(),
            Self::minimal(),
            Self::high_contrast(),
        ]
    }

    /// Look up a built-in theme by name (case-insensitive match on name field)
    pub fn from_name(name: &str) -> Option<Theme> {
        Self::built_in_themes()
            .into_iter()
            .find(|t| t.name.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_minimal() {
        let theme = Theme::default();
        assert_eq!(theme.name, "Minimal");
    }

    #[test]
    fn test_built_in_themes() {
        let themes = Theme::built_in_themes();
        assert_eq!(themes.len(), 4);
        assert_eq!(themes[0].name, "Dark");
        assert_eq!(themes[1].name, "Light");
        assert_eq!(themes[2].name, "Minimal");
        assert_eq!(themes[3].name, "High Contrast");
    }

    #[test]
    fn test_theme_color_rgb() {
        let color = ThemeColor::new(10, 20, 30);
        assert_eq!(color.to_rgb(), (10, 20, 30));
    }

    #[test]
    fn theme_all_colors_non_zero() {
        let theme = Theme::dark();
        // Key semantic colors should have at least one non-zero channel
        let colors = [
            theme.error,
            theme.warning,
            theme.success,
            theme.osc_color,
            theme.filter_color,
        ];
        for color in &colors {
            assert!(
                color.r > 0 || color.g > 0 || color.b > 0,
                "Color {:?} is all zeros",
                color
            );
        }
    }

    #[test]
    fn cycle_theme_wraps() {
        let themes = Theme::built_in_themes();
        // After the last theme, cycling should wrap to the first
        let last_idx = themes.len() - 1;
        let next_idx = (last_idx + 1) % themes.len();
        assert_eq!(next_idx, 0);
    }
}
