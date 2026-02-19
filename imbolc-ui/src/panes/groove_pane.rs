use std::any::Any;

use crate::state::{AppState, SwingGrid, TrackId};
use crate::ui::action_id::{ActionId, GrooveActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::{
    Action, Color, InputEvent, KeyCode, Keymap, NavAction, Palette, Pane, PaneIdStr, Rect,
    RenderBuf, SequencerAction, Style, TrackAction,
};

/// Parameter indices for the groove pane
const PARAM_SWING: usize = 0;
const PARAM_SWING_GRID: usize = 1;
const PARAM_HUMANIZE_VEL: usize = 2;
const PARAM_HUMANIZE_TIME: usize = 3;
const PARAM_TIMING_OFFSET: usize = 4;
const PARAM_COUNT: usize = 5;

pub struct GroovePane {
    keymap: Keymap,
    selected_param: usize,
    /// When Some, we're editing a drum pad's groove instead of the track groove
    pad_context: Option<usize>,
}

impl GroovePane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            selected_param: 0,
            pad_context: None,
        }
    }
}

impl Default for GroovePane {
    fn default() -> Self {
        Self::new(Keymap::new())
    }
}

impl Pane for GroovePane {
    fn id(&self) -> PaneIdStr {
        PaneIdStr("groove")
    }

    fn on_enter(&mut self, state: &AppState) {
        // Check if we're entering in pad-groove mode
        self.pad_context = state
            .tracks
            .selected_drum_sequencer()
            .and_then(|seq| seq.groove_editing_pad);
    }

    fn handle_raw_input(&mut self, event: &InputEvent, _state: &AppState) -> Action {
        match event.key {
            KeyCode::Escape => Action::Nav(NavAction::PopPane),
            _ => Action::None,
        }
    }

    fn handle_action(&mut self, action: ActionId, _event: &InputEvent, state: &AppState) -> Action {
        let instrument = match state.tracks.selected_track() {
            Some(i) => i,
            None => return Action::None,
        };
        let instrument_id = instrument.id;

        // Resolve which groove config we're editing
        let groove = match self.pad_context {
            Some(pad_idx) => match state.tracks.selected_drum_sequencer() {
                Some(seq) => match seq.pads.get(pad_idx) {
                    Some(pad) => &pad.groove,
                    None => return Action::None,
                },
                None => return Action::None,
            },
            None => &instrument.groove,
        };

        match action {
            ActionId::Groove(GrooveActionId::PrevParam) => {
                self.selected_param = self.selected_param.saturating_sub(1);
                Action::None
            }
            ActionId::Groove(GrooveActionId::NextParam) => {
                self.selected_param = (self.selected_param + 1).min(PARAM_COUNT - 1);
                Action::None
            }
            ActionId::Groove(GrooveActionId::Increase)
            | ActionId::Groove(GrooveActionId::IncreaseBig)
            | ActionId::Groove(GrooveActionId::IncreaseTiny) => adjust_param(
                self.pad_context,
                instrument_id,
                groove,
                self.selected_param,
                true,
                action,
            ),
            ActionId::Groove(GrooveActionId::Decrease)
            | ActionId::Groove(GrooveActionId::DecreaseBig)
            | ActionId::Groove(GrooveActionId::DecreaseTiny) => adjust_param(
                self.pad_context,
                instrument_id,
                groove,
                self.selected_param,
                false,
                action,
            ),
            ActionId::Groove(GrooveActionId::CycleSwingGrid) => {
                let current = groove.swing_grid.unwrap_or(SwingGrid::Eighths);
                let next = current.next();
                match self.pad_context {
                    Some(pad_idx) => {
                        Action::Sequencer(SequencerAction::SetPadSwingGrid(pad_idx, Some(next)))
                    }
                    None => {
                        Action::Track(TrackAction::SetTrackSwingGrid(instrument_id, Some(next)))
                    }
                }
            }
            ActionId::Groove(GrooveActionId::NextTimeSig) => match self.pad_context {
                Some(pad_idx) => Action::Sequencer(SequencerAction::NextPadTimeSignature(pad_idx)),
                None => Action::Track(TrackAction::NextTrackTimeSignature(instrument_id)),
            },
            ActionId::Groove(GrooveActionId::Reset) => match self.pad_context {
                Some(pad_idx) => Action::Sequencer(SequencerAction::ResetPadGroove(pad_idx)),
                None => Action::Track(TrackAction::ResetTrackGroove(instrument_id)),
            },
            _ => Action::None,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        let rect = center_rect(area, 40, 12);

        let instrument = state.tracks.selected_track();

        // Build title based on pad or track context
        let title = match self.pad_context {
            Some(pad_idx) => {
                let pad_name = state
                    .tracks
                    .selected_drum_sequencer()
                    .and_then(|seq| seq.pads.get(pad_idx))
                    .map(|pad| pad.name.as_str())
                    .unwrap_or("?");
                format!(" Pad Groove: {} ", pad_name)
            }
            None => match instrument {
                Some(i) => format!(" Groove: {} ", i.name),
                None => " Groove: (none) ".to_string(),
            },
        };

        let border_style = Style::new().fg(p.accent_secondary);
        let inner = buf.draw_block(rect, &title, border_style, border_style);

        let instrument = match instrument {
            Some(i) => i,
            None => {
                render_centered_text(inner, buf, "(no track selected)", p.muted);
                return;
            }
        };

        // In pad mode, read the pad's groove; in track mode, read the track groove
        let groove = match self.pad_context {
            Some(pad_idx) => match state.tracks.selected_drum_sequencer() {
                Some(seq) => match seq.pads.get(pad_idx) {
                    Some(pad) => &pad.groove,
                    None => return,
                },
                None => return,
            },
            None => &instrument.groove,
        };

        // In pad mode, fallback values come from the track groove (which itself falls back to global)
        let track_groove = &instrument.groove;
        let global_swing = state.session.piano_roll.swing_amount;
        let global_grid = SwingGrid::Eighths; // Default global grid
        let global_humanize_vel = state.session.humanize.velocity;
        let global_humanize_time = state.session.humanize.timing;

        // For pad mode, the "global" fallback is the track's effective value
        let fallback_swing = match self.pad_context {
            Some(_) => track_groove.effective_swing(global_swing),
            None => global_swing,
        };
        let fallback_grid = match self.pad_context {
            Some(_) => track_groove.effective_swing_grid(global_grid),
            None => global_grid,
        };
        let fallback_humanize_vel = match self.pad_context {
            Some(_) => track_groove.effective_humanize_velocity(global_humanize_vel),
            None => global_humanize_vel,
        };
        let fallback_humanize_time = match self.pad_context {
            Some(_) => track_groove.effective_humanize_timing(global_humanize_time),
            None => global_humanize_time,
        };

        // Calculate effective values using appropriate fallbacks
        let swing = groove.effective_swing(fallback_swing);
        let swing_grid = groove.effective_swing_grid(fallback_grid);
        let humanize_vel = groove.effective_humanize_velocity(fallback_humanize_vel);
        let humanize_time = groove.effective_humanize_timing(fallback_humanize_time);
        let timing_offset = groove.timing_offset_ms;

        // Is using fallback?
        let swing_is_fallback = groove.swing_amount.is_none();
        let grid_is_fallback = groove.swing_grid.is_none();
        let hvel_is_fallback = groove.humanize_velocity.is_none();
        let htime_is_fallback = groove.humanize_timing.is_none();
        let fallback_label = if self.pad_context.is_some() {
            " (track)"
        } else {
            " (global)"
        };

        let y = inner.y + 1;
        let label_x = inner.x + 2;
        let value_x = inner.x + 18;

        let normal_style = Style::new().fg(p.fg);
        let global_style = Style::new().fg(p.dim);
        let selected_style = Style::new().fg(p.warning);

        // Swing amount
        render_param_row(
            buf,
            label_x,
            value_x,
            y,
            "Swing:",
            &format!("{:.0}%", swing * 100.0),
            swing_is_fallback,
            fallback_label,
            self.selected_param == PARAM_SWING,
            normal_style,
            global_style,
            selected_style,
        );

        // Swing grid
        render_param_row(
            buf,
            label_x,
            value_x,
            y + 1,
            "Swing Grid:",
            swing_grid.name(),
            grid_is_fallback,
            fallback_label,
            self.selected_param == PARAM_SWING_GRID,
            normal_style,
            global_style,
            selected_style,
        );

        // Humanize velocity
        render_param_row(
            buf,
            label_x,
            value_x,
            y + 2,
            "Humanize Vel:",
            &format!("{:.0}%", humanize_vel * 100.0),
            hvel_is_fallback,
            fallback_label,
            self.selected_param == PARAM_HUMANIZE_VEL,
            normal_style,
            global_style,
            selected_style,
        );

        // Humanize timing
        render_param_row(
            buf,
            label_x,
            value_x,
            y + 3,
            "Humanize Time:",
            &format!("{:.0}%", humanize_time * 100.0),
            htime_is_fallback,
            fallback_label,
            self.selected_param == PARAM_HUMANIZE_TIME,
            normal_style,
            global_style,
            selected_style,
        );

        // Timing offset
        let offset_str = if timing_offset >= 0.0 {
            format!("+{:.1}ms", timing_offset)
        } else {
            format!("{:.1}ms", timing_offset)
        };
        render_param_row(
            buf,
            label_x,
            value_x,
            y + 4,
            "Push/Pull:",
            &offset_str,
            false, // Timing offset has no fallback
            fallback_label,
            self.selected_param == PARAM_TIMING_OFFSET,
            normal_style,
            global_style,
            selected_style,
        );
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// -- Helpers --

fn render_centered_text(area: Rect, buf: &mut RenderBuf, text: &str, color: Color) {
    let x = area.x + (area.width.saturating_sub(text.len() as u16)) / 2;
    let y = area.y + area.height / 2;
    let style = Style::new().fg(color);
    buf.draw_line(Rect::new(x, y, text.len() as u16, 1), &[(text, style)]);
}

#[allow(clippy::too_many_arguments)]
fn render_param_row(
    buf: &mut RenderBuf,
    label_x: u16,
    value_x: u16,
    y: u16,
    label: &str,
    value: &str,
    is_fallback: bool,
    fallback_label: &str,
    is_selected: bool,
    normal_style: Style,
    global_style: Style,
    selected_style: Style,
) {
    let label_style = if is_selected {
        selected_style
    } else {
        normal_style
    };
    let value_style = if is_selected {
        selected_style
    } else if is_fallback {
        global_style
    } else {
        normal_style
    };

    // Render label
    for (i, ch) in label.chars().enumerate() {
        buf.set_cell(label_x + i as u16, y, ch, label_style);
    }

    // Render value
    for (i, ch) in value.chars().enumerate() {
        buf.set_cell(value_x + i as u16, y, ch, value_style);
    }

    // Render fallback suffix if using fallback value
    if is_fallback && !is_selected {
        for (i, ch) in fallback_label.chars().enumerate() {
            buf.set_cell(value_x + value.len() as u16 + i as u16, y, ch, global_style);
        }
    }
}

fn adjust_param(
    pad_context: Option<usize>,
    instrument_id: TrackId,
    groove: &crate::state::GrooveConfig,
    param_idx: usize,
    increase: bool,
    action: ActionId,
) -> Action {
    match param_idx {
        PARAM_SWING => {
            let delta = match action {
                ActionId::Groove(GrooveActionId::IncreaseBig)
                | ActionId::Groove(GrooveActionId::DecreaseBig) => 0.1,
                ActionId::Groove(GrooveActionId::IncreaseTiny)
                | ActionId::Groove(GrooveActionId::DecreaseTiny) => 0.01,
                _ => 0.05,
            };
            let signed_delta = if increase { delta } else { -delta };
            match pad_context {
                Some(pad_idx) => {
                    Action::Sequencer(SequencerAction::AdjustPadSwing(pad_idx, signed_delta))
                }
                None => Action::Track(TrackAction::AdjustTrackSwing(instrument_id, signed_delta)),
            }
        }
        PARAM_SWING_GRID => {
            let current = groove.swing_grid.unwrap_or(SwingGrid::Eighths);
            let next = if increase {
                current.next()
            } else {
                cycle_swing_grid_rev(current)
            };
            match pad_context {
                Some(pad_idx) => {
                    Action::Sequencer(SequencerAction::SetPadSwingGrid(pad_idx, Some(next)))
                }
                None => Action::Track(TrackAction::SetTrackSwingGrid(instrument_id, Some(next))),
            }
        }
        PARAM_HUMANIZE_VEL => {
            let delta = match action {
                ActionId::Groove(GrooveActionId::IncreaseBig)
                | ActionId::Groove(GrooveActionId::DecreaseBig) => 0.1,
                ActionId::Groove(GrooveActionId::IncreaseTiny)
                | ActionId::Groove(GrooveActionId::DecreaseTiny) => 0.01,
                _ => 0.05,
            };
            let signed_delta = if increase { delta } else { -delta };
            match pad_context {
                Some(pad_idx) => Action::Sequencer(SequencerAction::AdjustPadHumanizeVelocity(
                    pad_idx,
                    signed_delta,
                )),
                None => Action::Track(TrackAction::AdjustTrackHumanizeVelocity(
                    instrument_id,
                    signed_delta,
                )),
            }
        }
        PARAM_HUMANIZE_TIME => {
            let delta = match action {
                ActionId::Groove(GrooveActionId::IncreaseBig)
                | ActionId::Groove(GrooveActionId::DecreaseBig) => 0.1,
                ActionId::Groove(GrooveActionId::IncreaseTiny)
                | ActionId::Groove(GrooveActionId::DecreaseTiny) => 0.01,
                _ => 0.05,
            };
            let signed_delta = if increase { delta } else { -delta };
            match pad_context {
                Some(pad_idx) => Action::Sequencer(SequencerAction::AdjustPadHumanizeTiming(
                    pad_idx,
                    signed_delta,
                )),
                None => Action::Track(TrackAction::AdjustTrackHumanizeTiming(
                    instrument_id,
                    signed_delta,
                )),
            }
        }
        PARAM_TIMING_OFFSET => {
            let delta = match action {
                ActionId::Groove(GrooveActionId::IncreaseBig)
                | ActionId::Groove(GrooveActionId::DecreaseBig) => 5.0,
                ActionId::Groove(GrooveActionId::IncreaseTiny)
                | ActionId::Groove(GrooveActionId::DecreaseTiny) => 0.5,
                _ => 1.0,
            };
            let signed_delta = if increase { delta } else { -delta };
            match pad_context {
                Some(pad_idx) => Action::Sequencer(SequencerAction::AdjustPadTimingOffset(
                    pad_idx,
                    signed_delta,
                )),
                None => Action::Track(TrackAction::AdjustTrackTimingOffset(
                    instrument_id,
                    signed_delta,
                )),
            }
        }
        _ => Action::None,
    }
}

fn cycle_swing_grid_rev(grid: SwingGrid) -> SwingGrid {
    match grid {
        SwingGrid::Eighths => SwingGrid::Both,
        SwingGrid::Sixteenths => SwingGrid::Eighths,
        SwingGrid::Both => SwingGrid::Sixteenths,
    }
}
