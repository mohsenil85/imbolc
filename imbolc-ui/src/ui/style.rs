use imbolc_types::state::theme::{Theme, ThemeColor};
use ratatui::style::{Color as RatatuiColor, Modifier, Style as RatatuiStyle};

/// RGB color. Construct with `Color::new(r, g, b)`.
/// Colors come from `Palette` (theme-driven). Use `Color::new()` only for
/// computed/dynamic colors that can't map to a palette field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<ThemeColor> for Color {
    fn from(tc: ThemeColor) -> Self {
        Color::new(tc.r, tc.g, tc.b)
    }
}

// === Palette: pre-converted theme colors for rendering ===

/// Pre-converted palette from a `Theme`. Construct once per render frame with
/// `Palette::from(&state.session.theme)` and pass to rendering code.
pub struct Palette {
    pub bg: Color,
    pub fg: Color,
    pub border: Color,
    pub accent: Color,
    pub accent_secondary: Color,
    pub dim: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub muted: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub mute_color: Color,
    pub solo_color: Color,
    pub bus_color: Color,
    pub group_color: Color,
    pub master_color: Color,
    pub osc_color: Color,
    pub filter_color: Color,
    pub env_color: Color,
    pub lfo_color: Color,
    pub fx_color: Color,
    pub sample_color: Color,
    pub midi_color: Color,
    pub audio_in_color: Color,
    pub eq_color: Color,
    pub custom_color: Color,
    pub kit_color: Color,
    pub bus_in_color: Color,
    pub vst_color: Color,
    pub meter_low: Color,
    pub meter_mid: Color,
    pub meter_high: Color,
    pub waveform_gradient: [Color; 4],
    pub playing: Color,
    pub recording: Color,
    pub armed: Color,
}

impl From<&Theme> for Palette {
    fn from(t: &Theme) -> Self {
        Self {
            bg: t.background.into(),
            fg: t.foreground.into(),
            border: t.border.into(),
            accent: t.accent.into(),
            accent_secondary: t.accent_secondary.into(),
            dim: t.dim.into(),
            selection_bg: t.selection_bg.into(),
            selection_fg: t.selection_fg.into(),
            muted: t.muted.into(),
            error: t.error.into(),
            warning: t.warning.into(),
            success: t.success.into(),
            mute_color: t.mute_color.into(),
            solo_color: t.solo_color.into(),
            bus_color: t.bus_color.into(),
            group_color: t.group_color.into(),
            master_color: t.master_color.into(),
            osc_color: t.osc_color.into(),
            filter_color: t.filter_color.into(),
            env_color: t.env_color.into(),
            lfo_color: t.lfo_color.into(),
            fx_color: t.fx_color.into(),
            sample_color: t.sample_color.into(),
            midi_color: t.midi_color.into(),
            audio_in_color: t.audio_in_color.into(),
            eq_color: t.eq_color.into(),
            custom_color: t.custom_color.into(),
            kit_color: t.kit_color.into(),
            bus_in_color: t.bus_in_color.into(),
            vst_color: t.vst_color.into(),
            meter_low: t.meter_low.into(),
            meter_mid: t.meter_mid.into(),
            meter_high: t.meter_high.into(),
            waveform_gradient: [
                t.waveform_gradient[0].into(),
                t.waveform_gradient[1].into(),
                t.waveform_gradient[2].into(),
                t.waveform_gradient[3].into(),
            ],
            playing: t.playing.into(),
            recording: t.recording.into(),
            armed: t.armed.into(),
        }
    }
}

impl Palette {
    /// Create style with conditional selection background.
    pub fn selected_style(&self, is_selected: bool, fg: Color) -> Style {
        if is_selected {
            Style::new().fg(fg).bg(self.selection_bg)
        } else {
            Style::new().fg(fg)
        }
    }

    /// Create bold style with conditional selection background.
    pub fn selected_style_bold(&self, is_selected: bool, fg: Color) -> Style {
        if is_selected {
            Style::new().fg(fg).bg(self.selection_bg).bold()
        } else {
            Style::new().fg(fg).bold()
        }
    }
}

/// Text style with foreground, background, and attributes.
///
/// Builder methods (all const, chainable):
/// - `fg(Color)` — set foreground color
/// - `bg(Color)` — set background color
/// - `bold()` — enable bold
/// - `underline()` — enable underline
///
/// No `italic()`, `dim()`, or `reset()` methods exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub underline: bool,
}

impl Style {
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            underline: false,
        }
    }

    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub const fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    #[allow(dead_code)]
    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
}

// --- Conversions to ratatui types ---

impl From<Color> for RatatuiColor {
    fn from(c: Color) -> Self {
        RatatuiColor::Rgb(c.r, c.g, c.b)
    }
}

impl From<Style> for RatatuiStyle {
    fn from(s: Style) -> Self {
        let mut rs = RatatuiStyle::default();
        if let Some(fg) = s.fg {
            rs = rs.fg(RatatuiColor::from(fg));
        }
        if let Some(bg) = s.bg {
            rs = rs.bg(RatatuiColor::from(bg));
        }
        if s.bold {
            rs = rs.add_modifier(Modifier::BOLD);
        }
        if s.underline {
            rs = rs.add_modifier(Modifier::UNDERLINED);
        }
        rs
    }
}
