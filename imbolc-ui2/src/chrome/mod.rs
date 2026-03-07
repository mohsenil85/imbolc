pub mod modeline;

use crate::audio::ServerStatus;
use crate::state::AppState;
use crate::ui::render::{Rect, RenderBuf};
use crate::ui::status_bar::{StatusBar, StatusLevel};
use crate::ui::style::{Palette, Style};

/// Global chrome: header bar (row 0) and status bar (last row).
pub struct GlobalChrome {
    pub project_name: String,
    pub status_bar: StatusBar,
    pub recording: bool,
    pub recording_secs: u64,

    // Meter/metrics
    master_peak: f32,
    peak_display: f32,
    master_mute: bool,
    sc_cpu: f32,
    osc_latency_ms: f32,
    audio_latency_ms: f32,

    // Autosave
    autosave_enabled: bool,
    autosave_interval_minutes: u64,
}

impl GlobalChrome {
    pub fn new() -> Self {
        Self {
            project_name: "untitled".to_string(),
            status_bar: StatusBar::new(),
            recording: false,
            recording_secs: 0,
            master_peak: 0.0,
            peak_display: 0.0,
            master_mute: false,
            sc_cpu: 0.0,
            osc_latency_ms: 0.0,
            audio_latency_ms: 0.0,
            autosave_enabled: false,
            autosave_interval_minutes: 2,
        }
    }

    pub fn set_project_name(&mut self, name: String) {
        self.project_name = name;
    }

    pub fn set_master_peak(&mut self, peak: f32, mute: bool) {
        self.master_peak = peak;
        self.master_mute = mute;
        self.peak_display = peak.max(self.peak_display * 0.85);
    }

    pub fn set_sc_metrics(&mut self, cpu: f32, osc_latency_ms: f32, audio_latency_ms: f32) {
        self.sc_cpu = cpu;
        self.osc_latency_ms = osc_latency_ms;
        self.audio_latency_ms = audio_latency_ms;
    }

    pub fn set_autosave_config(&mut self, enabled: bool, interval_minutes: u64) {
        self.autosave_enabled = enabled;
        self.autosave_interval_minutes = interval_minutes.max(1);
    }

    /// Render the global header bar (row 0 of the terminal).
    pub fn render_header(&self, buf: &mut RenderBuf, area: Rect, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        let session = &state.session;

        // Fill header row with accent background
        let header_rect = Rect::new(area.x, area.y, area.width, 1);
        buf.fill_bg(header_rect, p.border);

        let dirty = if state.project.dirty { "*" } else { "" };
        let tuning_str = {
            use imbolc_types::state::music::Tuning;
            let a4 = format!("A{:.0}", session.tuning_a4);
            if session.tuning == Tuning::EqualTemperament {
                a4
            } else {
                format!("{} {}", session.tuning.short_name(), a4)
            }
        };
        let header = format!(
            " {}{} | {} {} | BPM {} | {}/{} | {} | Snap:{} ",
            self.project_name,
            dirty,
            session.key.name(),
            session.scale.name(),
            session.bpm,
            session.time_signature.0,
            session.time_signature.1,
            tuning_str,
            if session.snap { "on" } else { "off" },
        );
        buf.draw_str(area.x, area.y, &header, Style::new().fg(p.accent));

        // Right-aligned indicators
        let mut cursor = area.x + area.width;

        // Track indicator
        if let Some(idx) = state.tracks.selected {
            if let Some(inst) = state.tracks.tracks.get(idx) {
                let text = format!(" {}:{} ", idx + 1, inst.name);
                cursor = cursor.saturating_sub(text.len() as u16);
                buf.draw_str(cursor, area.y, &text, Style::new().fg(p.fg));
            }
        }

        // REC indicator
        if self.recording {
            let mins = self.recording_secs / 60;
            let secs = self.recording_secs % 60;
            let text = format!(" REC {:02}:{:02} ", mins, secs);
            cursor = cursor.saturating_sub(text.len() as u16);
            buf.draw_str(cursor, area.y, &text, Style::new().fg(p.recording).bold());
        }
    }

    /// Render the global status bar (last row of the terminal).
    pub fn render_status_bar(&self, buf: &mut RenderBuf, area: Rect, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        let bar_rect = Rect::new(area.x, area.y, area.width, 1);
        buf.fill_bg(bar_rect, p.border);

        // Left side: CPU, latency
        if self.sc_cpu > 0.0 || self.audio_latency_ms > 0.0 {
            let cpu_color = if self.sc_cpu > 80.0 {
                p.error
            } else if self.sc_cpu > 50.0 {
                p.warning
            } else {
                p.success
            };
            let text = format!(
                " CPU:{:.0}% Audio:{:.1}ms OSC:{:.1}ms ",
                self.sc_cpu, self.audio_latency_ms, self.osc_latency_ms,
            );
            buf.draw_str(area.x, area.y, &text, Style::new().fg(cpu_color));
        }

        // Center: status message
        if let Some(msg) = self.status_bar.current() {
            let color = match msg.level {
                StatusLevel::Info => p.success,
                StatusLevel::Warning => p.warning,
                StatusLevel::Error => p.error,
            };
            let display = format!(" {} ", msg.text);
            let x = area.x + (area.width.saturating_sub(display.len() as u16)) / 2;
            buf.draw_str(x, area.y, &display, Style::new().fg(color));
        }

        // Right side: SC/MIDI dots
        if area.width > 30 {
            let right = area.x + area.width;
            let sc_color = match state.audio.server_status {
                ServerStatus::Connected => p.success,
                ServerStatus::Starting | ServerStatus::Running => p.warning,
                ServerStatus::Stopped => p.dim,
                ServerStatus::Error => p.error,
            };
            let midi_connected = state.midi.connected_port.is_some();
            let midi_color = if midi_connected { p.success } else { p.dim };

            let midi_label = "MIDI ";
            let midi_x = right.saturating_sub(midi_label.len() as u16 + 1);
            buf.draw_str(midi_x, area.y, midi_label, Style::new().fg(p.dim));
            buf.set_cell(
                midi_x.saturating_sub(1),
                area.y,
                '●',
                Style::new().fg(midi_color),
            );

            let sc_label = "SC ";
            let sc_x = midi_x.saturating_sub(sc_label.len() as u16 + 2);
            buf.draw_str(sc_x, area.y, sc_label, Style::new().fg(p.dim));
            buf.set_cell(
                sc_x.saturating_sub(1),
                area.y,
                '●',
                Style::new().fg(sc_color),
            );
        }
    }
}
