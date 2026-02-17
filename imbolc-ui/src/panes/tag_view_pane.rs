use std::any::Any;

use crate::state::AppState;
use crate::ui::action_id::{ActionId, TagViewActionId};
use crate::ui::pane::*;
use crate::ui::style::{Color, Style};
use crate::ui::{InputEvent, Keymap, MouseEvent, Rect, RenderBuf};
use imbolc_types::{
    AutomationTarget, BusParameter, GlobalParameter, InstrumentParameter, ParameterTarget,
    TagAction,
};

/// Pane showing tagged parameters as adjustable sliders.
pub struct TagViewPane {
    keymap: Keymap,
    selected_row: usize,
    scroll_offset: usize,
}

impl TagViewPane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            selected_row: 0,
            scroll_offset: 0,
        }
    }

    fn target_count(&self, state: &AppState) -> usize {
        state
            .session
            .param_tags
            .selected()
            .map(|t| t.targets.len())
            .unwrap_or(0)
    }

    fn current_target<'a>(&self, state: &'a AppState) -> Option<&'a AutomationTarget> {
        state
            .session
            .param_tags
            .selected()
            .and_then(|t| t.targets.get(self.selected_row))
    }

    fn adjust_current(&self, state: &AppState, delta: f32) -> Action {
        if let Some(target) = self.current_target(state) {
            if let Some(action) = action_for_target_adjust(target, delta) {
                return action;
            }
        }
        Action::None
    }
}

impl Pane for TagViewPane {
    fn id(&self) -> &'static str {
        PaneId::TagView.as_str()
    }

    fn handle_action(&mut self, action: ActionId, _event: &InputEvent, state: &AppState) -> Action {
        let count = self.target_count(state);
        match action {
            ActionId::TagView(TagViewActionId::NextParam) => {
                if count > 0 {
                    self.selected_row = (self.selected_row + 1).min(count - 1);
                }
                Action::None
            }
            ActionId::TagView(TagViewActionId::PrevParam) => {
                self.selected_row = self.selected_row.saturating_sub(1);
                Action::None
            }
            ActionId::TagView(TagViewActionId::Increase) => self.adjust_current(state, 0.02),
            ActionId::TagView(TagViewActionId::Decrease) => self.adjust_current(state, -0.02),
            ActionId::TagView(TagViewActionId::IncreaseBig) => self.adjust_current(state, 0.1),
            ActionId::TagView(TagViewActionId::DecreaseBig) => self.adjust_current(state, -0.1),
            ActionId::TagView(TagViewActionId::IncreaseTiny) => self.adjust_current(state, 0.005),
            ActionId::TagView(TagViewActionId::DecreaseTiny) => self.adjust_current(state, -0.005),
            ActionId::TagView(TagViewActionId::NextTag) => {
                let tags = &state.session.param_tags.tags;
                if !tags.is_empty() {
                    let current = state.session.param_tags.selected_tag.unwrap_or(0);
                    let next = (current + 1) % tags.len();
                    self.selected_row = 0;
                    self.scroll_offset = 0;
                    Action::Tag(TagAction::SelectTag(Some(next)))
                } else {
                    Action::None
                }
            }
            ActionId::TagView(TagViewActionId::PrevTag) => {
                let tags = &state.session.param_tags.tags;
                if !tags.is_empty() {
                    let current = state.session.param_tags.selected_tag.unwrap_or(0);
                    let prev = if current == 0 {
                        tags.len() - 1
                    } else {
                        current - 1
                    };
                    self.selected_row = 0;
                    self.scroll_offset = 0;
                    Action::Tag(TagAction::SelectTag(Some(prev)))
                } else {
                    Action::None
                }
            }
            ActionId::TagView(TagViewActionId::RemoveTarget) => {
                if let (Some(tag_idx), Some(target)) = (
                    state.session.param_tags.selected_tag,
                    self.current_target(state).cloned(),
                ) {
                    if self.selected_row > 0 && self.selected_row >= count - 1 {
                        self.selected_row = self.selected_row.saturating_sub(1);
                    }
                    Action::Tag(TagAction::RemoveTarget(tag_idx, target))
                } else {
                    Action::None
                }
            }
            ActionId::TagView(TagViewActionId::DeleteTag) => {
                if let Some(tag_idx) = state.session.param_tags.selected_tag {
                    self.selected_row = 0;
                    self.scroll_offset = 0;
                    Action::Tag(TagAction::DeleteTag(tag_idx))
                } else {
                    Action::None
                }
            }
            ActionId::TagView(TagViewActionId::CreateTag) => {
                let name = format!("Tag {}", state.session.param_tags.tags.len() + 1);
                Action::Tag(TagAction::CreateTag(name))
            }
            ActionId::TagView(TagViewActionId::RenameTag) => {
                // TODO: push text input layer for rename
                Action::None
            }
            _ => Action::None,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let tags = &state.session.param_tags;

        // Header: tag tabs
        let header_y = area.y;
        let dim = Style::new().fg(Color::new(100, 100, 100));
        let active_tab = Style::new().fg(Color::WHITE).bg(Color::new(60, 60, 80));
        let inactive_tab = Style::new().fg(Color::new(160, 160, 160));

        if tags.tags.is_empty() {
            let msg = "No tags. Press 'T' in instrument edit to tag a parameter.";
            let x = area.x + (area.width.saturating_sub(msg.len() as u16)) / 2;
            let y = area.y + area.height / 2;
            buf.draw_str(x, y, msg, dim);

            let hint = "Press 'n' to create a new tag.";
            let hx = area.x + (area.width.saturating_sub(hint.len() as u16)) / 2;
            buf.draw_str(hx, y + 1, hint, dim);
            return;
        }

        // Draw tag tabs
        let mut tx = area.x + 1;
        for (i, tag) in tags.tags.iter().enumerate() {
            let style = if tags.selected_tag == Some(i) {
                active_tab
            } else {
                inactive_tab
            };
            let label = format!(" {} ", tag.name);
            if tx + label.len() as u16 >= area.x + area.width - 1 {
                break;
            }
            buf.draw_str(tx, header_y, &label, style);
            tx += label.len() as u16 + 1;
        }

        // Separator
        let sep_y = header_y + 1;
        for x in area.x..area.x + area.width {
            buf.draw_str(x, sep_y, "\u{2500}", dim);
        }

        // Content area
        let content_y = sep_y + 1;
        let content_height = area.height.saturating_sub(3) as usize;

        let selected_tag = match tags.selected() {
            Some(t) => t,
            None => return,
        };

        if selected_tag.targets.is_empty() {
            let msg = "No parameters tagged. Press 'T' on a param in instrument edit.";
            let x = area.x + (area.width.saturating_sub(msg.len() as u16)) / 2;
            buf.draw_str(x, content_y + 1, msg, dim);
            return;
        }

        // Ensure scroll keeps selection visible
        if self.selected_row < self.scroll_offset {
            self.scroll_offset = self.selected_row;
        }
        if self.selected_row >= self.scroll_offset + content_height {
            self.scroll_offset = self.selected_row + 1 - content_height;
        }

        // Column layout: [inst_name 12] [param_name 16] [slider bar] [value 8] [CC 4]
        let inst_col_w = 12u16;
        let param_col_w = 16u16;
        let value_col_w = 8u16;
        let cc_col_w = 5u16;
        let bar_w = area
            .width
            .saturating_sub(inst_col_w + param_col_w + value_col_w + cc_col_w + 6);

        for (vi, target) in selected_tag
            .targets
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(content_height)
        {
            let y = content_y + (vi - self.scroll_offset) as u16;
            let is_selected = vi == self.selected_row;

            let row_style = if is_selected {
                Style::new().fg(Color::WHITE).bg(Color::SELECTION_BG)
            } else {
                Style::new().fg(Color::new(200, 200, 200))
            };

            // Clear row
            let blank = " ".repeat(area.width as usize);
            buf.draw_str(area.x, y, &blank, row_style);

            // Track name
            let inst_name = target_instrument_name(target, state);
            let inst_truncated: String = inst_name.chars().take(inst_col_w as usize).collect();
            let inst_style = if is_selected {
                Style::new()
                    .fg(Color::new(150, 200, 255))
                    .bg(Color::SELECTION_BG)
            } else {
                Style::new().fg(Color::new(100, 150, 200))
            };
            buf.draw_str(area.x + 1, y, &inst_truncated, inst_style);

            // Param name
            let param_name = target_param_name(target);
            let param_truncated: String = param_name.chars().take(param_col_w as usize).collect();
            buf.draw_str(area.x + 1 + inst_col_w + 1, y, &param_truncated, row_style);

            // Value and slider
            if let Some((value, min, max)) = read_target_value(target, state) {
                let normalized = if (max - min).abs() > f32::EPSILON {
                    ((value - min) / (max - min)).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                // Slider bar
                let bar_x = area.x + 1 + inst_col_w + 1 + param_col_w + 1;
                draw_slider(buf, bar_x, y, bar_w, normalized, is_selected);

                // Value text
                let val_str = format_value(value);
                let val_x = bar_x + bar_w + 1;
                buf.draw_str(val_x, y, &val_str, row_style);

                // CC badge
                let cc = find_cc_for_target(target, state);
                if let Some(cc_num) = cc {
                    let cc_str = format!("CC{}", cc_num);
                    let cc_x = val_x + value_col_w;
                    let cc_style = if is_selected {
                        Style::new().fg(Color::MIDI_COLOR).bg(Color::SELECTION_BG)
                    } else {
                        Style::new().fg(Color::MIDI_COLOR)
                    };
                    buf.draw_str(cc_x, y, &cc_str, cc_style);
                }
            }
        }

        // Footer with hint
        let footer_y = area.y + area.height - 1;
        let hint =
            "Tab/S-Tab: tags  j/k: nav  Left/Right: adjust  d: remove  n: new  x: delete tag";
        let hint_truncated: String = hint.chars().take(area.width as usize).collect();
        buf.draw_str(area.x, footer_y, &hint_truncated, dim);
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn on_enter(&mut self, _state: &AppState) {
        self.selected_row = 0;
        self.scroll_offset = 0;
    }

    fn tick(&mut self, state: &AppState) -> Vec<Action> {
        // Auto-select first tag if none selected
        if state.session.param_tags.selected_tag.is_none()
            && !state.session.param_tags.tags.is_empty()
        {
            return vec![Action::Tag(TagAction::SelectTag(Some(0)))];
        }
        vec![]
    }

    fn handle_mouse(&mut self, _event: &MouseEvent, _area: Rect, _state: &AppState) -> Action {
        Action::None
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn draw_slider(buf: &mut RenderBuf, x: u16, y: u16, width: u16, normalized: f32, selected: bool) {
    let filled = (normalized * width as f32) as u16;
    let bar_fg = if selected {
        Color::new(100, 200, 255)
    } else {
        Color::new(80, 160, 200)
    };
    let bar_bg_color = if selected {
        Color::SELECTION_BG
    } else {
        Color::new(40, 40, 40)
    };

    for i in 0..width {
        let ch = if i < filled { "\u{2588}" } else { "\u{2591}" };
        let style = if i < filled {
            Style::new().fg(bar_fg).bg(bar_bg_color)
        } else {
            Style::new().fg(Color::new(60, 60, 60)).bg(bar_bg_color)
        };
        buf.draw_str(x + i, y, ch, style);
    }
}

fn format_value(v: f32) -> String {
    if v.abs() >= 1000.0 {
        format!("{:.0}", v)
    } else if v.abs() >= 100.0 {
        format!("{:.1}", v)
    } else if v.abs() >= 10.0 {
        format!("{:.2}", v)
    } else {
        format!("{:.3}", v)
    }
}

fn target_instrument_name(target: &AutomationTarget, state: &AppState) -> String {
    match target {
        AutomationTarget::Track(id, _) => state
            .tracks
            .track(*id)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| format!("#{}", id.get())),
        AutomationTarget::Bus(id, _) => format!("Bus {}", id.get()),
        AutomationTarget::Global(_) => "Global".to_string(),
        AutomationTarget::Generative(_) => "Gen".to_string(),
    }
}

fn target_param_name(target: &AutomationTarget) -> String {
    match target {
        AutomationTarget::Track(_, InstrumentParameter::Standard(param)) => {
            param.name().to_string()
        }
        AutomationTarget::Bus(_, BusParameter::Level) => "Level".to_string(),
        AutomationTarget::Global(GlobalParameter::Bpm) => "BPM".to_string(),
        AutomationTarget::Global(GlobalParameter::TimeSignature) => "Time Sig".to_string(),
        AutomationTarget::Generative(param) => format!("{:?}", param),
    }
}

/// Read the current value of an automation target from state.
/// Returns (value, min, max) or None if target is invalid.
fn read_target_value(target: &AutomationTarget, state: &AppState) -> Option<(f32, f32, f32)> {
    let (min, max) = target.default_range();
    let value = match target {
        AutomationTarget::Track(id, InstrumentParameter::Standard(param)) => {
            let inst = state.tracks.track(*id)?;
            match param {
                ParameterTarget::Level => inst.channel_strip.level,
                ParameterTarget::Pan => inst.channel_strip.pan,
                ParameterTarget::FilterCutoff => inst.filter()?.cutoff.value,
                ParameterTarget::FilterResonance => inst.filter()?.resonance.value,
                ParameterTarget::Attack => inst.modulation.amp_envelope.attack,
                ParameterTarget::Decay => inst.modulation.amp_envelope.decay,
                ParameterTarget::Sustain => inst.modulation.amp_envelope.sustain,
                ParameterTarget::Release => inst.modulation.amp_envelope.release,
                ParameterTarget::LfoRate => inst.modulation.lfo.rate,
                ParameterTarget::LfoDepth => inst.modulation.lfo.depth,
                ParameterTarget::Pitch => 0.0,
                ParameterTarget::Swing => inst.groove.swing_amount.unwrap_or(0.0),
                ParameterTarget::HumanizeVelocity => inst.groove.humanize_velocity.unwrap_or(0.0),
                ParameterTarget::HumanizeTiming => inst.groove.humanize_timing.unwrap_or(0.0),
                ParameterTarget::TimingOffset => inst.groove.timing_offset_ms,
                ParameterTarget::EffectParam(eid, pidx) => {
                    let effect = inst.effects().find(|e| e.id == *eid)?;
                    let p = effect.params.get(pidx.get())?;
                    p.value.to_f32()
                }
                ParameterTarget::SendLevel(bus_id) => {
                    inst.channel_strip.sends.get(bus_id).map(|s| s.level)?
                }
                ParameterTarget::VstParam(idx) => inst
                    .vst_source_params()
                    .iter()
                    .find(|(i, _)| *i == *idx)
                    .map(|(_, v)| *v)?,
                ParameterTarget::SourceParam(idx) => {
                    let p = inst.source_params.get(idx.get())?;
                    p.value.to_f32()
                }
                _ => return None,
            }
        }
        AutomationTarget::Bus(id, BusParameter::Level) => {
            state.session.mixer.bus(*id)?.channel_strip.level
        }
        AutomationTarget::Global(GlobalParameter::Bpm) => state.session.bpm as f32,
        AutomationTarget::Global(GlobalParameter::TimeSignature) => return None,
        AutomationTarget::Generative(_) => return None,
    };
    Some((value, min, max))
}

/// Generate an Action to adjust a target by delta.
/// Delta is in the natural unit of the parameter (not normalized).
fn action_for_target_adjust(target: &AutomationTarget, delta: f32) -> Option<Action> {
    use imbolc_types::InstrumentAction;
    match target {
        AutomationTarget::Track(id, InstrumentParameter::Standard(param)) => {
            let id = *id;
            let action = match param {
                ParameterTarget::FilterCutoff => {
                    let (min, max) = target.default_range();
                    InstrumentAction::AdjustFilterCutoff(id, delta * (max - min))
                }
                ParameterTarget::FilterResonance => {
                    let (min, max) = target.default_range();
                    InstrumentAction::AdjustFilterResonance(id, delta * (max - min))
                }
                ParameterTarget::Attack => {
                    let (min, max) = target.default_range();
                    InstrumentAction::AdjustEnvelopeAttack(id, delta * (max - min))
                }
                ParameterTarget::Decay => {
                    let (min, max) = target.default_range();
                    InstrumentAction::AdjustEnvelopeDecay(id, delta * (max - min))
                }
                ParameterTarget::Sustain => InstrumentAction::AdjustEnvelopeSustain(id, delta),
                ParameterTarget::Release => {
                    let (min, max) = target.default_range();
                    InstrumentAction::AdjustEnvelopeRelease(id, delta * (max - min))
                }
                ParameterTarget::LfoRate => {
                    let (min, max) = target.default_range();
                    InstrumentAction::AdjustLfoRate(id, delta * (max - min))
                }
                ParameterTarget::LfoDepth => InstrumentAction::AdjustLfoDepth(id, delta),
                ParameterTarget::EffectParam(eid, pidx) => {
                    InstrumentAction::AdjustEffectParam(id, *eid, *pidx, delta)
                }
                ParameterTarget::Swing => InstrumentAction::AdjustTrackSwing(id, delta),
                ParameterTarget::HumanizeVelocity => {
                    InstrumentAction::AdjustTrackHumanizeVelocity(id, delta)
                }
                ParameterTarget::HumanizeTiming => {
                    InstrumentAction::AdjustTrackHumanizeTiming(id, delta)
                }
                ParameterTarget::TimingOffset => {
                    InstrumentAction::AdjustTrackTimingOffset(id, delta)
                }
                // Level/Pan: no per-instrument adjust action available
                // VstParam, SourceParam, SendLevel: no direct adjust action
                _ => return None,
            };
            Some(Action::Track(action))
        }
        // Bus level, Global BPM: no direct delta actions exposed to UI
        _ => None,
    }
}

/// Find the MIDI CC number mapped to a target (if any).
fn find_cc_for_target(target: &AutomationTarget, state: &AppState) -> Option<u8> {
    state
        .session
        .midi_recording
        .cc_mappings
        .iter()
        .find(|m| &m.target == target)
        .map(|m| m.cc_number)
}
