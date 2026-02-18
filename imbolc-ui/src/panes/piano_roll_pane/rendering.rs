use crate::state::automation::{AutomationTargetExt, CurveType};
use crate::state::AppState;
use crate::ui::layout_helpers::center_rect;
use crate::ui::{Palette, Rect, RenderBuf, Style};
use imbolc_types::state::piano_roll::Note;

use super::{PianoRollPane, TargetPickerState};

/// MIDI note name for a given pitch (0-127)
pub(super) fn note_name(pitch: u8) -> String {
    let names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (pitch / 12) as i8 - 1;
    let name = names[(pitch % 12) as usize];
    format!("{}{}", name, octave)
}

/// Check if a pitch is a black key
pub(super) fn is_black_key(pitch: u8) -> bool {
    matches!(pitch % 12, 1 | 3 | 6 | 8 | 10)
}

impl PianoRollPane {
    /// Render notes grid (buffer version)
    pub(super) fn render_notes_buf(&self, buf: &mut RenderBuf, area: Rect, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        let piano_roll = &state.session.piano_roll;

        // When rendering in split mode, use the area directly; otherwise center
        let rect = if self.automation_visible {
            area
        } else {
            center_rect(area, 97, 29)
        };

        // Layout constants
        let key_col_width: u16 = 5;
        let header_height: u16 = 2;
        let footer_height: u16 = 2;
        let grid_x = rect.x + key_col_width;
        let grid_y = rect.y + header_height;
        let grid_width = rect.width.saturating_sub(key_col_width + 1);
        let grid_height = rect
            .height
            .saturating_sub(header_height + footer_height + 1);

        // Compute virtual notes for Kit tracks
        let is_kit = self.is_kit_track(state);
        let virtual_notes: Option<Vec<Note>> = self
            .current_drum_sequencer(state)
            .map(|seq| seq.to_virtual_notes());

        // Border
        let track_label = if let Some(ref ctx) = state.session.arrangement.editing_clip {
            let clip_name = state
                .session
                .arrangement
                .clip(ctx.clip_id)
                .map(|c| c.name.as_str())
                .unwrap_or("?");
            format!(" Piano Roll - Editing: {} ", clip_name)
        } else if is_kit {
            format!(
                " Piano Roll: Kit [{}/{}] (read-only) ",
                self.current_track + 1,
                piano_roll.sequence_order.len(),
            )
        } else if let Some(seq) = piano_roll.sequence_at(self.current_track) {
            let mode = if seq.polyphonic { "POLY" } else { "MONO" };
            format!(
                " Piano Roll: midi-{} [{}/{}] {} ",
                seq.instrument_id,
                self.current_track + 1,
                piano_roll.sequence_order.len(),
                mode,
            )
        } else {
            " Piano Roll: (no tracks) ".to_string()
        };
        let border_color = if is_kit {
            p.accent_secondary
        } else if self.automation_visible && !self.automation_focus {
            p.fx_color
        } else if self.automation_visible {
            p.border
        } else {
            p.fx_color
        };
        let border_style = Style::new().fg(border_color);
        buf.draw_block(rect, &track_label, border_style, border_style);

        // Header: transport info
        let header_y = rect.y + 1;
        let play_icon = if state.audio.playing { "||" } else { "> " };
        let loop_icon = if piano_roll.looping { "L" } else { " " };
        let header_text = format!(
            " {}  {}  Beat:{:.1}",
            play_icon,
            loop_icon,
            piano_roll.tick_to_beat(state.audio.playhead),
        );
        buf.draw_line(
            Rect::new(rect.x + 1, header_y, rect.width.saturating_sub(2), 1),
            &[(&header_text, Style::new().fg(p.fg))],
        );

        // Loop range indicator
        if piano_roll.looping {
            let loop_info = format!(
                "Loop:{:.1}-{:.1}",
                piano_roll.tick_to_beat(piano_roll.loop_start),
                piano_roll.tick_to_beat(piano_roll.loop_end),
            );
            let loop_x = rect.x + rect.width - loop_info.len() as u16 - 2;
            buf.draw_line(
                Rect::new(
                    loop_x,
                    header_y,
                    rect.width.saturating_sub(loop_x - rect.x),
                    1,
                ),
                &[(&loop_info, Style::new().fg(p.warning))],
            );
        }

        // Rendering indicator
        if let Some(render) = &state.io.pending_render {
            if let Some(track_inst_id) = state
                .session
                .piano_roll
                .sequence_order
                .get(self.current_track)
            {
                if render.instrument_id == *track_inst_id {
                    let label = " RENDERING ";
                    let style = Style::new().fg(p.fg).bg(p.recording);
                    let x = rect.x + rect.width - label.len() as u16 - 2;
                    buf.draw_line(
                        Rect::new(x, header_y, label.len() as u16, 1),
                        &[(label, style)],
                    );
                }
            }
        }

        // Export progress indicator
        if let Some(export) = &state.io.pending_export {
            let progress = state.io.export_progress;
            let bar_width: usize = 20;
            let filled = (progress * bar_width as f32) as usize;
            let empty = bar_width.saturating_sub(filled);
            let label = match export.kind {
                imbolc_audio::commands::ExportKind::MasterBounce => "BOUNCING",
                imbolc_audio::commands::ExportKind::StemExport => "STEMS",
            };
            let text = format!(
                " {} [{}{}] {:.0}% ",
                label,
                "\u{2588}".repeat(filled),
                "\u{2591}".repeat(empty),
                progress * 100.0,
            );
            let style = Style::new().fg(p.fg).bg(p.accent_secondary);
            let x = rect.x + rect.width - text.len() as u16 - 2;
            buf.draw_line(
                Rect::new(x, header_y, text.len() as u16, 1),
                &[(&text, style)],
            );
        }

        // For Kit tracks, get pad info for key labels
        let kit_info: Option<(u8, Vec<String>)> = self.current_drum_sequencer(state).map(|seq| {
            let base = seq.midi_base_note;
            let pad_names: Vec<String> = seq
                .pads
                .iter()
                .map(|pad| {
                    if pad.name.is_empty() {
                        String::new()
                    } else {
                        pad.name.chars().take(3).collect()
                    }
                })
                .collect();
            (base, pad_names)
        });

        // Piano keys column + grid rows
        for row in 0..grid_height {
            let pitch = self
                .view_bottom_pitch
                .saturating_add((grid_height - 1 - row) as u8);
            if pitch > 127 {
                continue;
            }
            let y = grid_y + row;

            // Piano key label (pad name for Kit tracks, note name otherwise)
            let (name, is_pad_row) = if let Some((base, ref pad_names)) = kit_info {
                if pitch >= base && (pitch - base) < pad_names.len() as u8 {
                    let pad_idx = (pitch - base) as usize;
                    let pname = &pad_names[pad_idx];
                    if pname.is_empty() {
                        (format!("P{}", pad_idx + 1), true)
                    } else {
                        (pname.clone(), true)
                    }
                } else {
                    (note_name(pitch), false)
                }
            } else {
                (note_name(pitch), false)
            };
            let is_black = is_black_key(pitch);
            let key_style = if pitch == self.cursor_pitch {
                Style::new().fg(p.fg).bg(p.selection_bg)
            } else if is_pad_row {
                Style::new().fg(p.accent_secondary)
            } else if is_black {
                Style::new().fg(p.border)
            } else {
                Style::new().fg(p.fg)
            };
            let key_str = format!("{:>3}", name);
            for (j, ch) in key_str.chars().enumerate() {
                buf.set_cell(rect.x + 1 + j as u16, y, ch, key_style);
            }

            // Separator
            buf.set_cell(
                rect.x + key_col_width - 1,
                y,
                '|',
                Style::new().fg(p.border),
            );

            // Grid cells
            for col in 0..grid_width {
                let tick = self.view_start_tick + col as u32 * self.ticks_per_cell();
                let x = grid_x + col;

                // Note lookup: virtual notes for Kit, piano roll notes otherwise
                let (has_note, is_note_start) = if let Some(ref notes) = virtual_notes {
                    let has = notes
                        .iter()
                        .any(|n| n.pitch == pitch && tick >= n.tick && tick < n.tick + n.duration);
                    let start = notes.iter().any(|n| n.pitch == pitch && n.tick == tick);
                    (has, start)
                } else {
                    let has = piano_roll
                        .sequence_at(self.current_track)
                        .is_some_and(|track| {
                            track.notes.iter().any(|n| {
                                n.pitch == pitch && tick >= n.tick && tick < n.tick + n.duration
                            })
                        });
                    let start = piano_roll
                        .sequence_at(self.current_track)
                        .is_some_and(|track| {
                            track
                                .notes
                                .iter()
                                .any(|n| n.pitch == pitch && n.tick == tick)
                        });
                    (has, start)
                };

                let is_cursor = pitch == self.cursor_pitch && tick == self.cursor_tick;
                let is_playhead = state.audio.playing
                    && tick <= state.audio.playhead
                    && state.audio.playhead < tick + self.ticks_per_cell();

                let tpb = piano_roll.ticks_per_beat;
                let tpbar = piano_roll.ticks_per_bar();
                let is_bar_line = tick.is_multiple_of(tpbar);
                let is_beat_line = tick.is_multiple_of(tpb);

                let in_selection =
                    self.selection_anchor
                        .is_some_and(|(anchor_tick, anchor_pitch)| {
                            let (t0, t1) = if anchor_tick <= self.cursor_tick {
                                (anchor_tick, self.cursor_tick + self.ticks_per_cell())
                            } else {
                                (self.cursor_tick, anchor_tick + self.ticks_per_cell())
                            };
                            let (p0, p1) = if anchor_pitch <= self.cursor_pitch {
                                (anchor_pitch, self.cursor_pitch)
                            } else {
                                (self.cursor_pitch, anchor_pitch)
                            };
                            tick >= t0 && tick < t1 && pitch >= p0 && pitch <= p1
                        });

                // Note colors: orange for Kit, pink/magenta for normal
                let (note_start_color, note_body_color) = if is_kit {
                    (p.accent_secondary, p.accent_secondary)
                } else {
                    (p.fx_color, p.fx_color)
                };

                let (ch, style) = if is_cursor {
                    if has_note {
                        ('\u{2588}', Style::new().fg(p.bg).bg(p.fg))
                    } else {
                        ('\u{2592}', Style::new().fg(p.fg).bg(p.selection_bg))
                    }
                } else if in_selection && has_note {
                    // Selected note
                    ('\u{2588}', Style::new().fg(p.fg).bg(p.selection_bg))
                } else if in_selection {
                    // Selection region background
                    ('\u{2591}', Style::new().fg(p.selection_bg))
                } else if has_note {
                    if is_note_start {
                        ('\u{2588}', Style::new().fg(note_start_color))
                    } else {
                        ('\u{2588}', Style::new().fg(note_body_color))
                    }
                } else if is_playhead {
                    ('\u{2502}', Style::new().fg(p.playing))
                } else if is_bar_line {
                    ('\u{250a}', Style::new().fg(p.border))
                } else if is_beat_line {
                    ('\u{00b7}', Style::new().fg(p.dim))
                } else if is_black {
                    ('\u{00b7}', Style::new().fg(p.muted))
                } else {
                    (' ', Style::new())
                };

                buf.set_cell(x, y, ch, style);
            }
        }

        // Footer: beat markers
        let footer_y = grid_y + grid_height;
        for col in 0..grid_width {
            let tick = self.view_start_tick + col as u32 * self.ticks_per_cell();
            let tpb = piano_roll.ticks_per_beat;
            let tpbar = piano_roll.ticks_per_bar();
            let x = grid_x + col;

            if tick.is_multiple_of(tpbar) {
                let bar = tick / tpbar + 1;
                let label = format!("{}", bar);
                let bar_style = Style::new().fg(p.fg);
                for (j, ch) in label.chars().enumerate() {
                    buf.set_cell(x + j as u16, footer_y, ch, bar_style);
                }
            } else if tick.is_multiple_of(tpb) {
                buf.set_cell(x, footer_y, '\u{00b7}', Style::new().fg(p.border));
            }
        }

        // Status line
        let status_y = footer_y + 1;
        let vel_str = if is_kit {
            format!(
                "Kit (read-only)  Note:{} Tick:{}",
                note_name(self.cursor_pitch),
                self.cursor_tick,
            )
        } else if let Some((anchor_tick, anchor_pitch)) = self.selection_anchor {
            let t_diff = (self.cursor_tick as i64 - anchor_tick as i64).unsigned_abs() as u32
                + self.ticks_per_cell();
            let p_diff = (self.cursor_pitch as i16 - anchor_pitch as i16).abs() + 1;
            format!(
                "Sel: {:.1} beats x {} pitches",
                t_diff as f32 / piano_roll.ticks_per_beat as f32,
                p_diff
            )
        } else {
            format!(
                "Note:{} Tick:{} Vel:{} Dur:{}",
                note_name(self.cursor_pitch),
                self.cursor_tick,
                self.default_velocity,
                self.default_duration,
            )
        };
        buf.draw_line(
            Rect::new(rect.x + 1, status_y, rect.width.saturating_sub(2), 1),
            &[(&vel_str, Style::new().fg(p.border))],
        );

        // Piano mode indicator
        if self.piano.is_active() {
            let piano_str = self.piano.status_label();
            let mut indicator_x = rect.x + rect.width - piano_str.len() as u16 - 1;

            if self.recording {
                let rec_str = " REC ";
                indicator_x -= rec_str.len() as u16;
                let rec_style = Style::new().fg(p.fg).bg(p.recording);
                for (j, ch) in rec_str.chars().enumerate() {
                    buf.set_cell(indicator_x + j as u16, status_y, ch, rec_style);
                }
                indicator_x += rec_str.len() as u16;
            }

            let piano_style = Style::new().fg(p.bg).bg(p.fx_color);
            for (j, ch) in piano_str.chars().enumerate() {
                buf.set_cell(indicator_x + j as u16, status_y, ch, piano_style);
            }
        }
    }

    /// Render the horizontal separator between notes and automation
    pub(super) fn render_separator(
        &self,
        buf: &mut RenderBuf,
        rect: Rect,
        note_height: u16,
        state: &AppState,
    ) {
        let p = Palette::from(&state.session.theme);
        let sep_y = rect.y + note_height;

        let sep_color = if self.automation_focus {
            p.accent
        } else {
            p.border
        };
        let sep_style = Style::new().fg(sep_color);

        for x in rect.x..rect.x + rect.width {
            buf.set_cell(x, sep_y, '\u{2500}', sep_style);
        }

        // Lane name on separator
        let lane_info = state
            .session
            .automation
            .selected()
            .map(|l| {
                let enabled = if l.enabled { "\u{25cf}" } else { "\u{25cb}" };
                format!(" {} {} ", l.target.short_name(), enabled)
            })
            .unwrap_or_else(|| " (no lane) ".to_string());

        let title_style = Style::new().fg(if self.automation_focus {
            p.accent
        } else {
            p.dim
        });
        for (i, ch) in lane_info.chars().enumerate() {
            let x = rect.x + 1 + i as u16;
            if x >= rect.x + rect.width {
                break;
            }
            buf.set_cell(x, sep_y, ch, title_style);
        }

        // REC indicator on right side
        if state.recording.automation_recording {
            let rec_str = " REC ";
            let rec_style = Style::new().fg(p.fg).bg(p.recording);
            let rx = rect.x + rect.width - rec_str.len() as u16 - 1;
            for (i, ch) in rec_str.chars().enumerate() {
                buf.set_cell(rx + i as u16, sep_y, ch, rec_style);
            }
        }
    }

    /// Render the automation timeline in the bottom half of the split view
    pub(super) fn render_automation_timeline(
        &self,
        buf: &mut RenderBuf,
        area: Rect,
        state: &AppState,
    ) {
        let p = Palette::from(&state.session.theme);

        if area.height < 3 || area.width < 10 {
            return;
        }

        let automation = &state.session.automation;
        let lane = match automation.selected() {
            Some(l) => l,
            None => {
                let text = "(press 'a' to add a lane)";
                let style = Style::new().fg(p.muted);
                let x = area.x + (area.width.saturating_sub(text.len() as u16)) / 2;
                let y = area.y + area.height / 2;
                buf.draw_line(Rect::new(x, y, text.len() as u16, 1), &[(text, style)]);
                return;
            }
        };

        let tpc = self.ticks_per_cell();
        let graph_height = area.height.saturating_sub(2); // Reserve 1 for beat markers, 1 for status
        let graph_width = area.width;
        let graph_y = area.y;

        // Grid dots
        let bar_style = Style::new().fg(p.border);
        let bg_style = Style::new().fg(p.muted);
        for col in 0..graph_width {
            let tick = self.view_start_tick + col as u32 * tpc;
            let is_bar = tick.is_multiple_of(1920);
            let is_beat = tick.is_multiple_of(480);

            for row in 0..graph_height {
                let y = graph_y + row;
                let x = area.x + col;
                if is_bar {
                    buf.set_cell(x, y, '\u{250a}', bar_style);
                } else if is_beat && row == 0 {
                    buf.set_cell(x, y, '\u{00b7}', bg_style);
                }
            }
        }

        // Draw automation curve
        let curve_color = if lane.enabled { p.accent } else { p.muted };
        let curve_style = Style::new().fg(curve_color);
        let point_style = Style::new().fg(p.fg).bg(curve_color);

        if !lane.points.is_empty() && graph_height > 0 {
            for col in 0..graph_width {
                let tick = self.view_start_tick + col as u32 * tpc;
                if let Some(raw_value) = lane.value_at(tick) {
                    let normalized = if lane.max_value > lane.min_value {
                        (raw_value - lane.min_value) / (lane.max_value - lane.min_value)
                    } else {
                        0.5
                    };
                    let row = ((1.0 - normalized) * (graph_height.saturating_sub(1)) as f32) as u16;
                    let y = graph_y + row;
                    let x = area.x + col;
                    if y < graph_y + graph_height {
                        if lane.point_at(tick).is_some() {
                            buf.set_cell(x, y, '\u{25cf}', point_style);
                        } else {
                            buf.set_cell(x, y, '\u{2500}', curve_style);
                        }
                    }
                }
            }
        }

        // Draw cursor (only when automation has focus)
        if self.automation_focus {
            let cursor_col = if self.automation_cursor_tick >= self.view_start_tick {
                ((self.automation_cursor_tick - self.view_start_tick) / tpc) as u16
            } else {
                0
            };
            let cursor_row = ((1.0 - self.automation_cursor_value)
                * (graph_height.saturating_sub(1)) as f32) as u16;

            if cursor_col < graph_width {
                let x = area.x + cursor_col;

                for row in 0..graph_height {
                    let y = graph_y + row;
                    if let Some(cell) = buf.raw_buf().cell_mut((x, y)) {
                        if row == cursor_row {
                            cell.set_char('\u{25c6}')
                                .set_style(Style::new().fg(p.fg).bg(p.selection_bg));
                        } else if cell.symbol() == " " {
                            cell.set_char('\u{2502}').set_style(Style::new().fg(p.dim));
                        }
                    }
                }
            }
        }

        // Playhead
        if state.audio.playing {
            let playhead_col = if state.audio.playhead >= self.view_start_tick {
                ((state.audio.playhead - self.view_start_tick) / tpc) as u16
            } else {
                graph_width // off-screen
            };
            if playhead_col < graph_width {
                let x = area.x + playhead_col;
                for row in 0..graph_height {
                    let y = graph_y + row;
                    if let Some(cell) = buf.raw_buf().cell_mut((x, y)) {
                        if cell.symbol() == " " || cell.symbol() == "\u{250a}" {
                            cell.set_char('\u{2502}')
                                .set_style(Style::new().fg(p.playing));
                        }
                    }
                }
            }
        }

        // Beat markers row
        let marker_y = graph_y + graph_height;
        if marker_y < area.y + area.height {
            let marker_style = Style::new().fg(p.muted);
            for col in 0..graph_width {
                let tick = self.view_start_tick + col as u32 * tpc;
                if tick.is_multiple_of(1920) {
                    let bar = tick / 1920 + 1;
                    let label = format!("B{}", bar);
                    for (j, ch) in label.chars().enumerate() {
                        let x = area.x + col + j as u16;
                        if x < area.x + graph_width {
                            buf.set_cell(x, marker_y, ch, marker_style);
                        }
                    }
                }
            }
        }

        // Status line
        let status_y = graph_y + graph_height + 1;
        if status_y < area.y + area.height {
            let curve_at_cursor = lane
                .point_at(self.automation_cursor_tick)
                .map(|pt| match pt.curve {
                    CurveType::Linear => "Linear",
                    CurveType::Exponential => "Exp",
                    CurveType::Step => "Step",
                    CurveType::SCurve => "SCurve",
                })
                .unwrap_or("\u{2014}");

            let rec_indicator = if state.recording.automation_recording {
                " [REC]"
            } else {
                ""
            };
            let status = format!(
                " Tick:{:<6} Val:{:.2}  Curve:{}{}",
                self.automation_cursor_tick,
                self.automation_cursor_value,
                curve_at_cursor,
                rec_indicator,
            );

            let normal_style = Style::new().fg(p.dim);
            let rec_style = Style::new().fg(p.fg).bg(p.error);

            for (i, ch) in status.chars().enumerate() {
                let x = area.x + i as u16;
                if x >= area.x + graph_width {
                    break;
                }
                let is_rec_section = state.recording.automation_recording && i >= status.len() - 6;
                let style = if is_rec_section {
                    rec_style
                } else {
                    normal_style
                };
                buf.set_cell(x, status_y, ch, style);
            }
        }
    }

    /// Render the target picker overlay
    pub(super) fn render_target_picker(&self, buf: &mut RenderBuf, area: Rect, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        if let TargetPickerState::Active {
            ref options,
            cursor,
        } = self.target_picker
        {
            let picker_width = 40u16.min(area.width.saturating_sub(4));
            let picker_height = (options.len() as u16 + 2).min(area.height.saturating_sub(2));
            let picker_rect = center_rect(area, picker_width, picker_height);

            // Clear background
            let clear_style = Style::new().bg(p.bg);
            for y in picker_rect.y..picker_rect.y + picker_rect.height {
                for x in picker_rect.x..picker_rect.x + picker_rect.width {
                    buf.set_cell(x, y, ' ', clear_style);
                }
            }

            let border_style = Style::new().fg(p.accent);
            let title_style = Style::new().fg(p.accent);
            let inner = buf.draw_block(picker_rect, " Add Lane ", border_style, title_style);

            let inst = state.tracks.selected_track();
            let vst_registry = &state.session.vst_plugins;

            // Scroll offset: keep cursor visible
            let visible_rows = inner.height as usize;
            let scroll_offset = if cursor >= visible_rows {
                cursor - visible_rows + 1
            } else {
                0
            };

            for (vi, target) in options.iter().enumerate().skip(scroll_offset) {
                let row = vi - scroll_offset;
                let y = inner.y + row as u16;
                if y >= inner.y + inner.height {
                    break;
                }

                let is_selected = vi == cursor;
                let display_name = target.name_with_context(inst, vst_registry);
                let text = format!("{} {}", if is_selected { ">" } else { " " }, display_name);
                let style = if is_selected {
                    Style::new().fg(p.fg).bg(p.selection_bg).bold()
                } else {
                    Style::new().fg(p.dim)
                };

                for (j, ch) in text.chars().enumerate() {
                    let x = inner.x + j as u16;
                    if x >= inner.x + inner.width {
                        break;
                    }
                    buf.set_cell(x, y, ch, style);
                }
                if is_selected {
                    for x in (inner.x + text.len() as u16)..(inner.x + inner.width) {
                        buf.set_cell(x, y, ' ', style);
                    }
                }
            }
        }
    }
}
