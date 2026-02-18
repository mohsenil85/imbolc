//! DAW theme bridge for rat-widget styling.
//!
//! Maps `Palette` semantic colors to `ratatui::style::Style` values suitable
//! for rat-widget configuration. Provides a centralized place to control how
//! all rat-widget components look within the DAW.

use ratatui::style::{Color as RatatuiColor, Style as RatatuiStyle};

use super::style::Palette;

/// Centralized theme for rat-widget components in the DAW.
///
/// Each method returns a `ratatui::style::Style` that can be passed directly
/// to rat-widget builder methods (`.style()`, `.focus_style()`, etc.).
pub struct DawTheme;

impl DawTheme {
    // ── Text Input ────────────────────────────────────────────────

    /// Base style for text input widgets
    pub fn text_input_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.fg))
    }

    /// Style when a text input has focus
    pub fn text_input_focus_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.fg))
    }

    /// Style for selected text in inputs
    pub fn text_input_select_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(RatatuiColor::from(p.fg))
            .bg(RatatuiColor::from(p.selection_bg))
    }

    /// Cursor style for text inputs
    pub fn text_input_cursor_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(RatatuiColor::from(p.fg))
            .bg(RatatuiColor::from(p.selection_bg))
    }

    // ── Number Input ──────────────────────────────────────────────

    /// Base style for number inputs
    pub fn number_input_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.success))
    }

    /// Focus style for number inputs
    pub fn number_input_focus_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.fg))
    }

    /// Selection style for number inputs
    pub fn number_input_select_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(RatatuiColor::from(p.fg))
            .bg(RatatuiColor::from(p.selection_bg))
    }

    // ── Checkbox ──────────────────────────────────────────────────

    /// Base style for checkboxes
    pub fn checkbox_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.fg))
    }

    /// Focus style for checkboxes
    pub fn checkbox_focus_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.accent))
    }

    // ── Slider ────────────────────────────────────────────────────

    /// Track style for sliders
    pub fn slider_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.success))
    }

    /// Focus style for sliders
    pub fn slider_focus_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.accent))
    }

    /// Knob style for sliders
    pub fn slider_knob_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.fg))
    }

    // ── Dialog / Popup ────────────────────────────────────────────

    /// Border style for dialogs
    pub fn dialog_border_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.accent))
    }

    /// Title style for dialogs
    pub fn dialog_title_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.accent))
    }

    /// Warning dialog border
    pub fn warning_border_style(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.warning))
    }

    // ── Selection ─────────────────────────────────────────────────

    /// Background for selected items in lists
    pub fn selection_bg(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().bg(RatatuiColor::from(p.selection_bg))
    }

    /// Selected item text
    pub fn selection_text(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(RatatuiColor::from(p.fg))
            .bg(RatatuiColor::from(p.selection_bg))
    }

    // ── General ───────────────────────────────────────────────────

    /// Muted/disabled text
    pub fn muted_text(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.dim))
    }

    /// Help/hint text
    pub fn help_text(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.dim))
    }

    /// Error text
    pub fn error_text(p: &Palette) -> RatatuiStyle {
        RatatuiStyle::default().fg(RatatuiColor::from(p.error))
    }
}
