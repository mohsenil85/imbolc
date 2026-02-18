use std::any::Any;

use crate::state::AppState;
use crate::ui::action_id::{ActionId, ModeActionId, WaveformActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::widgets::TextInput;
use crate::ui::{
    Action, Color, FileSelectAction, InputEvent, Keymap, Pane, Rect, RenderBuf, SessionAction,
    Style,
};

/// Waveform display characters (8 levels) - used for spectrum/meters
const WAVEFORM_CHARS: [char; 8] = [
    '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}',
];

/// Braille dot pattern offsets (2 columns x 4 rows)
/// Column 0: bits 0,1,2,6  Column 1: bits 3,4,5,7
const BRAILLE_DOT_OFFSETS: [[u8; 4]; 2] = [
    [0, 1, 2, 6], // left column (x=0): rows 0,1,2,3
    [3, 4, 5, 7], // right column (x=1): rows 0,1,2,3
];

/// Spectrum band labels
const SPECTRUM_LABELS: [&str; 7] = ["60", "150", "400", "1k", "2.5k", "6k", "15k"];

/// Fixed waveform palette from center to edge.
/// Center: green -> yellow -> orange -> red at extremes.
const WAVEFORM_PALETTE: [Color; 4] = [
    Color::new(70, 210, 90),
    Color::new(245, 215, 70),
    Color::new(255, 155, 45),
    Color::new(235, 70, 70),
];

/// Lightweight shared layout for centered visual panes.
#[derive(Debug, Clone, Copy)]
struct PlotLayout {
    rect: Rect,
    grid_x: u16,
    grid_y: u16,
    grid_width: u16,
    grid_height: u16,
    status_y: u16,
}

impl PlotLayout {
    fn from_rect(rect: Rect, header_height: u16, footer_height: u16) -> Option<Self> {
        let grid_x = rect.x.saturating_add(1);
        let grid_y = rect.y.saturating_add(header_height);
        let grid_width = rect.width.saturating_sub(2);
        let grid_height = rect.height.saturating_sub(
            header_height
                .saturating_add(footer_height)
                .saturating_add(1),
        );

        if grid_width == 0 || grid_height == 0 {
            return None;
        }

        Some(Self {
            rect,
            grid_x,
            grid_y,
            grid_width,
            grid_height,
            status_y: grid_y.saturating_add(grid_height),
        })
    }

    fn status_rect(self) -> Rect {
        Rect::new(
            self.rect.x.saturating_add(1),
            self.status_y,
            self.rect.width.saturating_sub(2),
            1,
        )
    }
}

/// Dot raster backing a braille plot (2x4 dots per character cell).
struct DotRaster {
    dot_width: usize,
    dot_height: usize,
    dots: Vec<bool>,
}

impl DotRaster {
    fn new(char_width: u16, char_height: u16) -> Self {
        let dot_width = char_width as usize * 2;
        let dot_height = char_height as usize * 4;
        Self {
            dot_width,
            dot_height,
            dots: vec![false; dot_width.saturating_mul(dot_height)],
        }
    }

    fn dot_width(&self) -> usize {
        self.dot_width
    }

    fn dot_height(&self) -> usize {
        self.dot_height
    }

    fn set(&mut self, x: usize, y: usize) {
        if x >= self.dot_width || y >= self.dot_height {
            return;
        }
        let idx = y * self.dot_width + x;
        if let Some(cell) = self.dots.get_mut(idx) {
            *cell = true;
        }
    }

    fn fill_vertical(&mut self, x: usize, y0: usize, y1: usize) {
        if x >= self.dot_width || self.dot_height == 0 {
            return;
        }
        let start = y0.min(y1).min(self.dot_height - 1);
        let end = y0.max(y1).min(self.dot_height - 1);
        for y in start..=end {
            self.set(x, y);
        }
    }

    fn is_set(&self, x: usize, y: usize) -> bool {
        if x >= self.dot_width || y >= self.dot_height {
            return false;
        }
        self.dots[y * self.dot_width + x]
    }

    fn char_pattern(&self, char_col: usize, char_row: usize) -> u8 {
        let mut pattern: u8 = 0;
        for dx in 0..2 {
            for dy in 0..4 {
                let dot_x = char_col * 2 + dx;
                let dot_y = char_row * 4 + dy;
                if self.is_set(dot_x, dot_y) {
                    pattern |= 1 << BRAILLE_DOT_OFFSETS[dx][dy];
                }
            }
        }
        pattern
    }
}

/// Convert a braille bit pattern to a Unicode braille character.
fn braille_char(pattern: u8) -> char {
    char::from_u32(0x2800 + pattern as u32).unwrap_or(' ')
}

/// Resolve waveform color based on distance from center.
fn waveform_gradient_color(frac_from_center: f32) -> Color {
    let frac = frac_from_center.clamp(0.0, 1.0);
    let idx = ((frac * WAVEFORM_PALETTE.len() as f32) as usize).min(WAVEFORM_PALETTE.len() - 1);
    WAVEFORM_PALETTE[idx]
}

/// Color a meter bar by its normalized height fraction (0.0=bottom, 1.0=top).
fn meter_color(frac: f32) -> Color {
    if frac > 0.85 {
        Color::new(235, 70, 70) // red
    } else if frac > 0.65 {
        Color::new(255, 155, 45) // orange
    } else if frac > 0.4 {
        Color::new(245, 215, 70) // yellow
    } else {
        Color::new(70, 210, 90) // green
    }
}

/// Convert linear amplitude to dB
fn amp_to_db(amp: f32) -> f32 {
    if amp <= 0.0 {
        -96.0
    } else {
        20.0 * amp.log10()
    }
}

/// Resample any waveform into `columns` bins using peak-abs over each column window.
fn resample_peak_abs(samples: &[f32], columns: usize) -> Vec<f32> {
    if columns == 0 {
        return Vec::new();
    }
    if samples.is_empty() {
        return vec![0.0; columns];
    }

    let len = samples.len();
    let mut out = Vec::with_capacity(columns);
    for col in 0..columns {
        let start = col * len / columns;
        let mut end = (col + 1) * len / columns;
        if end <= start {
            end = (start + 1).min(len);
        }

        let mut peak = 0.0_f32;
        for &sample in &samples[start..end] {
            peak = peak.max(sample.abs());
        }
        out.push(peak.clamp(0.0, 1.0));
    }

    out
}

/// Resample to `columns` points with linear interpolation (preserves waveform continuity).
fn resample_linear(samples: &[f32], columns: usize) -> Vec<f32> {
    if columns == 0 {
        return Vec::new();
    }
    if samples.is_empty() {
        return vec![0.0; columns];
    }
    if samples.len() == 1 {
        return vec![samples[0].clamp(-1.0, 1.0); columns];
    }

    let last_idx = samples.len() - 1;
    let denom = columns.saturating_sub(1).max(1) as f32;

    (0..columns)
        .map(|i| {
            let pos = i as f32 * last_idx as f32 / denom;
            let left = pos.floor() as usize;
            let right = (left + 1).min(last_idx);
            let t = pos - left as f32;
            let v = samples[left] * (1.0 - t) + samples[right] * t;
            v.clamp(-1.0, 1.0)
        })
        .collect()
}

/// Display mode for the waveform pane
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaveformMode {
    Waveform,
    Spectrum,
    Oscilloscope,
    LufsMeter,
}

#[allow(dead_code)]
impl WaveformMode {
    fn next(self) -> Self {
        match self {
            WaveformMode::Waveform => WaveformMode::Spectrum,
            WaveformMode::Spectrum => WaveformMode::Oscilloscope,
            WaveformMode::Oscilloscope => WaveformMode::LufsMeter,
            WaveformMode::LufsMeter => WaveformMode::Waveform,
        }
    }

    fn name(self) -> &'static str {
        match self {
            WaveformMode::Waveform => "Waveform",
            WaveformMode::Spectrum => "Spectrum",
            WaveformMode::Oscilloscope => "Oscilloscope",
            WaveformMode::LufsMeter => "Level Meter",
        }
    }
}

pub struct WaveformPane {
    keymap: Keymap,
    /// Live waveform from audio input
    pub audio_in_waveform: Option<Vec<f32>>,
    /// Exponential smoothing cache for live waveform envelope.
    live_envelope_ema: Option<Vec<f32>>,
    /// Current display mode
    mode: WaveformMode,
    /// Wave editor cursor (index into recorded waveform peaks).
    cursor_index: usize,
    /// Left edge of the visible waveform window.
    view_offset: usize,
    /// Last rendered window width (in waveform samples).
    last_window_width: usize,
    /// Persistent selection (inclusive start/end).
    selection: Option<(usize, usize)>,
    /// Selection mode toggle (Ctrl+Space).
    select_mode: bool,
    /// Anchor index while selection mode is active.
    select_anchor: Option<usize>,
    /// Text input for clip rename.
    edit_input: TextInput,
    /// Whether clip rename is active.
    editing_name: bool,
}

impl WaveformPane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            audio_in_waveform: None,
            live_envelope_ema: None,
            mode: WaveformMode::Waveform,
            cursor_index: 0,
            view_offset: 0,
            last_window_width: 1,
            selection: None,
            select_mode: false,
            select_anchor: None,
            edit_input: TextInput::new(""),
            editing_name: false,
        }
    }
}

impl Default for WaveformPane {
    fn default() -> Self {
        Self::new(Keymap::new())
    }
}

impl WaveformPane {
    pub fn is_editing(&self) -> bool {
        self.editing_name
    }

    pub fn playback_start_norm(&self, state: &AppState) -> f32 {
        let len = state
            .recorded_waveform_peaks
            .as_ref()
            .map(|w| w.len())
            .unwrap_or(0);
        if len == 0 {
            return 0.0;
        }
        let idx = self.cursor_index.min(len.saturating_sub(1));
        (idx as f32 / len as f32).clamp(0.0, 0.999_9)
    }

    fn current_clip_name(&self, state: &AppState) -> String {
        if let Some(name) = state.recording.last_recording_name.as_ref() {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Some(path) = state.recording.last_recording_path.as_ref() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if !stem.trim().is_empty() {
                    return stem.trim().to_string();
                }
            }
        }
        "Clip".to_string()
    }

    fn smooth_live_envelope(&mut self, next: Vec<f32>) -> Vec<f32> {
        if let Some(prev) = &mut self.live_envelope_ema {
            if prev.len() == next.len() {
                for (p, n) in prev.iter_mut().zip(next.iter().copied()) {
                    // Fast attack, slow release to reduce jitter while staying responsive.
                    let alpha = if n > *p { 0.38 } else { 0.12 };
                    *p += (n - *p) * alpha;
                }
                return prev.clone();
            }
        }

        self.live_envelope_ema = Some(next.clone());
        next
    }

    fn draw_status(&self, layout: PlotLayout, buf: &mut RenderBuf, text: &str, color: Color) {
        buf.draw_line(layout.status_rect(), &[(text, Style::new().fg(color))]);
    }

    fn draw_center_line(&self, layout: PlotLayout, buf: &mut RenderBuf) {
        let style = Style::new().fg(Color::DARK_GRAY);
        let center_char_row = layout.grid_y.saturating_add(layout.grid_height / 2);
        for x in 0..layout.grid_width {
            buf.set_cell(layout.grid_x + x, center_char_row, '\u{2500}', style);
        }
    }

    fn draw_recording_progress(
        &self,
        layout: PlotLayout,
        buf: &mut RenderBuf,
        recording_secs: u64,
    ) {
        if layout.grid_width == 0 || layout.grid_height == 0 {
            return;
        }
        // Open-ended recording has no true endpoint, so show progress within a fixed 60s window.
        let secs = recording_secs as f32;
        let fill = (secs / 60.0).clamp(0.0, 1.0);
        let fill_width = (fill * layout.grid_width as f32).round() as u16;
        let y = layout.grid_y + layout.grid_height.saturating_sub(1);
        for i in 0..layout.grid_width {
            let x = layout.grid_x + i;
            if i < fill_width {
                let color = meter_color(fill);
                buf.set_cell(x, y, WAVEFORM_CHARS[7], Style::new().fg(color));
            } else {
                buf.set_cell(x, y, WAVEFORM_CHARS[0], Style::new().fg(Color::DARK_GRAY));
            }
        }
    }

    fn draw_vertical_marker(
        &self,
        layout: PlotLayout,
        buf: &mut RenderBuf,
        progress: f32,
        color: Color,
        ch: char,
    ) {
        if layout.grid_width == 0 || layout.grid_height == 0 {
            return;
        }
        let frac = progress.clamp(0.0, 1.0);
        let x_offset = ((layout.grid_width.saturating_sub(1) as f32) * frac).round() as u16;
        let x = layout.grid_x + x_offset;
        let style = Style::new().fg(color).bold();
        for y in 0..layout.grid_height {
            buf.set_cell(x, layout.grid_y + y, ch, style);
        }
    }

    fn draw_playback_playhead(&self, layout: PlotLayout, buf: &mut RenderBuf, progress: f32) {
        self.draw_vertical_marker(layout, buf, progress, Color::new(235, 60, 60), '\u{2502}');
    }

    fn draw_editor_cursor(&self, layout: PlotLayout, buf: &mut RenderBuf, progress: f32) {
        self.draw_vertical_marker(layout, buf, progress, Color::new(120, 220, 255), '\u{2502}');
    }

    fn draw_selection_overlay(
        &self,
        layout: PlotLayout,
        buf: &mut RenderBuf,
        start_progress: f32,
        end_progress: f32,
    ) {
        if layout.grid_width == 0 || layout.grid_height == 0 {
            return;
        }
        let a = start_progress.clamp(0.0, 1.0);
        let b = end_progress.clamp(0.0, 1.0);
        let left = ((layout.grid_width.saturating_sub(1) as f32) * a).round() as u16;
        let right = ((layout.grid_width.saturating_sub(1) as f32) * b).round() as u16;
        let (x1, x2) = if left <= right {
            (layout.grid_x + left, layout.grid_x + right)
        } else {
            (layout.grid_x + right, layout.grid_x + left)
        };
        let style = Style::new().fg(Color::new(255, 210, 70)).bold();
        for y in 0..layout.grid_height {
            let yy = layout.grid_y + y;
            buf.set_cell(x1, yy, '\u{2502}', style);
            buf.set_cell(x2, yy, '\u{2502}', style);
        }
        let top_y = layout.grid_y;
        let bottom_y = layout.grid_y + layout.grid_height.saturating_sub(1);
        for x in x1..=x2 {
            buf.set_cell(x, top_y, '\u{2500}', style);
            buf.set_cell(x, bottom_y, '\u{2500}', style);
        }
    }

    fn clamp_editor_state(&mut self, len: usize) {
        if len == 0 {
            self.cursor_index = 0;
            self.view_offset = 0;
            self.selection = None;
            self.select_anchor = None;
            self.select_mode = false;
            return;
        }
        self.cursor_index = self.cursor_index.min(len - 1);
        self.view_offset = self.view_offset.min(len - 1);
        if let Some((start, end)) = self.selection {
            let s = start.min(len - 1);
            let e = end.min(len - 1);
            self.selection = Some((s.min(e), s.max(e)));
        }
        if let Some(anchor) = self.select_anchor {
            self.select_anchor = Some(anchor.min(len - 1));
        }
    }

    fn ensure_cursor_visible(&mut self, len: usize) {
        if len == 0 {
            self.view_offset = 0;
            return;
        }
        let window = self.last_window_width.max(1);
        let max_start = len.saturating_sub(window);
        self.view_offset = self.view_offset.min(max_start);
        if self.cursor_index < self.view_offset {
            self.view_offset = self.cursor_index;
        } else if self.cursor_index >= self.view_offset + window {
            self.view_offset = self.cursor_index + 1 - window;
        }
        self.view_offset = self.view_offset.min(max_start);
    }

    fn active_selection(&self) -> Option<(usize, usize)> {
        if self.select_mode {
            self.select_anchor.map(|anchor| {
                if anchor <= self.cursor_index {
                    (anchor, self.cursor_index)
                } else {
                    (self.cursor_index, anchor)
                }
            })
        } else {
            self.selection
        }
    }

    fn selection_to_norm(&self, selection: (usize, usize), len: usize) -> Option<(f32, f32)> {
        if len == 0 {
            return None;
        }
        let (start, end_inclusive) = selection;
        let s = start.min(len - 1);
        let e = end_inclusive.min(len - 1);
        let (a, b) = if s <= e { (s, e) } else { (e, s) };
        let start_norm = a as f32 / len as f32;
        let end_norm = (b.saturating_add(1)).min(len) as f32 / len as f32;
        Some((start_norm, end_norm))
    }

    fn selection_or_full_norm(&self, len: usize) -> Option<(f32, f32)> {
        if len == 0 {
            return None;
        }
        if let Some(sel) = self.active_selection() {
            self.selection_to_norm(sel, len)
        } else {
            Some((0.0, 1.0))
        }
    }

    fn progress_for_index(
        &self,
        index: usize,
        visible_start: usize,
        visible_len: usize,
    ) -> Option<f32> {
        if visible_len == 0 {
            return None;
        }
        if index < visible_start || index >= visible_start + visible_len {
            return None;
        }
        if visible_len <= 1 {
            return Some(0.0);
        }
        let rel = index - visible_start;
        let denom = (visible_len - 1) as f32;
        Some(rel as f32 / denom)
    }

    fn first_sound_index(samples: &[f32], threshold: f32) -> Option<usize> {
        samples.iter().position(|&v| v.abs() >= threshold)
    }

    fn peak_indices(samples: &[f32], threshold: f32) -> Vec<usize> {
        let len = samples.len();
        if len < 3 {
            return Vec::new();
        }
        let mut peaks = Vec::new();
        for i in 1..(len - 1) {
            let v = samples[i].abs();
            if v < threshold {
                continue;
            }
            let prev = samples[i - 1].abs();
            let next = samples[i + 1].abs();
            if v >= prev && v > next {
                peaks.push(i);
            }
        }
        peaks
    }

    fn non_silent_bounds(samples: &[f32], threshold: f32) -> Option<(usize, usize)> {
        let start = samples.iter().position(|&v| v.abs() >= threshold)?;
        let end = samples.iter().rposition(|&v| v.abs() >= threshold)?;
        Some((start.min(end), start.max(end)))
    }

    fn move_cursor_left(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        self.clamp_editor_state(len);
        self.cursor_index = self.cursor_index.saturating_sub(1);
        self.ensure_cursor_visible(len);
    }

    fn move_cursor_right(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        self.clamp_editor_state(len);
        self.cursor_index = (self.cursor_index + 1).min(len - 1);
        self.ensure_cursor_visible(len);
    }

    fn jump_cursor_right(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        self.clamp_editor_state(samples.len());
        const THRESHOLD: f32 = 0.02;
        let Some(first_sound) = Self::first_sound_index(samples, THRESHOLD) else {
            self.cursor_index = samples.len() - 1;
            self.ensure_cursor_visible(samples.len());
            return;
        };
        if self.cursor_index < first_sound {
            self.cursor_index = first_sound;
            self.ensure_cursor_visible(samples.len());
            return;
        }
        let peaks = Self::peak_indices(samples, THRESHOLD);
        if let Some(next_peak) = peaks.into_iter().find(|&idx| idx > self.cursor_index) {
            self.cursor_index = next_peak;
        } else {
            self.cursor_index = samples.len() - 1;
        }
        self.ensure_cursor_visible(samples.len());
    }

    fn jump_cursor_left(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        self.clamp_editor_state(samples.len());
        const THRESHOLD: f32 = 0.02;
        let Some(first_sound) = Self::first_sound_index(samples, THRESHOLD) else {
            self.cursor_index = 0;
            self.ensure_cursor_visible(samples.len());
            return;
        };
        if self.cursor_index <= first_sound {
            self.cursor_index = 0;
            self.ensure_cursor_visible(samples.len());
            return;
        }
        let peaks = Self::peak_indices(samples, THRESHOLD);
        if let Some(prev_peak) = peaks.into_iter().rev().find(|&idx| idx < self.cursor_index) {
            self.cursor_index = prev_peak.max(first_sound);
        } else {
            self.cursor_index = first_sound;
        }
        self.ensure_cursor_visible(samples.len());
    }

    fn draw_braille_raster<F>(
        &self,
        layout: PlotLayout,
        raster: &DotRaster,
        buf: &mut RenderBuf,
        mut color_for_row: F,
    ) where
        F: FnMut(usize, usize) -> Color,
    {
        let rows = layout.grid_height as usize;
        for char_row in 0..layout.grid_height as usize {
            for char_col in 0..layout.grid_width as usize {
                let pattern = raster.char_pattern(char_col, char_row);
                if pattern == 0 {
                    continue;
                }
                let color = color_for_row(char_row, rows.max(1));
                buf.set_cell(
                    layout.grid_x + char_col as u16,
                    layout.grid_y + char_row as u16,
                    braille_char(pattern),
                    Style::new().fg(color),
                );
            }
        }
    }

    fn render_waveform(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let is_recorded = state.recorded_waveform_peaks.is_some();
        let waveform: Vec<f32> = state
            .recorded_waveform_peaks
            .clone()
            .or_else(|| self.audio_in_waveform.clone())
            .unwrap_or_default();

        let rect = center_rect(area, 97, 29);
        let title = if is_recorded {
            let clip_name = self.current_clip_name(state);
            if self.select_mode {
                format!(" Recorded Waveform: {} [SELECT] ", clip_name)
            } else if self.editing_name {
                format!(" Recorded Waveform: {} [RENAME] ", clip_name)
            } else {
                format!(" Recorded Waveform: {} ", clip_name)
            }
        } else if let Some(inst) = state.tracks.selected_track() {
            format!(" Audio Input: {} ", inst.name)
        } else {
            " Audio Input ".to_string()
        };

        let border_color = if is_recorded && self.editing_name {
            Color::LIME
        } else if is_recorded && self.select_mode {
            Color::new(255, 210, 70)
        } else {
            Color::AUDIO_IN_COLOR
        };
        self.render_border(rect, buf, &title, border_color);
        self.render_header(rect, buf, state, WaveformMode::Waveform.name());
        if is_recorded && (self.select_mode || self.editing_name) {
            let badge = if self.editing_name {
                " [RENAME MODE] "
            } else {
                " [SELECT MODE] "
            };
            let badge_w = badge.chars().count() as u16;
            if rect.width > badge_w + 2 {
                let x = rect.x + rect.width - badge_w - 1;
                buf.draw_line(
                    Rect::new(x, rect.y + 1, badge_w, 1),
                    &[(
                        badge,
                        Style::new()
                            .fg(Color::new(20, 20, 20))
                            .bg(if self.editing_name {
                                Color::LIME
                            } else {
                                Color::new(255, 210, 70)
                            })
                            .bold(),
                    )],
                );
            }
        }

        let Some(layout) = PlotLayout::from_rect(rect, 2, 2) else {
            return;
        };

        self.draw_center_line(layout, buf);

        if waveform.is_empty() {
            self.live_envelope_ema = None;
            self.draw_status(layout, buf, "Samples: 0  [Tab: cycle mode]", Color::GRAY);
            return;
        }

        let mut raster = DotRaster::new(layout.grid_width, layout.grid_height);
        if raster.dot_width() == 0 || raster.dot_height() == 0 {
            self.draw_status(layout, buf, "Terminal too small", Color::GRAY);
            return;
        }

        const LIVE_BUFFER_SIZE: usize = 100;
        let (display_source, visible_start, visible_len) = if is_recorded {
            let total_len = waveform.len();
            self.clamp_editor_state(total_len);
            self.last_window_width = raster.dot_width().max(1);
            self.ensure_cursor_visible(total_len);
            let max_start = total_len.saturating_sub(self.last_window_width);
            self.view_offset = self.view_offset.min(max_start);
            let end = (self.view_offset + self.last_window_width).min(total_len);
            let visible_len = end.saturating_sub(self.view_offset);
            let mut slice = waveform[self.view_offset..end].to_vec();
            if slice.len() < self.last_window_width {
                slice.resize(self.last_window_width, 0.0);
            }
            (slice, self.view_offset, visible_len.max(1))
        } else if waveform.len() >= LIVE_BUFFER_SIZE {
            (
                waveform[waveform.len() - LIVE_BUFFER_SIZE..].to_vec(),
                0,
                LIVE_BUFFER_SIZE,
            )
        } else {
            // Keep mapping stable during startup by padding to fixed live window size.
            let mut padded = vec![0.0; LIVE_BUFFER_SIZE];
            let start = LIVE_BUFFER_SIZE - waveform.len();
            padded[start..].copy_from_slice(&waveform);
            (padded, 0, LIVE_BUFFER_SIZE)
        };

        let center_dot_y = raster.dot_height() / 2;

        let envelope = if is_recorded {
            self.live_envelope_ema = None;
            resample_peak_abs(&display_source, raster.dot_width())
        } else {
            let raw_envelope = resample_peak_abs(&display_source, raster.dot_width());
            self.smooth_live_envelope(raw_envelope)
        };

        for (idx, amplitude) in envelope.iter().copied().enumerate() {
            let extent = (amplitude * center_dot_y as f32).round() as usize;
            if extent == 0 {
                continue;
            }
            let top = center_dot_y.saturating_sub(extent);
            let bottom = (center_dot_y + extent).min(raster.dot_height() - 1);
            raster.fill_vertical(idx, top, bottom);
        }

        self.draw_braille_raster(layout, &raster, buf, |row, rows| {
            let center = rows as f32 / 2.0;
            let dist = (row as f32 + 0.5 - center).abs();
            let frac = if center <= 0.0 { 0.0 } else { dist / center };
            waveform_gradient_color(frac)
        });

        if is_recorded {
            if let Some((start, end)) = self.active_selection() {
                if let (Some(a), Some(b)) = (
                    self.progress_for_index(start, visible_start, visible_len),
                    self.progress_for_index(end, visible_start, visible_len),
                ) {
                    self.draw_selection_overlay(layout, buf, a, b);
                }
            }
            if let Some(cursor_progress) =
                self.progress_for_index(self.cursor_index, visible_start, visible_len)
            {
                self.draw_editor_cursor(layout, buf, cursor_progress);
            }
        }

        if is_recorded {
            if let (Some(started), Some(duration_secs)) = (
                state.recording.preview_started_at,
                state.recording.preview_duration_secs,
            ) {
                if duration_secs > 0.0 {
                    let start_norm = state
                        .recording
                        .preview_start_norm
                        .unwrap_or(0.0)
                        .clamp(0.0, 1.0);
                    let elapsed = started.elapsed().as_secs_f32();
                    let local_progress = (elapsed / duration_secs).clamp(0.0, 1.0);
                    let progress = start_norm + local_progress * (1.0 - start_norm);
                    if state.recording.preview_node_id.is_some() || local_progress < 1.0 {
                        self.draw_playback_playhead(layout, buf, progress);
                    }
                }
            }
        }

        if state.recording.recording {
            self.draw_recording_progress(layout, buf, state.recording.recording_secs);
        }

        let source = if is_recorded { "recorded" } else { "live" };
        let live_samples_width = LIVE_BUFFER_SIZE.to_string().len();
        let status = if !is_recorded && waveform.len() < LIVE_BUFFER_SIZE {
            format!(
                "Source: {}  Samples: {} (stabilizing)  REC: {}s  [Tab: cycle mode]",
                source,
                format_args!(
                    "{:0width$}/{:0width$}",
                    waveform.len(),
                    LIVE_BUFFER_SIZE,
                    width = live_samples_width
                ),
                state.recording.recording_secs
            )
        } else if is_recorded {
            let duration_label = state
                .recording
                .last_recording_duration_secs
                .map(|secs| format!("{:.1}s", secs))
                .unwrap_or_else(|| "n/a".to_string());
            let select_state = if self.select_mode { "ACTIVE" } else { "OFF" };
            let sel_label = if let Some((s, e)) = self.active_selection() {
                format!("{}-{}", s, e)
            } else {
                "-".to_string()
            };
            format!(
                "Source:{}  Samples:{}  Len:{}  Cursor:{}  Sel:{}  Select:{}  [n rename, p pitched, s stretch, c chopper, Ctrl+Space select, Del delete, t trim]",
                source,
                waveform.len(),
                duration_label,
                self.cursor_index,
                sel_label,
                select_state
            )
        } else {
            format!(
                "Source: {}  Samples: {}  REC: {}s  [Tab: cycle mode]",
                source,
                waveform.len(),
                state.recording.recording_secs
            )
        };
        if self.editing_name {
            let input_rect = layout.status_rect();
            self.edit_input
                .render_buf(buf.raw_buf(), input_rect.x, input_rect.y, input_rect.width);
            if let Some((cx, cy)) = self.edit_input.screen_cursor() {
                buf.set_cursor_position(cx, cy);
            }
        } else {
            self.draw_status(layout, buf, &status, Color::GRAY);
        }
    }

    fn render_spectrum(&self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let rect = center_rect(area, 97, 29);
        self.render_border(rect, buf, " Spectrum Analyzer ", Color::METER_LOW);
        self.render_header(rect, buf, state, WaveformMode::Spectrum.name());

        let Some(layout) = PlotLayout::from_rect(rect, 2, 3) else {
            return;
        };

        let bands = &state.audio.visualization.spectrum_bands;
        let num_bands = bands.len();
        if num_bands == 0 {
            self.draw_status(layout, buf, "No spectrum data", Color::DARK_GRAY);
            return;
        }

        let total_gaps = num_bands.saturating_sub(1) as u16;
        let usable_width = layout.grid_width.saturating_sub(total_gaps);
        let base_width = (usable_width / num_bands as u16).max(1);
        let extra = usable_width % num_bands as u16;

        let mut cursor_x = layout.grid_x;
        for (i, &amp) in bands.iter().enumerate() {
            if cursor_x >= layout.grid_x.saturating_add(layout.grid_width) {
                break;
            }

            let width_bonus = if (i as u16) < extra { 1 } else { 0 };
            let bar_width = base_width + width_bonus;
            let bar_height = (amp.clamp(0.0, 1.0) * layout.grid_height as f32).round() as u16;

            for dy in 0..bar_height.min(layout.grid_height) {
                let y = layout.grid_y + layout.grid_height - 1 - dy;
                let frac = (dy + 1) as f32 / layout.grid_height as f32;
                let color = meter_color(frac);
                let style = Style::new().fg(color);

                for bx in 0..bar_width {
                    let x = cursor_x + bx;
                    if x < layout.grid_x + layout.grid_width {
                        buf.set_cell(x, y, WAVEFORM_CHARS[7], style);
                    }
                }
            }

            let label = SPECTRUM_LABELS[i];
            let label_y = layout.grid_y + layout.grid_height;
            let label_x = cursor_x + bar_width / 2;
            let label_start = label_x.saturating_sub(label.len() as u16 / 2);
            buf.draw_line(
                Rect::new(label_start, label_y, label.len() as u16 + 1, 1),
                &[(label, Style::new().fg(Color::GRAY))],
            );

            let db = amp_to_db(amp);
            let db_str = if db <= -60.0 {
                "-inf".to_string()
            } else {
                format!("{:.0}", db)
            };
            let db_y = label_y.saturating_add(1);
            let db_start = label_x.saturating_sub(db_str.len() as u16 / 2);
            if db_y < rect.y + rect.height - 1 {
                buf.draw_line(
                    Rect::new(db_start, db_y, db_str.len() as u16 + 1, 1),
                    &[(&db_str, Style::new().fg(Color::DARK_GRAY))],
                );
            }

            cursor_x = cursor_x.saturating_add(bar_width);
            if i + 1 < num_bands {
                cursor_x = cursor_x.saturating_add(1);
            }
        }

        let status_y = rect.y + rect.height - 2;
        buf.draw_line(
            Rect::new(rect.x + 1, status_y, rect.width.saturating_sub(2), 1),
            &[("[Tab: cycle mode]", Style::new().fg(Color::DARK_GRAY))],
        );
    }

    fn render_oscilloscope(&self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let rect = center_rect(area, 97, 29);
        self.render_border(rect, buf, " Oscilloscope ", Color::MIDI_COLOR);
        self.render_header(rect, buf, state, WaveformMode::Oscilloscope.name());

        let Some(layout) = PlotLayout::from_rect(rect, 2, 2) else {
            return;
        };

        self.draw_center_line(layout, buf);

        // +1/-1 labels
        let marker_style = Style::new().fg(Color::DARK_GRAY);
        buf.draw_line(
            Rect::new(layout.grid_x, layout.grid_y, 2, 1),
            &[("+1", marker_style)],
        );
        buf.draw_line(
            Rect::new(
                layout.grid_x,
                layout.grid_y + layout.grid_height.saturating_sub(1),
                2,
                1,
            ),
            &[("-1", marker_style)],
        );

        let scope = &state.audio.visualization.scope_buffer;
        if scope.is_empty() {
            self.draw_status(layout, buf, "Samples: 0  [Tab: cycle mode]", Color::GRAY);
            return;
        }

        let mut raster = DotRaster::new(layout.grid_width, layout.grid_height);
        if raster.dot_width() == 0 || raster.dot_height() == 0 {
            self.draw_status(layout, buf, "Terminal too small", Color::GRAY);
            return;
        }

        let scope_vec: Vec<f32> = scope.iter().copied().collect();
        let trace = resample_linear(&scope_vec, raster.dot_width());

        let mut prev_dot_y: Option<usize> = None;
        for (dot_x, sample) in trace.iter().copied().enumerate() {
            let normalized = (1.0 - sample) * 0.5; // +1 -> 0(top), -1 -> 1(bottom)
            let dot_y = (normalized * (raster.dot_height().saturating_sub(1)) as f32)
                .round()
                .clamp(0.0, (raster.dot_height().saturating_sub(1)) as f32)
                as usize;

            raster.set(dot_x, dot_y);
            if let Some(prev_y) = prev_dot_y {
                raster.fill_vertical(dot_x, prev_y, dot_y);
            }
            prev_dot_y = Some(dot_y);
        }

        self.draw_braille_raster(layout, &raster, buf, |_, _| Color::new(80, 220, 120));

        let status = format!("Samples: {}  [Tab: cycle mode]", scope.len());
        self.draw_status(layout, buf, &status, Color::GRAY);
    }

    fn render_lufs_meter(&self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let rect = center_rect(area, 97, 29);
        self.render_border(rect, buf, " Level Meter ", Color::METER_LOW);
        self.render_header(rect, buf, state, WaveformMode::LufsMeter.name());

        let Some(layout) = PlotLayout::from_rect(rect, 2, 2) else {
            return;
        };

        let gap: u16 = 4;
        if layout.grid_width <= gap + 3 {
            self.draw_status(layout, buf, "Terminal too small", Color::GRAY);
            return;
        }

        let meter_width = (layout.grid_width - gap) / 2;
        let left_x = layout.grid_x;
        let right_x = layout.grid_x + meter_width + gap;

        let viz = &state.audio.visualization;

        self.render_single_meter(
            left_x,
            layout.grid_y,
            meter_width,
            layout.grid_height,
            viz.peak_l,
            viz.rms_l,
            "L",
            buf,
        );
        self.render_single_meter(
            right_x,
            layout.grid_y,
            meter_width,
            layout.grid_height,
            viz.peak_r,
            viz.rms_r,
            "R",
            buf,
        );

        let peak_db_l = amp_to_db(viz.peak_l);
        let peak_db_r = amp_to_db(viz.peak_r);
        let rms_db_l = amp_to_db(viz.rms_l);
        let rms_db_r = amp_to_db(viz.rms_r);
        let status = format!(
            "L: peak {:.1}dB rms {:.1}dB    R: peak {:.1}dB rms {:.1}dB    [Tab: cycle mode]",
            peak_db_l, rms_db_l, peak_db_r, rms_db_r,
        );
        self.draw_status(layout, buf, &status, Color::GRAY);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_single_meter(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        peak: f32,
        rms: f32,
        label: &str,
        buf: &mut RenderBuf,
    ) {
        if width == 0 || height == 0 {
            return;
        }

        // dB scale: -60 to 0
        let db_range = 60.0_f32;
        let peak_db = amp_to_db(peak).max(-db_range);
        let rms_db = amp_to_db(rms).max(-db_range);
        let peak_frac = ((peak_db + db_range) / db_range).clamp(0.0, 1.0);
        let rms_frac = ((rms_db + db_range) / db_range).clamp(0.0, 1.0);

        let peak_height = (peak_frac * height as f32).round() as u16;
        let rms_height = (rms_frac * height as f32).round() as u16;

        let has_peak_col = width >= 2;
        let rms_width = if has_peak_col { width - 1 } else { width };

        // Draw RMS bars from bottom up
        for dy in 0..rms_height.min(height) {
            let row = y + height - 1 - dy;
            let frac = (dy + 1) as f32 / height as f32;
            let style = Style::new().fg(meter_color(frac));
            for bx in 0..rms_width {
                buf.set_cell(x + bx, row, WAVEFORM_CHARS[7], style);
            }
        }

        // Draw peak indicator at right edge
        if has_peak_col && peak_height > 0 {
            let peak_y = y + height - peak_height.min(height);
            let peak_frac_color = peak_height as f32 / height as f32;
            let peak_color = meter_color(peak_frac_color);
            buf.set_cell(
                x + width - 1,
                peak_y,
                '\u{2501}',
                Style::new().fg(peak_color),
            );
        }

        // Channel label
        let label_x = x + width / 2;
        let label_y = y + height;
        buf.draw_line(
            Rect::new(label_x, label_y, 2, 1),
            &[(label, Style::new().fg(Color::WHITE))],
        );

        // dB markers at left side
        let markers = [
            ("0", 0.0),
            ("-6", 6.0),
            ("-12", 12.0),
            ("-24", 24.0),
            ("-48", 48.0),
        ];
        let marker_style = Style::new().fg(Color::DARK_GRAY);
        for (text, db_offset) in markers {
            let frac = (db_range - db_offset) / db_range;
            let marker_y = y + ((1.0 - frac) * height as f32).round() as u16;
            if marker_y >= y && marker_y < y + height {
                let marker_x = x.saturating_sub(text.len() as u16 + 1);
                buf.draw_line(
                    Rect::new(marker_x, marker_y, text.len() as u16, 1),
                    &[(text, marker_style)],
                );
            }
        }
    }

    fn render_border(&self, rect: Rect, buf: &mut RenderBuf, title: &str, color: Color) {
        let border_style = Style::new().fg(color);
        buf.draw_block(rect, title, border_style, border_style);
    }

    fn render_header(&self, rect: Rect, buf: &mut RenderBuf, state: &AppState, mode_name: &str) {
        let header_y = rect.y + 1;
        let play_icon = if state.audio.playing { "||" } else { "> " };
        let header_text = format!(" BPM:{:.0}  {}  {}", state.audio.bpm, play_icon, mode_name);
        buf.draw_line(
            Rect::new(rect.x + 1, header_y, rect.width.saturating_sub(2), 1),
            &[(&header_text, Style::new().fg(Color::WHITE))],
        );
    }
}

impl Pane for WaveformPane {
    fn id(&self) -> &'static str {
        "waveform"
    }

    fn handle_action(
        &mut self,
        action: ActionId,
        _event: &InputEvent,
        _state: &AppState,
    ) -> Action {
        match action {
            ActionId::Waveform(WaveformActionId::CycleMode) => {
                self.mode = self.mode.next();
                Action::None
            }
            ActionId::Waveform(WaveformActionId::Left) => {
                let len = _state
                    .recorded_waveform_peaks
                    .as_ref()
                    .map(|w| w.len())
                    .unwrap_or(0);
                self.move_cursor_left(len);
                Action::None
            }
            ActionId::Waveform(WaveformActionId::Right) => {
                let len = _state
                    .recorded_waveform_peaks
                    .as_ref()
                    .map(|w| w.len())
                    .unwrap_or(0);
                self.move_cursor_right(len);
                Action::None
            }
            ActionId::Waveform(WaveformActionId::JumpLeft) => {
                if let Some(wave) = _state.recorded_waveform_peaks.as_ref() {
                    self.jump_cursor_left(wave);
                }
                Action::None
            }
            ActionId::Waveform(WaveformActionId::JumpRight) => {
                if let Some(wave) = _state.recorded_waveform_peaks.as_ref() {
                    self.jump_cursor_right(wave);
                }
                Action::None
            }
            ActionId::Waveform(WaveformActionId::ToggleSelect) => {
                if _state.recorded_waveform_peaks.is_none() {
                    return Action::None;
                }
                if !self.select_mode {
                    self.select_mode = true;
                    self.select_anchor = Some(self.cursor_index);
                    self.selection = None;
                } else {
                    self.select_mode = false;
                    self.selection = self.select_anchor.map(|anchor| {
                        if anchor <= self.cursor_index {
                            (anchor, self.cursor_index)
                        } else {
                            (self.cursor_index, anchor)
                        }
                    });
                    self.select_anchor = None;
                }
                Action::None
            }
            ActionId::Waveform(WaveformActionId::DeleteSelection) => {
                let len = _state
                    .recorded_waveform_peaks
                    .as_ref()
                    .map(|w| w.len())
                    .unwrap_or(0);
                if let Some(selection) = self.active_selection() {
                    if let Some((start, end)) = self.selection_to_norm(selection, len) {
                        self.select_mode = false;
                        self.select_anchor = None;
                        self.selection = None;
                        self.cursor_index = selection.0.min(len.saturating_sub(1));
                        return Action::Session(SessionAction::DeleteWaveformSelection {
                            start,
                            end,
                        });
                    }
                }
                Action::None
            }
            ActionId::Waveform(WaveformActionId::TrimSelection) => {
                let len = _state
                    .recorded_waveform_peaks
                    .as_ref()
                    .map(|w| w.len())
                    .unwrap_or(0);
                if let Some(selection) = self.active_selection() {
                    if let Some((start, end)) = self.selection_to_norm(selection, len) {
                        self.select_mode = false;
                        self.select_anchor = None;
                        self.selection = None;
                        self.cursor_index = 0;
                        self.view_offset = 0;
                        return Action::Session(SessionAction::TrimWaveformSelection {
                            start,
                            end,
                        });
                    }
                }
                if let Some(wave) = _state.recorded_waveform_peaks.as_ref() {
                    const SILENCE_THRESHOLD: f32 = 0.01;
                    if let Some((start_idx, end_idx)) =
                        Self::non_silent_bounds(wave, SILENCE_THRESHOLD)
                    {
                        // No detectable front/back silence to trim.
                        if start_idx == 0 && end_idx.saturating_add(1) >= wave.len() {
                            return Action::None;
                        }
                        if let Some((start, end)) =
                            self.selection_to_norm((start_idx, end_idx), wave.len())
                        {
                            self.select_mode = false;
                            self.select_anchor = None;
                            self.selection = None;
                            self.cursor_index = 0;
                            self.view_offset = 0;
                            return Action::Session(SessionAction::TrimWaveformSelection {
                                start,
                                end,
                            });
                        }
                    }
                }
                Action::None
            }
            ActionId::Waveform(WaveformActionId::RenameClip) => {
                if _state.recorded_waveform_peaks.is_none() {
                    return Action::None;
                }
                let current_name = self.current_clip_name(_state);
                self.edit_input.set_value(&current_name);
                self.edit_input.select_all();
                self.edit_input.set_focused(true);
                self.editing_name = true;
                Action::PushLayer("text_edit")
            }
            ActionId::Waveform(WaveformActionId::CreatePitchedSampler) => {
                let len = _state
                    .recorded_waveform_peaks
                    .as_ref()
                    .map(|w| w.len())
                    .unwrap_or(0);
                if let Some((start, end)) = self.selection_or_full_norm(len) {
                    return Action::Session(
                        SessionAction::CreatePitchedSamplerFromWaveformSelection { start, end },
                    );
                }
                Action::None
            }
            ActionId::Waveform(WaveformActionId::CreateTimeStretchSampler) => {
                let len = _state
                    .recorded_waveform_peaks
                    .as_ref()
                    .map(|w| w.len())
                    .unwrap_or(0);
                if let Some((start, end)) = self.selection_or_full_norm(len) {
                    return Action::Session(
                        SessionAction::CreateTimeStretchFromWaveformSelection { start, end },
                    );
                }
                Action::None
            }
            ActionId::Waveform(WaveformActionId::OpenInChopper) => {
                let len = _state
                    .recorded_waveform_peaks
                    .as_ref()
                    .map(|w| w.len())
                    .unwrap_or(0);
                if let Some((start, end)) = self.selection_or_full_norm(len) {
                    return Action::Session(SessionAction::OpenWaveformSelectionInChopper {
                        start,
                        end,
                    });
                }
                Action::None
            }
            ActionId::Mode(ModeActionId::TextConfirm) => {
                if !self.editing_name {
                    return Action::None;
                }
                let next_name = self.edit_input.value().trim().to_string();
                self.editing_name = false;
                self.edit_input.set_focused(false);
                if next_name.is_empty() {
                    return Action::None;
                }
                Action::Session(SessionAction::RenameWaveformClip(next_name))
            }
            ActionId::Mode(ModeActionId::TextCancel) => {
                if !self.editing_name {
                    return Action::None;
                }
                self.editing_name = false;
                self.edit_input.set_focused(false);
                Action::None
            }
            ActionId::Waveform(WaveformActionId::LoadSample) => Action::Session(
                SessionAction::RequestFileBrowser(FileSelectAction::LoadWaveformFile),
            ),
            _ => Action::None,
        }
    }

    fn handle_raw_input(&mut self, event: &InputEvent, _state: &AppState) -> Action {
        if self.editing_name {
            self.edit_input.handle_input(event);
        }
        Action::None
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        match self.mode {
            WaveformMode::Waveform => self.render_waveform(area, buf, state),
            WaveformMode::Spectrum => self.render_spectrum(area, buf, state),
            WaveformMode::Oscilloscope => self.render_oscilloscope(area, buf, state),
            WaveformMode::LufsMeter => self.render_lufs_meter(area, buf, state),
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{braille_char, resample_linear, resample_peak_abs};

    #[test]
    fn braille_char_non_zero_pattern() {
        assert_ne!(braille_char(0b1), ' ');
    }

    #[test]
    fn peak_resample_respects_output_width() {
        let out = resample_peak_abs(&[0.1, -0.7, 0.3, -0.2], 10);
        assert_eq!(out.len(), 10);
        assert!(out.iter().all(|v| *v >= 0.0 && *v <= 1.0));
    }

    #[test]
    fn linear_resample_preserves_endpoints() {
        let out = resample_linear(&[-1.0, 1.0], 5);
        assert_eq!(out.len(), 5);
        assert!((out[0] + 1.0).abs() < 0.001);
        assert!((out[4] - 1.0).abs() < 0.001);
    }
}
