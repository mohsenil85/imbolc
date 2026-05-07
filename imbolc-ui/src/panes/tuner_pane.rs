use std::any::Any;

use crate::action::TunerAction;
use crate::state::AppState;
use crate::ui::action_id::{ActionId, TunerActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::{Action, InputEvent, Keymap, Palette, Pane, PaneIdStr, Rect, RenderBuf, Style};

// ── Track presets ──────────────────────────────────────────────────

struct TunerPreset {
    name: &'static str,
    strings: &'static [(&'static str, u8)], // (note_name, midi_note) high→low
}

const PRESETS: &[TunerPreset] = &[
    TunerPreset {
        name: "Guitar",
        strings: &[
            ("E4", 64),
            ("B3", 59),
            ("G3", 55),
            ("D3", 50),
            ("A2", 45),
            ("E2", 40),
        ],
    },
    TunerPreset {
        name: "Bass",
        strings: &[("G2", 43), ("D2", 38), ("A1", 33), ("E1", 28)],
    },
    TunerPreset {
        name: "Ukulele",
        strings: &[("A4", 69), ("E4", 64), ("C4", 60), ("G4", 67)],
    },
    TunerPreset {
        name: "Guitulele",
        strings: &[
            ("A4", 69),
            ("E4", 64),
            ("C4", 60),
            ("G3", 55),
            ("D3", 50),
            ("A2", 45),
        ],
    },
];

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

fn midi_to_note_name(midi: u8) -> String {
    let note = (midi % 12) as usize;
    let octave = (midi as i32 / 12) - 1;
    format!("{}{}", NOTE_NAMES[note], octave)
}

// ── Pane state ──────────────────────────────────────────────────────────

pub struct TunerPane {
    keymap: Keymap,
    instrument_idx: usize,
    string_idx: usize,
    playing: bool,
    full_step_down: bool,
    bright: bool,
}

impl TunerPane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            instrument_idx: 0,
            string_idx: 0,
            playing: false,
            full_step_down: false,
            bright: false,
        }
    }

    fn preset(&self) -> &TunerPreset {
        &PRESETS[self.instrument_idx]
    }

    fn effective_midi(&self, midi: u8) -> u8 {
        if self.full_step_down {
            midi.saturating_sub(2)
        } else {
            midi
        }
    }

    fn play_current(&self, state: &AppState) -> Action {
        let midi = self.effective_midi(self.preset().strings[self.string_idx].1);
        let freq = Self::midi_to_freq(midi, state.session.tuning_a4);
        Action::Tuner(TunerAction::PlayTone(freq, self.bright))
    }

    fn midi_to_freq(midi: u8, tuning_a4: f32) -> f32 {
        tuning_a4 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0)
    }
}

impl Default for TunerPane {
    fn default() -> Self {
        Self::new(Keymap::new())
    }
}

impl Pane for TunerPane {
    fn id(&self) -> PaneIdStr {
        PaneIdStr("tuner")
    }

    fn handle_action(&mut self, action: ActionId, _event: &InputEvent, state: &AppState) -> Action {
        let ActionId::Tuner(a) = action else {
            return Action::None;
        };

        match a {
            TunerActionId::PrevInstrument => {
                let was_playing = self.playing;
                if self.instrument_idx == 0 {
                    self.instrument_idx = PRESETS.len() - 1;
                } else {
                    self.instrument_idx -= 1;
                }
                self.string_idx = 0;
                self.playing = false;
                if was_playing {
                    Action::Tuner(TunerAction::StopTone)
                } else {
                    Action::None
                }
            }
            TunerActionId::NextInstrument => {
                let was_playing = self.playing;
                self.instrument_idx = (self.instrument_idx + 1) % PRESETS.len();
                self.string_idx = 0;
                self.playing = false;
                if was_playing {
                    Action::Tuner(TunerAction::StopTone)
                } else {
                    Action::None
                }
            }
            TunerActionId::PrevString => {
                let count = self.preset().strings.len();
                if self.string_idx == 0 {
                    self.string_idx = count - 1;
                } else {
                    self.string_idx -= 1;
                }
                if self.playing {
                    return self.play_current(state);
                }
                Action::None
            }
            TunerActionId::NextString => {
                let count = self.preset().strings.len();
                self.string_idx = (self.string_idx + 1) % count;
                if self.playing {
                    return self.play_current(state);
                }
                Action::None
            }
            TunerActionId::TogglePlayback => {
                if self.playing {
                    self.playing = false;
                    Action::Tuner(TunerAction::StopTone)
                } else {
                    self.playing = true;
                    self.play_current(state)
                }
            }
            TunerActionId::ToggleFullStepDown => {
                self.full_step_down = !self.full_step_down;
                if self.playing {
                    self.play_current(state)
                } else {
                    Action::None
                }
            }
            TunerActionId::ToggleBright => {
                self.bright = !self.bright;
                if self.playing {
                    self.play_current(state)
                } else {
                    Action::None
                }
            }
        }
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        let preset = self.preset();
        let extra_lines = if self.full_step_down { 1 } else { 0 };
        let height = (preset.strings.len() as u16) + 6 + extra_lines;
        let width = 44;
        let inner = center_rect(area, width, height);

        // Background
        let bg = p.bg;
        for y in inner.y..inner.y + inner.height {
            for x in inner.x..inner.x + inner.width {
                buf.set_cell(x, y, ' ', Style::new().bg(bg));
            }
        }

        let mut y = inner.y;

        // Title
        let title = if self.bright {
            "Reference Tuner [bright]"
        } else {
            "Reference Tuner"
        };
        let tx = inner.x + (inner.width.saturating_sub(title.len() as u16)) / 2;
        buf.draw_str(tx, y, title, Style::new().fg(p.fg).bg(bg));
        y += 2;

        // Track selector
        let inst_line = format!("<  {}  >", preset.name);
        let ix = inner.x + (inner.width.saturating_sub(inst_line.len() as u16)) / 2;
        buf.draw_str(
            ix,
            y,
            &inst_line,
            Style::new().fg(p.accent_secondary).bg(bg),
        );
        y += 1;

        // Full step down indicator
        if self.full_step_down {
            let label = "Full Step Down";
            let lx = inner.x + (inner.width.saturating_sub(label.len() as u16)) / 2;
            buf.draw_str(lx, y, label, Style::new().fg(p.accent).bg(bg));
            y += 1;
        }

        // A4 tuning value
        let a4_line = format!("A4 = {:.1} Hz", state.session.tuning_a4);
        let ax = inner.x + (inner.width.saturating_sub(a4_line.len() as u16)) / 2;
        buf.draw_str(ax, y, &a4_line, Style::new().fg(p.dim).bg(bg));
        y += 2;

        // String list
        for (i, (_note_name, midi)) in preset.strings.iter().enumerate() {
            let effective = self.effective_midi(*midi);
            let note_name = midi_to_note_name(effective);
            let freq = Self::midi_to_freq(effective, state.session.tuning_a4);
            let is_selected = i == self.string_idx;

            let marker = if is_selected && self.playing {
                ">>"
            } else if is_selected {
                " >"
            } else {
                "  "
            };

            let line = format!("{} {:>3}  {:>8.2} Hz", marker, note_name, freq);
            let sx = inner.x + (inner.width.saturating_sub(line.len() as u16)) / 2;

            let style = if is_selected {
                Style::new().fg(p.accent).bg(bg)
            } else {
                Style::new().fg(p.dim).bg(bg)
            };

            buf.draw_str(sx, y, &line, style);
            y += 1;
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn on_exit(&mut self, _state: &AppState) {
        self.playing = false;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::input::KeyCode;

    fn make_state() -> AppState {
        AppState::new()
    }

    fn make_event() -> InputEvent {
        InputEvent::new(KeyCode::Enter, crate::ui::input::Modifiers::none())
    }

    #[test]
    fn test_play_stop_toggle() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        assert!(!pane.playing);

        let action = pane.handle_action(
            ActionId::Tuner(TunerActionId::TogglePlayback),
            &event,
            &state,
        );
        assert!(pane.playing);
        assert!(matches!(
            action,
            Action::Tuner(TunerAction::PlayTone(_, false))
        ));

        let action = pane.handle_action(
            ActionId::Tuner(TunerActionId::TogglePlayback),
            &event,
            &state,
        );
        assert!(!pane.playing);
        assert!(matches!(action, Action::Tuner(TunerAction::StopTone)));
    }

    #[test]
    fn test_next_instrument_wraps() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        assert_eq!(pane.instrument_idx, 0);
        pane.handle_action(
            ActionId::Tuner(TunerActionId::NextInstrument),
            &event,
            &state,
        );
        assert_eq!(pane.instrument_idx, 1);
        pane.handle_action(
            ActionId::Tuner(TunerActionId::NextInstrument),
            &event,
            &state,
        );
        assert_eq!(pane.instrument_idx, 2);
        pane.handle_action(
            ActionId::Tuner(TunerActionId::NextInstrument),
            &event,
            &state,
        );
        assert_eq!(pane.instrument_idx, 3);
        pane.handle_action(
            ActionId::Tuner(TunerActionId::NextInstrument),
            &event,
            &state,
        );
        assert_eq!(pane.instrument_idx, 0);
    }

    #[test]
    fn test_prev_instrument_wraps() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        assert_eq!(pane.instrument_idx, 0);
        pane.handle_action(
            ActionId::Tuner(TunerActionId::PrevInstrument),
            &event,
            &state,
        );
        assert_eq!(pane.instrument_idx, PRESETS.len() - 1);
    }

    #[test]
    fn test_string_navigation() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        assert_eq!(pane.string_idx, 0);
        pane.handle_action(ActionId::Tuner(TunerActionId::NextString), &event, &state);
        assert_eq!(pane.string_idx, 1);
        pane.handle_action(ActionId::Tuner(TunerActionId::PrevString), &event, &state);
        assert_eq!(pane.string_idx, 0);
        pane.handle_action(ActionId::Tuner(TunerActionId::PrevString), &event, &state);
        assert_eq!(pane.string_idx, 5);
    }

    #[test]
    fn test_instrument_switch_resets_string_and_stops() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        pane.string_idx = 3;
        pane.playing = true;
        let action = pane.handle_action(
            ActionId::Tuner(TunerActionId::NextInstrument),
            &event,
            &state,
        );
        assert_eq!(pane.string_idx, 0);
        assert!(!pane.playing);
        assert!(matches!(action, Action::Tuner(TunerAction::StopTone)));
    }

    #[test]
    fn test_on_exit_resets_playing() {
        let mut pane = TunerPane::default();
        let state = make_state();
        pane.playing = true;
        pane.on_exit(&state);
        assert!(!pane.playing);
    }

    #[test]
    fn test_midi_to_freq() {
        let freq = TunerPane::midi_to_freq(69, 432.0);
        assert!((freq - 432.0).abs() < 0.01);

        let freq = TunerPane::midi_to_freq(69, 440.0);
        assert!((freq - 440.0).abs() < 0.01);

        let freq = TunerPane::midi_to_freq(40, 432.0);
        let expected = 432.0 * 2.0_f32.powf((40.0 - 69.0) / 12.0);
        assert!((freq - expected).abs() < 0.01);
    }

    #[test]
    fn test_string_change_updates_freq_when_playing() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        pane.handle_action(
            ActionId::Tuner(TunerActionId::TogglePlayback),
            &event,
            &state,
        );
        assert!(pane.playing);

        let action = pane.handle_action(ActionId::Tuner(TunerActionId::NextString), &event, &state);
        assert!(matches!(action, Action::Tuner(TunerAction::PlayTone(_, _))));
    }

    #[test]
    fn test_full_step_down_toggle() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        assert!(!pane.full_step_down);
        pane.handle_action(
            ActionId::Tuner(TunerActionId::ToggleFullStepDown),
            &event,
            &state,
        );
        assert!(pane.full_step_down);
        pane.handle_action(
            ActionId::Tuner(TunerActionId::ToggleFullStepDown),
            &event,
            &state,
        );
        assert!(!pane.full_step_down);
    }

    #[test]
    fn test_full_step_down_lowers_frequency() {
        let tuning_a4 = 440.0;
        let midi_e4: u8 = 64;

        let freq_standard = TunerPane::midi_to_freq(midi_e4, tuning_a4);
        let freq_dropped = TunerPane::midi_to_freq(midi_e4 - 2, tuning_a4);

        assert!(freq_dropped < freq_standard);
        let expected = tuning_a4 * 2.0_f32.powf((62.0 - 69.0) / 12.0);
        assert!((freq_dropped - expected).abs() < 0.01);
    }

    #[test]
    fn test_full_step_down_updates_tone_when_playing() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        pane.handle_action(
            ActionId::Tuner(TunerActionId::TogglePlayback),
            &event,
            &state,
        );
        assert!(pane.playing);

        let action = pane.handle_action(
            ActionId::Tuner(TunerActionId::ToggleFullStepDown),
            &event,
            &state,
        );
        assert!(matches!(action, Action::Tuner(TunerAction::PlayTone(_, _))));
        assert!(pane.full_step_down);
    }

    #[test]
    fn test_full_step_down_no_action_when_not_playing() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        let action = pane.handle_action(
            ActionId::Tuner(TunerActionId::ToggleFullStepDown),
            &event,
            &state,
        );
        assert!(matches!(action, Action::None));
    }

    #[test]
    fn test_midi_to_note_name() {
        assert_eq!(midi_to_note_name(69), "A4");
        assert_eq!(midi_to_note_name(60), "C4");
        assert_eq!(midi_to_note_name(64), "E4");
        assert_eq!(midi_to_note_name(62), "D4");
        assert_eq!(midi_to_note_name(40), "E2");
        assert_eq!(midi_to_note_name(38), "D2");
    }

    #[test]
    fn test_bright_toggle() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        assert!(!pane.bright);
        pane.handle_action(ActionId::Tuner(TunerActionId::ToggleBright), &event, &state);
        assert!(pane.bright);
        pane.handle_action(ActionId::Tuner(TunerActionId::ToggleBright), &event, &state);
        assert!(!pane.bright);
    }

    #[test]
    fn test_bright_updates_tone_when_playing() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        pane.handle_action(
            ActionId::Tuner(TunerActionId::TogglePlayback),
            &event,
            &state,
        );
        assert!(pane.playing);

        let action =
            pane.handle_action(ActionId::Tuner(TunerActionId::ToggleBright), &event, &state);
        assert!(matches!(
            action,
            Action::Tuner(TunerAction::PlayTone(_, true))
        ));
    }

    #[test]
    fn test_bright_no_action_when_not_playing() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        let action =
            pane.handle_action(ActionId::Tuner(TunerActionId::ToggleBright), &event, &state);
        assert!(matches!(action, Action::None));
    }

    #[test]
    fn test_play_carries_bright_state() {
        let mut pane = TunerPane::default();
        let state = make_state();
        let event = make_event();

        // Enable bright first
        pane.bright = true;

        let action = pane.handle_action(
            ActionId::Tuner(TunerActionId::TogglePlayback),
            &event,
            &state,
        );
        assert!(matches!(
            action,
            Action::Tuner(TunerAction::PlayTone(_, true))
        ));
    }
}
