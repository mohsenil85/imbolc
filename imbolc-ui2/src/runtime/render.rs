use std::time::Instant;

use super::AppRuntime;
use crate::ui::ratatui_impl::RatatuiBackend;
use crate::ui::render::RenderBuf;
use crate::ui::style::Color;

impl AppRuntime {
    /// Render at ~60fps if needed.
    pub(crate) fn maybe_render(&mut self, backend: &mut RatatuiBackend) -> std::io::Result<()> {
        let now = Instant::now();
        if now.duration_since(self.last_render_time).as_millis() < 16 {
            return Ok(());
        }

        if self.audio.is_running() {
            self.render_needed = true;
        }

        if self.chrome.status_bar.current().is_some() {
            self.render_needed = true;
        }

        if !self.render_needed {
            return Ok(());
        }

        self.last_render_time = now;

        // Update master meter
        {
            let peak = if self.audio.is_running() {
                self.audio.master_peak()
            } else {
                0.0
            };
            let mute = self.dispatcher.state().session.mixer.master_mute;
            self.chrome.set_master_peak(peak, mute);
        }

        // Update SC metrics
        {
            let cpu = if self.audio.is_running() {
                self.audio.sc_cpu()
            } else {
                0.0
            };
            let osc_latency = if self.audio.is_running() {
                self.audio.osc_latency_ms()
            } else {
                0.0
            };
            let audio_latency = self.audio.audio_latency_ms();
            self.chrome.set_sc_metrics(cpu, osc_latency, audio_latency);
        }

        // Update recording state
        {
            let state = self.dispatcher.state_mut();
            state.recording.recording = self.audio.is_recording();
            state.recording.recording_secs = self
                .audio
                .recording_elapsed()
                .map(|d| d.as_secs())
                .unwrap_or(0);
            self.chrome.recording = state.recording.recording;
            self.chrome.recording_secs = state.recording.recording_secs;
        }

        // Copy audio-owned state into AppState
        {
            let ars = self.audio.read_state();
            let state = self.dispatcher.state_mut();
            state.audio.playhead = ars.playhead;
            state.audio.bpm = ars.bpm;
            state.audio.playing = ars.playing;
            state.audio.server_status = ars.server_status;
        }

        // Render
        let mut frame = backend.begin_frame()?;
        let area = frame.area();
        self.last_area = area;
        let cursor_pos;
        {
            let mut buf = RenderBuf::new(frame.buffer_mut());

            // Fill background
            let bg: Color = self.dispatcher.state().session.theme.background.into();
            buf.fill_bg(area, bg);

            // Header (row 0)
            let header_rect = ratatui::layout::Rect::new(area.x, area.y, area.width, 1);
            self.chrome
                .render_header(&mut buf, header_rect, self.dispatcher.state());

            // Window tree (rows 1..n-1)
            let win_area = super::window_area(area);
            crate::window::render::render_window_tree(
                &self.tree,
                &mut self.registry,
                win_area,
                &mut buf,
                self.dispatcher.state(),
            );

            // Status bar (last row)
            let status_rect = ratatui::layout::Rect::new(
                area.x,
                area.y + area.height.saturating_sub(1),
                area.width,
                1,
            );
            self.chrome
                .render_status_bar(&mut buf, status_rect, self.dispatcher.state());

            cursor_pos = buf.cursor_position();
        }
        if let Some((x, y)) = cursor_pos {
            frame.set_cursor_position(x, y);
        }
        backend.end_frame(frame)?;

        self.render_needed = false;
        Ok(())
    }
}
