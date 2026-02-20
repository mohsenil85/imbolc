use std::any::Any;
use std::time::Instant;

use crate::state::music::{JIFlavor, Key, Scale, Tuning};
use crate::state::{AppState, MusicalSettings};
use crate::ui::action_id::{ActionId, FrameEditActionId, ModeActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::widgets::TextInput;
use crate::ui::{
    Action, InputEvent, Keymap, Palette, Pane, PaneIdStr, Rect, RenderBuf, SessionAction, Style,
};

/// Fields editable in the frame editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Bpm,
    TimeSig,
    TuningA4,
    TuningSystem,
    JIFlavor,
    Key,
    Scale,
    Snap,
}

const FIELDS: [Field; 8] = [
    Field::Bpm,
    Field::TimeSig,
    Field::TuningA4,
    Field::TuningSystem,
    Field::JIFlavor,
    Field::Key,
    Field::Scale,
    Field::Snap,
];

const BPM_MIN: u16 = 20;
const BPM_MAX: u16 = 300;
const TAP_TIMEOUT_SECS: f64 = 2.0;

pub struct FrameEditPane {
    keymap: Keymap,
    settings: MusicalSettings,
    original_settings: MusicalSettings,
    selected: usize,
    editing: bool,
    edit_input: TextInput,
    // Tap tempo state
    tapping: bool,
    tap_times: Vec<Instant>,
    tap_candidates: [Option<u16>; 3], // [half, detected, double]
    tap_selected: usize,              // 0=half, 1=detected, 2=double
}

impl FrameEditPane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            settings: MusicalSettings::default(),
            original_settings: MusicalSettings::default(),
            selected: 0,
            editing: false,
            edit_input: TextInput::new(""),
            tapping: false,
            tap_times: Vec::new(),
            tap_candidates: [None; 3],
            tap_selected: 1,
        }
    }

    /// Set musical settings to edit (called before switching to this pane)
    #[allow(dead_code)]
    pub fn set_settings(&mut self, settings: MusicalSettings) {
        self.settings = settings;
        self.original_settings = self.settings.clone();
        self.selected = 0;
        self.editing = false;
        self.tapping = false;
    }

    pub fn is_tapping(&self) -> bool {
        self.tapping
    }

    fn current_field(&self) -> Field {
        FIELDS[self.selected]
    }

    fn cycle_key(&mut self, forward: bool) {
        let idx = Key::ALL
            .iter()
            .position(|k| *k == self.settings.key)
            .unwrap_or(0);
        self.settings.key = if forward {
            Key::ALL[(idx + 1) % 12]
        } else {
            Key::ALL[(idx + 11) % 12]
        };
    }

    fn cycle_scale(&mut self, forward: bool) {
        let idx = Scale::ALL
            .iter()
            .position(|s| *s == self.settings.scale)
            .unwrap_or(0);
        let len = Scale::ALL.len();
        self.settings.scale = if forward {
            Scale::ALL[(idx + 1) % len]
        } else {
            Scale::ALL[(idx + len - 1) % len]
        };
    }

    const TIME_SIGS: [(u8, u8); 5] = [(4, 4), (3, 4), (6, 8), (5, 4), (7, 8)];

    fn cycle_time_sig(&mut self, forward: bool) {
        let idx = Self::TIME_SIGS
            .iter()
            .position(|ts| *ts == self.settings.time_signature)
            .unwrap_or(0);
        let len = Self::TIME_SIGS.len();
        self.settings.time_signature = if forward {
            Self::TIME_SIGS[(idx + 1) % len]
        } else {
            Self::TIME_SIGS[(idx + len - 1) % len]
        };
    }

    fn cycle_tuning_system(&mut self, forward: bool) {
        let idx = Tuning::ALL
            .iter()
            .position(|t| *t == self.settings.tuning)
            .unwrap_or(0);
        let len = Tuning::ALL.len();
        self.settings.tuning = if forward {
            Tuning::ALL[(idx + 1) % len]
        } else {
            Tuning::ALL[(idx + len - 1) % len]
        };
    }

    fn cycle_ji_flavor(&mut self, forward: bool) {
        let idx = JIFlavor::ALL
            .iter()
            .position(|f| *f == self.settings.ji_flavor)
            .unwrap_or(0);
        let len = JIFlavor::ALL.len();
        self.settings.ji_flavor = if forward {
            JIFlavor::ALL[(idx + 1) % len]
        } else {
            JIFlavor::ALL[(idx + len - 1) % len]
        };
    }

    fn adjust(&mut self, increase: bool) {
        match self.current_field() {
            Field::Bpm => {
                let delta: i16 = if increase { 1 } else { -1 };
                self.settings.bpm =
                    (self.settings.bpm as i16 + delta).clamp(BPM_MIN as i16, BPM_MAX as i16) as u16;
            }
            Field::TimeSig => self.cycle_time_sig(increase),
            Field::TuningA4 => {
                let delta: f32 = if increase { 1.0 } else { -1.0 };
                self.settings.tuning_a4 = (self.settings.tuning_a4 + delta).clamp(400.0, 480.0);
            }
            Field::TuningSystem => self.cycle_tuning_system(increase),
            Field::JIFlavor => self.cycle_ji_flavor(increase),
            Field::Key => self.cycle_key(increase),
            Field::Scale => self.cycle_scale(increase),
            Field::Snap => self.settings.snap = !self.settings.snap,
        }
    }

    fn field_label(field: Field) -> &'static str {
        match field {
            Field::Bpm => "BPM",
            Field::TimeSig => "Time Sig",
            Field::TuningA4 => "Tuning (A4)",
            Field::TuningSystem => "Tuning",
            Field::JIFlavor => "JI Flavor",
            Field::Key => "Key",
            Field::Scale => "Scale",
            Field::Snap => "Snap",
        }
    }

    fn field_value(&self, field: Field) -> String {
        match field {
            Field::Bpm => format!("{}", self.settings.bpm),
            Field::TimeSig => format!(
                "{}/{}",
                self.settings.time_signature.0, self.settings.time_signature.1
            ),
            Field::TuningA4 => format!("{:.1} Hz", self.settings.tuning_a4),
            Field::TuningSystem => self.settings.tuning.name().to_string(),
            Field::JIFlavor => self.settings.ji_flavor.name().to_string(),
            Field::Key => self.settings.key.name().to_string(),
            Field::Scale => self.settings.scale.name().to_string(),
            Field::Snap => {
                if self.settings.snap {
                    "ON".into()
                } else {
                    "OFF".into()
                }
            }
        }
    }

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    // --- Tap tempo methods ---

    fn recalculate_tap_bpm(&mut self) {
        if self.tap_times.len() < 2 {
            self.tap_candidates = [None; 3];
            return;
        }
        let intervals: Vec<f64> = self
            .tap_times
            .windows(2)
            .map(|w| w[1].duration_since(w[0]).as_secs_f64())
            .collect();
        let avg = intervals.iter().sum::<f64>() / intervals.len() as f64;
        let detected = 60.0 / avg;

        let clamp_bpm = |bpm: f64| -> Option<u16> {
            let rounded = bpm.round() as i32;
            if rounded >= BPM_MIN as i32 && rounded <= BPM_MAX as i32 {
                Some(rounded as u16)
            } else {
                None
            }
        };

        self.tap_candidates = [
            clamp_bpm(detected / 2.0),
            clamp_bpm(detected),
            clamp_bpm(detected * 2.0),
        ];

        // Ensure selected index points to a valid candidate
        if self.tap_candidates[self.tap_selected].is_none() {
            // Find nearest valid
            for &i in &[1, 0, 2] {
                if self.tap_candidates[i].is_some() {
                    self.tap_selected = i;
                    return;
                }
            }
        }
    }

    fn select_prev_candidate(&mut self) {
        let mut i = self.tap_selected;
        loop {
            if i == 0 {
                break;
            }
            i -= 1;
            if self.tap_candidates[i].is_some() {
                self.tap_selected = i;
                return;
            }
        }
    }

    fn select_next_candidate(&mut self) {
        let mut i = self.tap_selected;
        loop {
            if i >= 2 {
                break;
            }
            i += 1;
            if self.tap_candidates[i].is_some() {
                self.tap_selected = i;
                return;
            }
        }
    }

    fn render_tap_tempo(&self, area: Rect, buf: &mut RenderBuf, p: &Palette) {
        let rect = center_rect(area, 44, 11);
        let border_style = Style::new().fg(p.accent);
        let inner = buf.draw_block(rect, " Tap Tempo ", border_style, border_style);

        let cx = inner.x + inner.width / 2;
        let mut y = inner.y + 1;

        // "Tap SPACE to the beat"
        let prompt = "Tap SPACE to the beat";
        let px = cx.saturating_sub(prompt.len() as u16 / 2);
        buf.draw_line(
            Rect::new(px, y, prompt.len() as u16, 1),
            &[(prompt, Style::new().fg(p.fg))],
        );

        y += 2;

        if self.tap_times.len() < 2 {
            let waiting = "waiting for taps...";
            let wx = cx.saturating_sub(waiting.len() as u16 / 2);
            buf.draw_line(
                Rect::new(wx, y, waiting.len() as u16, 1),
                &[(waiting, Style::new().fg(p.dim))],
            );
        } else {
            // "Taps: N          ♩ = BPM"
            let tap_count = format!("Taps: {}", self.tap_times.len());
            let detected_bpm = self.tap_candidates[1].map(|b| format!("{}", b));
            let bpm_str = detected_bpm.as_deref().unwrap_or("---");
            let info = format!("{}          ♩ = {}", tap_count, bpm_str);
            let ix = cx.saturating_sub(info.len() as u16 / 2);
            buf.draw_line(
                Rect::new(ix, y, info.len() as u16, 1),
                &[(&info, Style::new().fg(p.fg))],
            );

            y += 2;

            // Three candidates: half  ▸ detected  double
            let labels = ["(half)", "(detected)", "(double)"];
            let mut parts: Vec<String> = Vec::new();
            for (i, label) in labels.iter().enumerate() {
                let val = self.tap_candidates[i]
                    .map(|b| format!("{}", b))
                    .unwrap_or_else(|| "---".to_string());
                let marker = if i == self.tap_selected {
                    "\u{25b8} "
                } else {
                    "  "
                };
                parts.push(format!("{}{}", marker, val));
                parts.push(label.to_string());
            }

            // Render values line
            let val_line = format!(
                "{}    {}    {}",
                parts[0], // half value
                parts[2], // detected value
                parts[4], // double value
            );
            let vx = cx.saturating_sub(val_line.len() as u16 / 2);
            // Draw character by character to highlight selected
            let items = [
                (parts[0].as_str(), 0usize),
                (parts[2].as_str(), 1),
                (parts[4].as_str(), 2),
            ];
            let mut x = vx;
            for (idx, (text, cand_idx)) in items.iter().enumerate() {
                let style = if *cand_idx == self.tap_selected {
                    Style::new().fg(p.accent).bold()
                } else if self.tap_candidates[*cand_idx].is_some() {
                    Style::new().fg(p.fg)
                } else {
                    Style::new().fg(p.dim)
                };
                buf.draw_line(Rect::new(x, y, text.len() as u16, 1), &[(text, style)]);
                x += text.len() as u16;
                if idx < 2 {
                    buf.draw_line(Rect::new(x, y, 4, 1), &[("    ", Style::new().fg(p.fg))]);
                    x += 4;
                }
            }

            y += 1;
            // Labels line — compute total width for centering
            let label_total_width = parts[0].len() + 4 + parts[2].len() + 4 + parts[4].len();
            let lx = cx.saturating_sub(label_total_width as u16 / 2);
            let mut lxx = lx;
            for (idx, label) in labels.iter().enumerate() {
                let w = parts[idx * 2].len();
                let padded = format!("{:>width$}", label, width = w);
                let style = if idx == self.tap_selected {
                    Style::new().fg(p.accent)
                } else {
                    Style::new().fg(p.dim)
                };
                buf.draw_line(
                    Rect::new(lxx, y, padded.len() as u16, 1),
                    &[(&padded, style)],
                );
                lxx += padded.len() as u16;
                if idx < 2 {
                    lxx += 4;
                }
            }
        }

        // Footer hint
        let footer = "\u{25c4}/\u{25ba} select   ENTER apply   ESC cancel";
        let fy = inner.y + inner.height.saturating_sub(1);
        let fx = cx.saturating_sub(footer.len() as u16 / 2);
        buf.draw_line(
            Rect::new(fx, fy, footer.len() as u16, 1),
            &[(footer, Style::new().fg(p.dim))],
        );
    }
}

impl Default for FrameEditPane {
    fn default() -> Self {
        Self::new(Keymap::new())
    }
}

impl Pane for FrameEditPane {
    fn id(&self) -> PaneIdStr {
        PaneIdStr("frame_edit")
    }

    fn handle_action(
        &mut self,
        action: ActionId,
        _event: &InputEvent,
        _state: &AppState,
    ) -> Action {
        if let ActionId::Mode(mode_action) = action {
            return match mode_action {
                // Text edit layer actions
                ModeActionId::TextConfirm => {
                    let text = self.edit_input.value().to_string();
                    match self.current_field() {
                        Field::Bpm => {
                            if let Ok(v) = text.parse::<u16>() {
                                self.settings.bpm = v.clamp(BPM_MIN, BPM_MAX);
                            }
                        }
                        Field::TuningA4 => {
                            if let Ok(v) = text.parse::<f32>() {
                                self.settings.tuning_a4 = v.clamp(400.0, 480.0);
                            }
                        }
                        _ => {}
                    }
                    self.editing = false;
                    self.edit_input.set_focused(false);
                    Action::Session(SessionAction::SetSession(self.settings.clone()))
                }
                ModeActionId::TextCancel => {
                    self.editing = false;
                    self.edit_input.set_focused(false);
                    self.settings = self.original_settings.clone();
                    Action::Session(SessionAction::SetSession(self.original_settings.clone()))
                }
                // Tap tempo layer actions
                ModeActionId::TapTap => {
                    let now = Instant::now();
                    // Reset if gap > 2 seconds
                    if let Some(last) = self.tap_times.last() {
                        if now.duration_since(*last).as_secs_f64() > TAP_TIMEOUT_SECS {
                            self.tap_times.clear();
                        }
                    }
                    self.tap_times.push(now);
                    self.recalculate_tap_bpm();
                    Action::None
                }
                ModeActionId::TapPrev => {
                    self.select_prev_candidate();
                    Action::None
                }
                ModeActionId::TapNext => {
                    self.select_next_candidate();
                    Action::None
                }
                ModeActionId::TapConfirm => {
                    if let Some(bpm) = self.tap_candidates[self.tap_selected] {
                        self.settings.bpm = bpm;
                        self.tapping = false;
                        Action::Session(SessionAction::SetSession(self.settings.clone()))
                    } else {
                        Action::None
                    }
                }
                ModeActionId::TapCancel => {
                    self.tapping = false;
                    Action::None
                }
                ModeActionId::PianoEscape
                | ModeActionId::PianoOctaveDown
                | ModeActionId::PianoOctaveUp
                | ModeActionId::PianoSpace
                | ModeActionId::PianoKey
                | ModeActionId::PadEscape
                | ModeActionId::PadKey
                | ModeActionId::PaletteConfirm
                | ModeActionId::PaletteCancel => Action::None,
            };
        }

        let ActionId::FrameEdit(action) = action else {
            return Action::None;
        };

        match action {
            FrameEditActionId::Prev => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Action::None
            }
            FrameEditActionId::Next => {
                if self.selected < FIELDS.len() - 1 {
                    self.selected += 1;
                }
                Action::None
            }
            FrameEditActionId::Decrease => {
                self.adjust(false);
                Action::Session(SessionAction::SetSessionLive(self.settings.clone()))
            }
            FrameEditActionId::Increase => {
                self.adjust(true);
                Action::Session(SessionAction::SetSessionLive(self.settings.clone()))
            }
            FrameEditActionId::Confirm => {
                let field = self.current_field();
                if matches!(field, Field::Bpm | Field::TuningA4) {
                    let val = match field {
                        Field::Bpm => format!("{}", self.settings.bpm),
                        Field::TuningA4 => format!("{:.1}", self.settings.tuning_a4),
                        _ => unreachable!(),
                    };
                    self.edit_input.set_value(&val);
                    self.edit_input.select_all();
                    self.edit_input.set_focused(true);
                    self.editing = true;
                    Action::PushLayer("text_edit")
                } else {
                    Action::Session(SessionAction::SetSession(self.settings.clone()))
                }
            }
            FrameEditActionId::Cancel => {
                self.settings = self.original_settings.clone();
                Action::Session(SessionAction::SetSession(self.original_settings.clone()))
            }
            FrameEditActionId::TapTempo => {
                self.tapping = true;
                self.tap_times.clear();
                self.tap_candidates = [None; 3];
                self.tap_selected = 1;
                Action::PushLayer("tap_tempo")
            }
        }
    }

    fn handle_raw_input(&mut self, event: &InputEvent, _state: &AppState) -> Action {
        if self.editing {
            self.edit_input.handle_input(event);
        }
        Action::None
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let p = Palette::from(&state.session.theme);

        if self.tapping {
            self.render_tap_tempo(area, buf, &p);
            return;
        }

        let rect = center_rect(area, 50, 13);

        let border_style = Style::new().fg(p.accent);
        let inner = buf.draw_block(rect, " Session ", border_style, border_style);

        let label_col = inner.x + 2;
        let value_col = label_col + 15;

        for (i, field) in FIELDS.iter().enumerate() {
            let y = inner.y + 1 + i as u16;
            if y >= inner.y + inner.height {
                break;
            }
            let is_selected = i == self.selected;
            let sel_bg = Style::new().bg(p.selection_bg);

            // Indicator
            if is_selected {
                buf.set_cell(
                    label_col,
                    y,
                    '>',
                    Style::new().fg(p.fg).bg(p.selection_bg).bold(),
                );
            }

            // Label
            let label_style = if is_selected {
                Style::new().fg(p.accent).bg(p.selection_bg)
            } else {
                Style::new().fg(p.accent)
            };
            let label = format!("{:14}", Self::field_label(*field));
            buf.draw_line(Rect::new(label_col + 2, y, 14, 1), &[(&label, label_style)]);

            // Value
            if is_selected && self.editing {
                // Render TextInput inline
                self.edit_input.render_buf(
                    buf.raw_buf(),
                    value_col,
                    y,
                    inner.width.saturating_sub(18),
                    &p,
                );
            } else {
                let val_style = if is_selected {
                    Style::new().fg(p.fg).bg(p.selection_bg)
                } else {
                    Style::new().fg(p.fg)
                };
                let val = self.field_value(*field);
                buf.draw_line(
                    Rect::new(value_col, y, inner.width.saturating_sub(18), 1),
                    &[(&val, val_style)],
                );

                // Fill rest of line with selection bg
                if is_selected {
                    let fill_start = value_col + val.len() as u16;
                    let fill_end = inner.x + inner.width;
                    for x in fill_start..fill_end {
                        buf.set_cell(x, y, ' ', sel_bg);
                    }
                }
            }
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn on_enter(&mut self, state: &AppState) {
        self.set_settings(state.session.musical_settings());
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use crate::ui::{InputEvent, KeyCode, Modifiers};

    fn dummy_event() -> InputEvent {
        InputEvent::new(KeyCode::Char('x'), Modifiers::default())
    }

    #[test]
    fn confirm_non_text_field_commits() {
        use crate::ui::action_id::{ActionId, FrameEditActionId};
        let mut pane = FrameEditPane::new(Keymap::new());
        let state = AppState::new();
        let settings = MusicalSettings::default();
        pane.set_settings(settings);

        for _ in 0..3 {
            pane.handle_action(
                ActionId::FrameEdit(FrameEditActionId::Next),
                &dummy_event(),
                &state,
            );
        }

        let action = pane.handle_action(
            ActionId::FrameEdit(FrameEditActionId::Confirm),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Session(SessionAction::SetSession(updated)) => {
                assert_eq!(updated.key, pane.settings.key);
            }
            _ => panic!("Expected SetSession for non-text field confirm"),
        }
    }

    #[test]
    fn text_confirm_commits_and_returns() {
        use crate::ui::action_id::{ActionId, ModeActionId};
        let mut pane = FrameEditPane::new(Keymap::new());
        let state = AppState::new();
        let settings = MusicalSettings::default();
        pane.set_settings(settings);
        pane.edit_input.set_value("180");

        let action = pane.handle_action(
            ActionId::Mode(ModeActionId::TextConfirm),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Session(SessionAction::SetSession(updated)) => {
                assert_eq!(updated.bpm, 180);
                assert!(!pane.editing);
            }
            _ => panic!("Expected SetSession on text confirm"),
        }
    }

    #[test]
    fn cancel_reverts_to_original_settings() {
        use crate::ui::action_id::{ActionId, FrameEditActionId};
        let mut pane = FrameEditPane::new(Keymap::new());
        let state = AppState::new();
        let settings = MusicalSettings {
            bpm: 140,
            ..Default::default()
        };
        pane.set_settings(settings.clone());

        pane.settings.bpm = 200;

        let action = pane.handle_action(
            ActionId::FrameEdit(FrameEditActionId::Cancel),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Session(SessionAction::SetSession(updated)) => {
                assert_eq!(updated, settings);
                assert_eq!(pane.settings, settings);
            }
            _ => panic!("Expected SetSession on cancel"),
        }
    }

    #[test]
    fn text_cancel_reverts_to_original_settings() {
        use crate::ui::action_id::{ActionId, ModeActionId};
        let mut pane = FrameEditPane::new(Keymap::new());
        let state = AppState::new();
        let settings = MusicalSettings {
            tuning_a4: 432.0,
            ..Default::default()
        };
        pane.set_settings(settings.clone());

        pane.settings.tuning_a4 = 450.0;
        pane.editing = true;

        let action = pane.handle_action(
            ActionId::Mode(ModeActionId::TextCancel),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Session(SessionAction::SetSession(updated)) => {
                assert_eq!(updated, settings);
                assert_eq!(pane.settings, settings);
                assert!(!pane.editing);
            }
            _ => panic!("Expected SetSession on text cancel"),
        }
    }

    #[test]
    fn tap_tempo_enters_mode() {
        let mut pane = FrameEditPane::new(Keymap::new());
        let state = AppState::new();
        pane.set_settings(MusicalSettings::default());

        let action = pane.handle_action(
            ActionId::FrameEdit(FrameEditActionId::TapTempo),
            &dummy_event(),
            &state,
        );
        assert!(pane.is_tapping());
        assert!(matches!(action, Action::PushLayer("tap_tempo")));
        assert!(pane.tap_times.is_empty());
    }

    #[test]
    fn tap_tempo_cancel_exits() {
        let mut pane = FrameEditPane::new(Keymap::new());
        let state = AppState::new();
        pane.set_settings(MusicalSettings::default());
        pane.tapping = true;

        let action = pane.handle_action(
            ActionId::Mode(ModeActionId::TapCancel),
            &dummy_event(),
            &state,
        );
        assert!(!pane.is_tapping());
        assert!(matches!(action, Action::None));
    }

    #[test]
    fn tap_tempo_calculates_bpm() {
        let mut pane = FrameEditPane::new(Keymap::new());
        pane.tapping = true;

        // Simulate taps at 120 BPM (0.5s intervals)
        let base = Instant::now();
        pane.tap_times = vec![base, base + std::time::Duration::from_millis(500)];
        pane.recalculate_tap_bpm();

        assert_eq!(pane.tap_candidates[1], Some(120)); // detected
        assert_eq!(pane.tap_candidates[0], Some(60)); // half
        assert_eq!(pane.tap_candidates[2], Some(240)); // double
    }

    #[test]
    fn tap_tempo_confirm_applies_bpm() {
        let mut pane = FrameEditPane::new(Keymap::new());
        let state = AppState::new();
        pane.set_settings(MusicalSettings {
            bpm: 120,
            ..Default::default()
        });
        pane.tapping = true;
        pane.tap_candidates = [Some(60), Some(120), Some(240)];
        pane.tap_selected = 2; // double = 240

        let action = pane.handle_action(
            ActionId::Mode(ModeActionId::TapConfirm),
            &dummy_event(),
            &state,
        );
        assert!(!pane.is_tapping());
        assert_eq!(pane.settings.bpm, 240);
        assert!(matches!(
            action,
            Action::Session(SessionAction::SetSession(_))
        ));
    }

    #[test]
    fn tap_tempo_confirm_noop_without_candidates() {
        let mut pane = FrameEditPane::new(Keymap::new());
        let state = AppState::new();
        pane.set_settings(MusicalSettings::default());
        pane.tapping = true;
        // No taps, no candidates
        pane.tap_candidates = [None; 3];

        let action = pane.handle_action(
            ActionId::Mode(ModeActionId::TapConfirm),
            &dummy_event(),
            &state,
        );
        assert!(pane.is_tapping()); // Still tapping — confirm was a no-op
        assert!(matches!(action, Action::None));
    }

    #[test]
    fn tap_tempo_candidate_out_of_range() {
        let mut pane = FrameEditPane::new(Keymap::new());
        pane.tapping = true;

        // Simulate very fast taps at 300 BPM (0.2s intervals)
        let base = Instant::now();
        pane.tap_times = vec![base, base + std::time::Duration::from_millis(200)];
        pane.recalculate_tap_bpm();

        assert_eq!(pane.tap_candidates[1], Some(300)); // detected = 300 (at limit)
        assert_eq!(pane.tap_candidates[0], Some(150)); // half = 150
        assert_eq!(pane.tap_candidates[2], None); // double = 600 — out of range
    }

    #[test]
    fn tap_tempo_navigate_candidates() {
        let mut pane = FrameEditPane::new(Keymap::new());
        pane.tapping = true;
        pane.tap_candidates = [Some(60), Some(120), Some(240)];
        pane.tap_selected = 1;

        pane.select_prev_candidate();
        assert_eq!(pane.tap_selected, 0);

        pane.select_next_candidate();
        assert_eq!(pane.tap_selected, 1);

        pane.select_next_candidate();
        assert_eq!(pane.tap_selected, 2);

        // Can't go further
        pane.select_next_candidate();
        assert_eq!(pane.tap_selected, 2);
    }

    #[test]
    fn tap_tempo_skips_none_candidates() {
        let mut pane = FrameEditPane::new(Keymap::new());
        pane.tapping = true;
        pane.tap_candidates = [None, Some(120), Some(240)];
        pane.tap_selected = 1;

        // Prev should not move to None
        pane.select_prev_candidate();
        assert_eq!(pane.tap_selected, 1);

        pane.select_next_candidate();
        assert_eq!(pane.tap_selected, 2);
    }
}
