use std::any::Any;
use std::time::Instant;

use crate::panes::FileBrowserPane;
use crate::state::AppState;
use crate::ui::action_id::{ActionId, SampleSlicerActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::widgets::waveform;
use crate::ui::{
    Action, FileSelectAction, InputEvent, Keymap, NavAction, Palette, Pane, PaneIdStr, Rect,
    RenderBuf, SampleSlicerAction, Style,
};

pub struct SampleSlicerPane {
    keymap: Keymap,
    cursor_pos: f32, // 0.0-1.0
    auto_slice_n: usize,
    file_browser: FileBrowserPane,
    /// Preview playback tracking
    preview_started_at: Option<Instant>,
    preview_slice_start: f32,
    preview_slice_end: f32,
    preview_duration_secs: f32,
}

impl SampleSlicerPane {
    pub fn new(keymap: Keymap, file_browser_keymap: Keymap) -> Self {
        Self {
            keymap,
            cursor_pos: 0.5,
            auto_slice_n: 4,
            file_browser: FileBrowserPane::new(file_browser_keymap),
            preview_started_at: None,
            preview_slice_start: 0.0,
            preview_slice_end: 1.0,
            preview_duration_secs: 0.0,
        }
    }

    fn selected_drum_sequencer<'a>(
        &self,
        state: &'a AppState,
    ) -> Option<&'a crate::state::drum_sequencer::DrumSequencerState> {
        state
            .tracks
            .selected_track()
            .and_then(|i| i.drum_sequencer())
    }

    fn get_slicer_state<'a>(
        &self,
        state: &'a AppState,
    ) -> Option<&'a crate::state::drum_sequencer::SampleSlicerState> {
        self.selected_drum_sequencer(state)
            .and_then(|d| d.chopper.as_ref())
    }

    fn should_show_file_browser(&self, state: &AppState) -> bool {
        self.selected_drum_sequencer(state)
            .map(|d| d.chopper.is_none())
            .unwrap_or(false)
    }

    /// Start tracking a preview playback.
    fn start_preview(&mut self, state: &AppState) {
        if let Some(chopper) = self.get_slicer_state(state) {
            if let Some(slice) = chopper.slices.get(chopper.selected_slice) {
                self.preview_slice_start = slice.start;
                self.preview_slice_end = slice.end;
                self.preview_duration_secs = (slice.end - slice.start) * chopper.duration_secs;
                self.preview_started_at = Some(Instant::now());
            }
        }
    }

    /// Get current preview playback progress as normalized position (0.0-1.0),
    /// or None if not playing.
    fn preview_progress(&self) -> Option<f32> {
        let started = self.preview_started_at?;
        if self.preview_duration_secs <= 0.0 {
            return None;
        }
        let elapsed = started.elapsed().as_secs_f32();
        let local_progress = elapsed / self.preview_duration_secs;
        if local_progress >= 1.0 {
            return None;
        }
        let progress = self.preview_slice_start
            + local_progress * (self.preview_slice_end - self.preview_slice_start);
        Some(progress)
    }
}

impl Default for SampleSlicerPane {
    fn default() -> Self {
        Self::new(Keymap::new(), Keymap::new())
    }
}

impl Pane for SampleSlicerPane {
    fn id(&self) -> PaneIdStr {
        PaneIdStr("sample_slicer")
    }

    fn handle_action(&mut self, action: ActionId, event: &InputEvent, state: &AppState) -> Action {
        if self.should_show_file_browser(state) {
            if let Some(fb_action) = self.file_browser.keymap().lookup(event) {
                return self.file_browser.handle_action(fb_action, event, state);
            }
            return Action::None;
        }

        match action {
            ActionId::SampleSlicer(SampleSlicerActionId::MoveLeft) => {
                self.cursor_pos = (self.cursor_pos - 0.01).max(0.0);
                Action::SampleSlicer(SampleSlicerAction::MoveCursor(-1))
            }
            ActionId::SampleSlicer(SampleSlicerActionId::MoveRight) => {
                self.cursor_pos = (self.cursor_pos + 0.01).min(1.0);
                Action::SampleSlicer(SampleSlicerAction::MoveCursor(1))
            }
            ActionId::SampleSlicer(SampleSlicerActionId::NextSlice) => {
                Action::SampleSlicer(SampleSlicerAction::SelectSlice(1))
            }
            ActionId::SampleSlicer(SampleSlicerActionId::PrevSlice) => {
                Action::SampleSlicer(SampleSlicerAction::SelectSlice(-1))
            }
            ActionId::SampleSlicer(SampleSlicerActionId::NudgeStart) => {
                Action::SampleSlicer(SampleSlicerAction::NudgeSliceStart(-0.005))
            }
            ActionId::SampleSlicer(SampleSlicerActionId::NudgeEnd) => {
                Action::SampleSlicer(SampleSlicerAction::NudgeSliceEnd(0.005))
            }
            ActionId::SampleSlicer(SampleSlicerActionId::Chop) => {
                Action::SampleSlicer(SampleSlicerAction::AddSlice(self.cursor_pos))
            }
            ActionId::SampleSlicer(SampleSlicerActionId::Delete) => {
                Action::SampleSlicer(SampleSlicerAction::RemoveSlice)
            }
            ActionId::SampleSlicer(SampleSlicerActionId::AutoSlice) => {
                let n = self.auto_slice_n;
                self.auto_slice_n = match n {
                    4 => 8,
                    8 => 12,
                    12 => 16,
                    _ => 4,
                };
                Action::SampleSlicer(SampleSlicerAction::AutoSlice(n))
            }
            ActionId::SampleSlicer(SampleSlicerActionId::Commit) => {
                Action::SampleSlicer(SampleSlicerAction::CommitAll)
            }
            ActionId::SampleSlicer(SampleSlicerActionId::LoadSample) => {
                Action::SampleSlicer(SampleSlicerAction::LoadSample)
            }
            ActionId::SampleSlicer(SampleSlicerActionId::Preview) => {
                self.start_preview(state);
                Action::SampleSlicer(SampleSlicerAction::PreviewSlice)
            }
            ActionId::SampleSlicer(SampleSlicerActionId::Back) => Action::Nav(NavAction::PopPane),
            ActionId::SampleSlicer(SampleSlicerActionId::AssignToPad(pad_num)) => {
                Action::SampleSlicer(SampleSlicerAction::AssignToPad(
                    pad_num.saturating_sub(1) as usize
                ))
            }
            _ => Action::None,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        // Delegate to file browser before unwrapping RenderBuf
        if let Some(drum_seq) = self.selected_drum_sequencer(state) {
            if drum_seq.chopper.is_none() {
                self.file_browser.render(area, buf, state);
                return;
            }
        }

        let rect = center_rect(area, 97, 29);

        if self.selected_drum_sequencer(state).is_none() {
            let border_style = Style::new().fg(p.dim);
            let inner = buf.draw_block(rect, " Sample Slicer ", border_style, border_style);
            buf.draw_line(
                Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1),
                &[("No drum machine track selected.", Style::new().fg(p.muted))],
            );
            return;
        }

        let border_style = Style::new().fg(p.dim);
        let _inner = buf.draw_block(rect, " Sample Slicer ", border_style, border_style);

        // Get slicer state
        let chopper = match self.get_slicer_state(state) {
            Some(c) => c,
            None => {
                buf.draw_line(
                    Rect::new(rect.x + 2, rect.y + 2, rect.width.saturating_sub(4), 1),
                    &[("No sample loaded.", Style::new().fg(p.muted))],
                );
                return;
            }
        };

        let content_x = rect.x + 2;
        let content_y = rect.y + 2;

        // Header info
        let filename = chopper
            .path
            .as_ref()
            .map(|p| {
                std::path::Path::new(p)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| "No Sample".to_string());
        buf.draw_line(
            Rect::new(content_x, content_y, rect.width.saturating_sub(4), 1),
            &[(&filename, Style::new().fg(p.accent).bold())],
        );

        let info = format!(
            "{:.1}s   {} slices",
            chopper.duration_secs,
            chopper.slices.len()
        );
        let info_x = rect.x + rect.width - 2 - info.len() as u16;
        buf.draw_line(
            Rect::new(
                info_x,
                content_y,
                rect.width.saturating_sub(info_x - rect.x),
                1,
            ),
            &[(&info, Style::new().fg(p.muted))],
        );

        // Waveform
        let wave_y = content_y + 2;
        let wave_height: u16 = 8;
        let wave_width = rect.width - 4;

        if !chopper.waveform_peaks.is_empty() {
            waveform::draw_center_line(content_x, wave_y, wave_width, wave_height, buf, &p);
            waveform::draw_braille_waveform(
                &chopper.waveform_peaks,
                content_x,
                wave_y,
                wave_width,
                wave_height,
                buf,
                &p,
            );
        } else {
            buf.draw_line(
                Rect::new(content_x, wave_y + wave_height / 2, 20, 1),
                &[("(No waveform data)", Style::new().fg(p.muted))],
            );
        }

        // Draw slices
        let wave_width_usize = wave_width as usize;
        let slice_y_start = wave_y;
        let slice_y_end = wave_y + wave_height;
        let dark_gray_style = Style::new().fg(p.dim);
        let sel_bg_style = Style::new().bg(p.selection_bg);
        let sel_white_style = Style::new().fg(p.fg).bg(p.selection_bg);

        for (i, slice) in chopper.slices.iter().enumerate() {
            let start_x = (slice.start * wave_width_usize as f32) as u16;
            let end_x = (slice.end * wave_width_usize as f32) as u16;
            let center_x = (start_x + end_x) / 2;

            // Draw slice boundaries
            if i > 0 {
                for y in slice_y_start..=slice_y_end {
                    buf.set_cell(content_x + start_x, y, '|', dark_gray_style);
                }
            }

            // Highlight selected slice
            if i == chopper.selected_slice {
                for x in start_x..end_x {
                    if x >= wave_width {
                        break;
                    }
                    buf.set_cell(content_x + x, slice_y_end + 1, ' ', sel_bg_style);
                }
                let label = format!("{}", i + 1);
                let lx = content_x + center_x.saturating_sub(label.len() as u16 / 2);
                for (j, ch) in label.chars().enumerate() {
                    buf.set_cell(lx + j as u16, slice_y_end + 1, ch, sel_white_style);
                }
            } else {
                let label = format!("{}", i + 1);
                if end_x - start_x > label.len() as u16 {
                    let lx = content_x + center_x.saturating_sub(label.len() as u16 / 2);
                    for (j, ch) in label.chars().enumerate() {
                        buf.set_cell(lx + j as u16, slice_y_end + 1, ch, dark_gray_style);
                    }
                }
            }
        }

        // Draw playback playhead
        if let Some(progress) = self.preview_progress() {
            waveform::draw_playhead(
                progress,
                content_x,
                wave_y,
                wave_width,
                wave_height,
                buf,
                p.error,
            );
        }

        // Draw cursor
        let cursor_screen_x = (self.cursor_pos * wave_width_usize as f32) as u16;
        let yellow_style = Style::new().fg(p.warning);
        for y in slice_y_start..=slice_y_end {
            buf.set_cell(content_x + cursor_screen_x, y, '┆', yellow_style);
        }
        buf.set_cell(
            content_x + cursor_screen_x,
            slice_y_end + 2,
            '▲',
            yellow_style,
        );

        // List slices
        let list_y = slice_y_end + 4;
        for i in 0..8 {
            if i >= chopper.slices.len() {
                break;
            }
            let slice = &chopper.slices[i];
            let y = list_y + i as u16;

            if i == chopper.selected_slice {
                buf.set_cell(content_x, y, '>', Style::new().fg(p.fg).bold());
            }

            let text = format!("{:<2} {:.3}-{:.3}", i + 1, slice.start, slice.end);
            let style = Style::new().fg(if i == chopper.selected_slice {
                p.fg
            } else {
                p.dim
            });
            buf.draw_line(
                Rect::new(content_x + 2, y, text.len() as u16, 1),
                &[(&text, style)],
            );

            // Check pad assignments
            if let Some(inst) = state.tracks.selected_track() {
                if let Some(ds) = inst.drum_sequencer() {
                    for (pad_idx, pad) in ds.pads.iter().enumerate() {
                        if pad.buffer_id == chopper.buffer_id
                            && (pad.slice_start - slice.start).abs() < 0.001
                            && (pad.slice_end - slice.end).abs() < 0.001
                        {
                            let pad_label = format!("→ Pad {}", pad_idx + 1);
                            buf.draw_line(
                                Rect::new(content_x + 25, y, pad_label.len() as u16, 1),
                                &[(&pad_label, style)],
                            );
                        }
                    }
                }
            }
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_enter(&mut self, state: &AppState) {
        if self.should_show_file_browser(state) {
            self.file_browser
                .open_for(FileSelectAction::LoadSlicerSample, None);
        }
    }
}
