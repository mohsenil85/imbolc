use imbolc_types::state::theme::{Theme, ThemeColor};
use ratatui::style::{Color as RatatuiColor, Modifier, Style as RatatuiStyle};

/// RGB color. Construct with `Color::new(r, g, b)` or use named constants
/// (e.g. `Color::WHITE`, `Color::PINK`, `Color::MIDI_COLOR`, `Color::METER_LOW`).
///
/// No `Color::rgb()` alias exists — use `Color::new()`.
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

    // Basic colors
    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const RED: Color = Color::new(255, 0, 0);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const DEEP_GREEN: Color = Color::new(0, 100, 0);
    pub const BLUE: Color = Color::new(0, 0, 255);
    pub const YELLOW: Color = Color::new(255, 255, 0);
    pub const CYAN: Color = Color::new(0, 255, 255);
    pub const MAGENTA: Color = Color::new(255, 0, 255);
    pub const GRAY: Color = Color::new(128, 128, 128);
    pub const DARK_GRAY: Color = Color::new(100, 100, 100);

    // DAW accent colors
    pub const ORANGE: Color = Color::new(255, 165, 0);
    pub const PINK: Color = Color::new(255, 105, 180);
    pub const PURPLE: Color = Color::new(147, 112, 219);
    pub const LIME: Color = Color::new(50, 205, 50);
    pub const TEAL: Color = Color::new(0, 128, 128);
    #[allow(dead_code)]
    pub const CORAL: Color = Color::new(255, 127, 80);
    pub const SKY_BLUE: Color = Color::new(135, 206, 235);
    pub const GOLD: Color = Color::new(255, 215, 0);

    // Module type colors
    #[allow(dead_code)]
    pub const MIDI_COLOR: Color = Color::new(255, 100, 160); // Magenta - MIDI/note source
    pub const OSC_COLOR: Color = Color::new(100, 180, 255); // Blue - oscillators
    pub const FILTER_COLOR: Color = Color::new(255, 140, 90); // Orange - filters
    pub const ENV_COLOR: Color = Color::new(180, 130, 255); // Purple - envelopes
    #[allow(dead_code)]
    pub const LFO_COLOR: Color = Color::new(130, 255, 180); // Mint - LFOs
    pub const FX_COLOR: Color = Color::new(255, 180, 220); // Pink - effects
    pub const EQ_COLOR: Color = Color::new(100, 200, 255); // Light blue - EQ
    #[allow(dead_code)]
    pub const OUTPUT_COLOR: Color = Color::new(255, 220, 100); // Gold - output
    pub const AUDIO_IN_COLOR: Color = Color::new(100, 255, 200); // Teal/Cyan - audio input
    pub const SAMPLE_COLOR: Color = Color::new(255, 200, 100); // Warm orange - sample
    pub const CUSTOM_COLOR: Color = Color::new(200, 150, 255); // Light purple - custom synthdef
    pub const KIT_COLOR: Color = Color::new(255, 165, 0); // Orange - kit
    pub const BUS_IN_COLOR: Color = Color::new(180, 220, 100); // Yellow-green - bus input
    pub const VST_COLOR: Color = Color::new(255, 120, 200); // Hot pink - VST plugins

    // Port type colors
    #[allow(dead_code)]
    pub const AUDIO_PORT: Color = Color::new(80, 200, 255); // Cyan - audio
    #[allow(dead_code)]
    pub const CONTROL_PORT: Color = Color::new(100, 255, 150); // Green - control
    #[allow(dead_code)]
    pub const GATE_PORT: Color = Color::new(255, 230, 80); // Yellow - gate

    // Meter colors
    pub const METER_LOW: Color = Color::new(80, 220, 100); // Green
    pub const METER_MID: Color = Color::new(255, 220, 50); // Yellow
    pub const METER_HIGH: Color = Color::new(255, 80, 80); // Red

    // UI colors
    pub const SELECTION_BG: Color = Color::new(60, 100, 180); // Selection highlight
    pub const MUTE_COLOR: Color = Color::new(255, 100, 100); // Muted state
    pub const SOLO_COLOR: Color = Color::new(255, 220, 80); // Solo state
}

impl From<ThemeColor> for Color {
    fn from(tc: ThemeColor) -> Self {
        Color::new(tc.r, tc.g, tc.b)
    }
}

// === Palette: pre-converted theme colors for rendering ===

/// Pre-converted palette from a `Theme`. Construct once per render frame with
/// `Palette::from(&state.session.theme)` and pass to rendering code.
#[allow(dead_code)]
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
