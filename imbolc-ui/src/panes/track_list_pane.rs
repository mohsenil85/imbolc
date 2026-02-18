use std::any::Any;

use crate::state::{AppState, OwnershipDisplayStatus, SourceType};
use crate::ui::action_id::{ActionId, ModeActionId, TrackListActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::performance::PerformanceController;
use crate::ui::widgets::TextInput;
use crate::ui::{
    translate_key, Action, Color, InputEvent, KeyCode, Keymap, MouseButton, MouseEvent,
    MouseEventKind, NavAction, Pane, PaneId, PaneIdStr, Rect, RenderBuf, SessionAction, Style,
    ToggleResult, TrackAction,
};
use imbolc_types::{GroupAction, TrackId};

fn source_color(source: SourceType) -> Color {
    match source {
        // Oscillators and synths
        SourceType::Saw | SourceType::Sin | SourceType::Sqr | SourceType::Tri
        | SourceType::Noise | SourceType::Pulse | SourceType::SuperSaw | SourceType::Sync
        | SourceType::Ring | SourceType::FBSin | SourceType::FM | SourceType::PhaseMod
        | SourceType::FMBell | SourceType::FMBrass
        | SourceType::Pluck | SourceType::Formant | SourceType::Gendy | SourceType::Chaos
        | SourceType::Additive | SourceType::Wavetable | SourceType::Granular
        | SourceType::Bowed | SourceType::Blown | SourceType::Membrane
        // Mallet percussion
        | SourceType::Marimba | SourceType::Vibes | SourceType::Kalimba | SourceType::SteelDrum
        | SourceType::TubularBell | SourceType::Glockenspiel
        // Plucked strings
        | SourceType::Guitar | SourceType::BassGuitar | SourceType::Harp | SourceType::Koto
        // Drums
        | SourceType::Kick | SourceType::Snare | SourceType::HihatClosed | SourceType::HihatOpen
        | SourceType::Clap | SourceType::Cowbell | SourceType::Rim | SourceType::Tom
        | SourceType::Clave | SourceType::Conga
        // Classic synths
        | SourceType::Choir | SourceType::EPiano | SourceType::Organ | SourceType::BrassStab
        | SourceType::Strings | SourceType::Acid
        | SourceType::Universe | SourceType::Dreamscape | SourceType::Soundtrack => Color::OSC_COLOR,
        SourceType::AudioIn => Color::AUDIO_IN_COLOR,
        SourceType::PitchedSampler | SourceType::TimeStretch | SourceType::Kit => {
            Color::SAMPLE_COLOR
        }
        SourceType::BusIn => Color::BUS_IN_COLOR,
        SourceType::Custom(_) => Color::CUSTOM_COLOR,
        SourceType::Vst(_) => Color::VST_COLOR,
    }
}

pub struct TrackListPane {
    keymap: Keymap,
    perf: PerformanceController,
    /// When Some, we're waiting for the user to select a target instrument to link with
    linking_from: Option<crate::state::TrackId>,
    /// Text input for renaming layer groups
    edit_input: TextInput,
    /// Layer group being renamed (None = not editing)
    editing_group: Option<u32>,
}

impl TrackListPane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            perf: PerformanceController::new(),
            linking_from: None,
            edit_input: TextInput::new(""),
            editing_group: None,
        }
    }

    pub fn is_editing(&self) -> bool {
        self.editing_group.is_some()
    }

    pub fn set_enhanced_keyboard(&mut self, enabled: bool) {
        self.perf.set_enhanced_keyboard(enabled);
    }

    fn format_filter(instrument: &crate::state::track::Track) -> String {
        match instrument.filter() {
            Some(f) => format!("[{}]", f.filter_type.name()),
            None => "---".to_string(),
        }
    }

    fn format_eq(instrument: &crate::state::track::Track) -> &'static str {
        if instrument.eq().is_some() {
            "[EQ]"
        } else {
            ""
        }
    }

    fn format_effects(instrument: &crate::state::track::Track) -> String {
        let effects: Vec<_> = instrument.effects().collect();
        if effects.is_empty() {
            return "---".to_string();
        }
        effects
            .iter()
            .map(|e| e.effect_type.name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_level(level: f32) -> String {
        let filled = (level * 5.0) as usize;
        let bar: String = (0..5).map(|i| if i < filled { '▊' } else { '░' }).collect();
        format!("{} {:.0}%", bar, level * 100.0)
    }
}

impl Default for TrackListPane {
    fn default() -> Self {
        Self {
            keymap: Keymap::new(),
            perf: PerformanceController::new(),
            linking_from: None,
            edit_input: TextInput::new(""),
            editing_group: None,
        }
    }
}

impl Pane for TrackListPane {
    fn id(&self) -> PaneIdStr {
        PaneIdStr("instrument")
    }

    fn handle_action(&mut self, action: ActionId, event: &InputEvent, state: &AppState) -> Action {
        // If we're in linking mode, handle confirm/cancel/navigate
        if let Some(from_id) = self.linking_from {
            match action {
                // Press 'l' again to confirm the link
                ActionId::TrackList(TrackListActionId::LinkLayer) => {
                    self.linking_from = None;
                    if let Some(target) = state.tracks.selected_track() {
                        let target_id = target.id;
                        if target_id != from_id {
                            return Action::Track(TrackAction::LinkLayer(from_id, target_id));
                        }
                    }
                    return Action::None;
                }
                // Navigation passes through to normal handling below
                ActionId::TrackList(TrackListActionId::Next)
                | ActionId::TrackList(TrackListActionId::Prev)
                | ActionId::TrackList(TrackListActionId::GotoTop)
                | ActionId::TrackList(TrackListActionId::GotoBottom) => {
                    // Let navigation proceed normally
                }
                // Any other action cancels linking mode
                _ => {
                    self.linking_from = None;
                }
            }
        }

        match action {
            ActionId::TrackList(TrackListActionId::Quit) => Action::QuitIntent,
            ActionId::TrackList(TrackListActionId::Next) => Action::Track(TrackAction::SelectNext),
            ActionId::TrackList(TrackListActionId::Prev) => Action::Track(TrackAction::SelectPrev),
            ActionId::TrackList(TrackListActionId::GotoTop) => {
                Action::Track(TrackAction::SelectFirst)
            }
            ActionId::TrackList(TrackListActionId::GotoBottom) => {
                Action::Track(TrackAction::SelectLast)
            }
            ActionId::TrackList(TrackListActionId::Add) => {
                Action::Nav(NavAction::SwitchPane(PaneId::Add))
            }
            ActionId::TrackList(TrackListActionId::Delete) => {
                if let Some(instrument) = state.tracks.selected_track() {
                    Action::Track(TrackAction::Delete(instrument.id))
                } else {
                    Action::None
                }
            }
            ActionId::TrackList(TrackListActionId::Edit) => {
                if let Some(instrument) = state.tracks.selected_track() {
                    Action::Track(TrackAction::Edit(instrument.id))
                } else {
                    Action::None
                }
            }
            ActionId::TrackList(TrackListActionId::Save) => Action::Session(SessionAction::Save),
            ActionId::TrackList(TrackListActionId::Load) => Action::Session(SessionAction::Load),
            ActionId::TrackList(TrackListActionId::LinkLayer) => {
                if let Some(instrument) = state.tracks.selected_track() {
                    self.linking_from = Some(instrument.id);
                }
                Action::None
            }
            ActionId::TrackList(TrackListActionId::UnlinkLayer) => {
                if let Some(instrument) = state.tracks.selected_track() {
                    Action::Track(TrackAction::UnlinkLayer(instrument.id))
                } else {
                    Action::None
                }
            }
            ActionId::TrackList(TrackListActionId::LayerOctaveUp) => {
                if let Some(instrument) = state.tracks.selected_track() {
                    Action::Track(TrackAction::AdjustLayerOctaveOffset(instrument.id, 1))
                } else {
                    Action::None
                }
            }
            ActionId::TrackList(TrackListActionId::LayerOctaveDown) => {
                if let Some(instrument) = state.tracks.selected_track() {
                    Action::Track(TrackAction::AdjustLayerOctaveOffset(instrument.id, -1))
                } else {
                    Action::None
                }
            }
            ActionId::TrackList(TrackListActionId::RenameLayerGroup) => {
                if let Some(instrument) = state.tracks.selected_track() {
                    if let Some(group_id) = instrument.layer.group {
                        let current_name = state
                            .session
                            .mixer
                            .layer_group_mixer(group_id)
                            .map(|gm| gm.name.clone())
                            .unwrap_or_default();
                        self.edit_input.set_value(&current_name);
                        self.edit_input.select_all();
                        self.edit_input.set_focused(true);
                        self.editing_group = Some(group_id);
                        return Action::PushLayer("text_edit");
                    }
                }
                Action::None
            }

            // Text edit mode actions
            ActionId::Mode(ModeActionId::TextConfirm) => {
                if let Some(group_id) = self.editing_group.take() {
                    let name = self.edit_input.value().to_string();
                    self.edit_input.set_focused(false);
                    return Action::Group(GroupAction::Rename(group_id, name));
                }
                Action::None
            }
            ActionId::Mode(ModeActionId::TextCancel) => {
                self.editing_group = None;
                self.edit_input.set_focused(false);
                Action::None
            }

            // Piano layer actions
            ActionId::Mode(ModeActionId::PianoEscape) => {
                let was_active = self.perf.piano.is_active();
                self.perf.piano.handle_escape();
                if was_active && !self.perf.piano.is_active() {
                    Action::ExitPerformanceMode
                } else {
                    Action::None
                }
            }
            ActionId::Mode(ModeActionId::PianoOctaveDown) => {
                self.perf.piano.octave_down();
                Action::None
            }
            ActionId::Mode(ModeActionId::PianoOctaveUp) => {
                self.perf.piano.octave_up();
                Action::None
            }
            ActionId::Mode(ModeActionId::PianoKey) | ActionId::Mode(ModeActionId::PianoSpace) => {
                if let KeyCode::Char(c) = event.key {
                    let c = translate_key(c, state.keyboard_layout);
                    if let Some(pitches) = self.perf.piano.key_to_pitches(c) {
                        // Check if this is a new press or key repeat (sustain)
                        if let Some(new_pitches) = self.perf.piano.key_pressed(
                            c,
                            pitches.clone(),
                            event.timestamp,
                            event.is_repeat,
                        ) {
                            // NEW press - spawn voice(s)
                            if new_pitches.len() == 1 {
                                return Action::Track(TrackAction::PlayNote(new_pitches[0], 100));
                            } else {
                                return Action::Track(TrackAction::PlayNotes(new_pitches, 100));
                            }
                        }
                        // Key repeat - sustain, no action needed
                    }
                }
                Action::None
            }

            // Pad layer actions
            ActionId::Mode(ModeActionId::PadEscape) => {
                self.perf.pad.deactivate();
                Action::ExitPerformanceMode
            }
            ActionId::Mode(ModeActionId::PadKey) => {
                if let KeyCode::Char(c) = event.key {
                    let c = translate_key(c, state.keyboard_layout);
                    if let Some(pad_idx) = self.perf.pad.key_to_pad(c) {
                        return Action::Track(TrackAction::PlayDrumPad(pad_idx, 100));
                    }
                }
                Action::None
            }

            _ => Action::None,
        }
    }

    fn handle_raw_input(&mut self, event: &InputEvent, _state: &AppState) -> Action {
        if self.editing_group.is_some() {
            self.edit_input.handle_input(event);
        }
        Action::None
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let rect = center_rect(area, 97, 29);

        let border_style = Style::new().fg(Color::CYAN);
        let inner = buf.draw_block(rect, " Tracks ", border_style, border_style);

        let content_x = inner.x + 1;
        let content_y = inner.y + 1;

        buf.draw_line(
            Rect::new(content_x, content_y, inner.width.saturating_sub(2), 1),
            &[("Tracks:", Style::new().fg(Color::CYAN).bold())],
        );

        let list_y = content_y + 2;
        let max_visible = ((inner.height.saturating_sub(7)) as usize).max(3);

        if state.tracks.tracks.is_empty() {
            buf.draw_line(
                Rect::new(content_x + 2, list_y, inner.width.saturating_sub(4), 1),
                &[(
                    "(no tracks — press 'a' to add)",
                    Style::new().fg(Color::DARK_GRAY),
                )],
            );
        }

        let scroll_offset = state
            .tracks
            .selected
            .map(|s| {
                if s >= max_visible {
                    s - max_visible + 1
                } else {
                    0
                }
            })
            .unwrap_or(0);
        let sel_bg = Style::new().bg(Color::SELECTION_BG);

        for (i, instrument) in state.tracks.tracks.iter().enumerate().skip(scroll_offset) {
            let row = i - scroll_offset;
            if row >= max_visible {
                break;
            }
            let y = list_y + row as u16;
            if y >= inner.y + inner.height {
                break;
            }
            let is_selected = state.tracks.selected == Some(i);

            // Selection indicator
            if is_selected {
                buf.set_cell(
                    content_x,
                    y,
                    '>',
                    Style::new().fg(Color::WHITE).bg(Color::SELECTION_BG).bold(),
                );
            }

            let mk_style = |fg: Color| -> Style {
                if is_selected {
                    Style::new().fg(fg).bg(Color::SELECTION_BG)
                } else {
                    Style::new().fg(fg)
                }
            };

            // Build row as a Line with multiple spans
            let name_str = format!("{:14}", &instrument.name[..instrument.name.len().min(14)]);
            let source_str = format!(" {:10}", instrument.source.name());
            let filter_str = format!(" {:12}", Self::format_filter(instrument));
            let eq_str = format!(" {:4}", Self::format_eq(instrument));
            let fx_raw = Self::format_effects(instrument);
            let fx_str = format!(" {:18}", &fx_raw[..fx_raw.len().min(18)]);
            let level_str = format!(" {}", Self::format_level(instrument.channel_strip.level));

            let source_c = source_color(instrument.source);

            let layer_str = match instrument.layer.group {
                Some(g) => {
                    let group_name = state
                        .session
                        .mixer
                        .layer_group_mixer(g)
                        .map(|gm| gm.name.as_str())
                        .unwrap_or("");
                    let label = if group_name.is_empty() {
                        format!("L{}", g)
                    } else {
                        group_name.to_string()
                    };
                    if instrument.layer.octave_offset != 0 {
                        format!(" [{}:{:+}]", label, instrument.layer.octave_offset)
                    } else {
                        format!(" [{}]", label)
                    }
                }
                None => String::new(),
            };

            // Ownership indicator for network mode
            let ownership_str = match state.ownership_status(instrument.id) {
                OwnershipDisplayStatus::OwnedByMe => " [ME]".to_string(),
                OwnershipDisplayStatus::OwnedByOther(ref name) => {
                    let short = if name.len() > 6 { &name[..6] } else { name };
                    format!(" [{}]", short)
                }
                OwnershipDisplayStatus::Unowned => String::new(),
                OwnershipDisplayStatus::Local => String::new(),
            };
            let ownership_color = match state.ownership_status(instrument.id) {
                OwnershipDisplayStatus::OwnedByMe => Color::LIME,
                OwnershipDisplayStatus::OwnedByOther(_) => Color::ORANGE,
                _ => Color::DARK_GRAY,
            };

            let mut spans: Vec<(&str, Style)> = vec![
                (&name_str, mk_style(Color::WHITE)),
                (&source_str, mk_style(source_c)),
                (&filter_str, mk_style(Color::FILTER_COLOR)),
                (&eq_str, mk_style(Color::EQ_COLOR)),
                (&fx_str, mk_style(Color::FX_COLOR)),
                (&level_str, mk_style(Color::LIME)),
            ];
            if !layer_str.is_empty() {
                spans.push((&layer_str, mk_style(Color::ORANGE)));
            }
            if !ownership_str.is_empty() {
                spans.push((&ownership_str, mk_style(ownership_color)));
            }
            let line_width = inner.width.saturating_sub(3);
            buf.draw_line(Rect::new(content_x + 2, y, line_width, 1), &spans);

            // Fill rest of line with selection bg
            if is_selected {
                let fill_start = content_x + 2 + line_width;
                let fill_end = inner.x + inner.width;
                for x in fill_start..fill_end {
                    buf.set_cell(x, y, ' ', sel_bg);
                }
            }
        }

        // Scroll indicators
        let scroll_style = Style::new().fg(Color::ORANGE);
        if scroll_offset > 0 {
            buf.draw_line(
                Rect::new(rect.x + rect.width - 5, list_y, 3, 1),
                &[("...", scroll_style)],
            );
        }
        if scroll_offset + max_visible < state.tracks.tracks.len() {
            buf.draw_line(
                Rect::new(
                    rect.x + rect.width - 5,
                    list_y + max_visible as u16 - 1,
                    3,
                    1,
                ),
                &[("...", scroll_style)],
            );
        }

        // Piano/Pad mode indicator
        if self.perf.pad.is_active() {
            let pad_str = self.perf.pad.status_label();
            let pad_x = rect.x + rect.width - pad_str.len() as u16 - 1;
            buf.draw_line(
                Rect::new(pad_x, rect.y, pad_str.len() as u16, 1),
                &[(&pad_str, Style::new().fg(Color::BLACK).bg(Color::KIT_COLOR))],
            );
        } else if self.perf.piano.is_active() {
            let piano_str = self.perf.piano.status_label();
            let piano_x = rect.x + rect.width - piano_str.len() as u16 - 1;
            buf.draw_line(
                Rect::new(piano_x, rect.y, piano_str.len() as u16, 1),
                &[(&piano_str, Style::new().fg(Color::BLACK).bg(Color::PINK))],
            );
        }

        // Link mode indicator
        if self.linking_from.is_some() {
            let link_str = " LINK: \u{2191}/\u{2193} navigate, l confirm, Esc cancel ";
            let link_x = rect.x + rect.width - link_str.len() as u16 - 1;
            buf.draw_line(
                Rect::new(link_x, rect.y, link_str.len() as u16, 1),
                &[(link_str, Style::new().fg(Color::BLACK).bg(Color::ORANGE))],
            );
        }

        // Rename mode indicator + inline text input
        if self.editing_group.is_some() {
            let rename_str = " RENAME: Enter confirm, Esc cancel ";
            let rename_x = rect.x + rect.width - rename_str.len() as u16 - 1;
            buf.draw_line(
                Rect::new(rename_x, rect.y, rename_str.len() as u16, 1),
                &[(rename_str, Style::new().fg(Color::BLACK).bg(Color::LIME))],
            );

            // Draw text input at the bottom of the inner area
            let input_y = inner.y + inner.height.saturating_sub(2);
            let input_x = inner.x + 1;
            let input_width = inner.width.saturating_sub(2);
            self.edit_input
                .render_buf(buf.raw_buf(), input_x, input_y, input_width);
            if let Some((cx, cy)) = self.edit_input.screen_cursor() {
                buf.set_cursor_position(cx, cy);
            }
        }
    }

    fn handle_mouse(&mut self, event: &MouseEvent, area: Rect, state: &AppState) -> Action {
        let rect = center_rect(area, 97, 29);
        let inner_x = rect.x + 2;
        let inner_y = rect.y + 2;
        let content_y = inner_y + 1;
        let list_y = content_y + 2;
        let inner_height = rect.height.saturating_sub(4);
        let max_visible = ((inner_height.saturating_sub(7)) as usize).max(3);

        let scroll_offset = state
            .tracks
            .selected
            .map(|s| {
                if s >= max_visible {
                    s - max_visible + 1
                } else {
                    0
                }
            })
            .unwrap_or(0);

        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let col = event.column;
                let row = event.row;
                // Click on instrument list
                if col >= inner_x && row >= list_y && row < list_y + max_visible as u16 {
                    let clicked_idx = scroll_offset + (row - list_y) as usize;
                    if clicked_idx < state.tracks.tracks.len() {
                        return Action::Track(TrackAction::Select(clicked_idx));
                    }
                }
                Action::None
            }
            MouseEventKind::ScrollUp => Action::Track(TrackAction::SelectPrev),
            MouseEventKind::ScrollDown => Action::Track(TrackAction::SelectNext),
            _ => Action::None,
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn tick(&mut self, state: &AppState) -> Vec<Action> {
        let instrument_id = state
            .tracks
            .selected_track()
            .map(|inst| inst.id)
            .unwrap_or(TrackId::new(0));
        self.perf.tick_releases(instrument_id)
    }

    fn toggle_performance_mode(&mut self, state: &AppState) -> ToggleResult {
        let is_kit = state
            .tracks
            .selected_track()
            .is_some_and(|s| s.source.is_kit());
        self.perf.toggle(is_kit)
    }

    fn activate_piano(&mut self) {
        self.perf.activate_piano();
    }

    fn activate_pad(&mut self) {
        self.perf.activate_pad();
    }

    fn deactivate_performance(&mut self) {
        self.perf.deactivate();
    }

    fn supports_performance_mode(&self) -> bool {
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, SourceType};
    use crate::ui::{InputEvent, KeyCode, Modifiers};

    fn dummy_event() -> InputEvent {
        InputEvent::new(KeyCode::Char('x'), Modifiers::default())
    }

    #[test]
    fn delete_returns_selected_instrument_id() {
        use crate::ui::action_id::{ActionId, TrackListActionId};
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);
        let mut pane = TrackListPane::new(Keymap::new());

        let action = pane.handle_action(
            ActionId::TrackList(TrackListActionId::Delete),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Track(TrackAction::Delete(got)) => assert_eq!(got, id),
            _ => panic!("Expected TrackAction::Delete"),
        }
    }

    #[test]
    fn edit_returns_selected_instrument_id() {
        use crate::ui::action_id::{ActionId, TrackListActionId};
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Sin);
        let mut pane = TrackListPane::new(Keymap::new());

        let action = pane.handle_action(
            ActionId::TrackList(TrackListActionId::Edit),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Track(TrackAction::Edit(got)) => assert_eq!(got, id),
            _ => panic!("Expected TrackAction::Edit"),
        }
    }

    #[test]
    fn add_navigates_to_add_pane() {
        use crate::ui::action_id::{ActionId, TrackListActionId};
        let state = AppState::new();
        let mut pane = TrackListPane::new(Keymap::new());

        let action = pane.handle_action(
            ActionId::TrackList(TrackListActionId::Add),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Nav(NavAction::SwitchPane(id)) => assert_eq!(id, crate::ui::PaneId::Add),
            _ => panic!("Expected SwitchPane(add)"),
        }
    }

    #[test]
    fn next_prev_return_select_actions() {
        use crate::ui::action_id::{ActionId, TrackListActionId};
        let state = AppState::new();
        let mut pane = TrackListPane::new(Keymap::new());

        let action = pane.handle_action(
            ActionId::TrackList(TrackListActionId::Next),
            &dummy_event(),
            &state,
        );
        assert!(matches!(action, Action::Track(TrackAction::SelectNext)));

        let action = pane.handle_action(
            ActionId::TrackList(TrackListActionId::Prev),
            &dummy_event(),
            &state,
        );
        assert!(matches!(action, Action::Track(TrackAction::SelectPrev)));
    }

    #[test]
    fn link_mode_navigation_passes_through() {
        use crate::ui::action_id::{ActionId, TrackListActionId};
        let mut state = AppState::new();
        let id0 = state.add_track(SourceType::Saw);
        let _id1 = state.add_track(SourceType::Sin);

        // Select first instrument before entering link mode
        state.tracks.selected = Some(0);

        let mut pane = TrackListPane::new(Keymap::new());
        // Enter linking mode
        pane.handle_action(
            ActionId::TrackList(TrackListActionId::LinkLayer),
            &dummy_event(),
            &state,
        );
        assert_eq!(pane.linking_from, Some(id0));

        // Navigation should pass through (return SelectNext), not complete the link
        let action = pane.handle_action(
            ActionId::TrackList(TrackListActionId::Next),
            &dummy_event(),
            &state,
        );
        assert!(matches!(action, Action::Track(TrackAction::SelectNext)));
        // Should still be in linking mode
        assert_eq!(pane.linking_from, Some(id0));
    }

    #[test]
    fn link_mode_confirm_with_different_target() {
        use crate::ui::action_id::{ActionId, TrackListActionId};
        let mut state = AppState::new();
        let id0 = state.add_track(SourceType::Saw);
        let id1 = state.add_track(SourceType::Sin);

        // Select first instrument, enter linking mode
        state.tracks.selected = Some(0);
        let mut pane = TrackListPane::new(Keymap::new());
        pane.handle_action(
            ActionId::TrackList(TrackListActionId::LinkLayer),
            &dummy_event(),
            &state,
        );
        assert_eq!(pane.linking_from, Some(id0));

        // Move selection to second instrument
        state.tracks.selected = Some(1);

        // Press 'l' again to confirm
        let action = pane.handle_action(
            ActionId::TrackList(TrackListActionId::LinkLayer),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Track(TrackAction::LinkLayer(from, to)) => {
                assert_eq!(from, id0);
                assert_eq!(to, id1);
            }
            _ => panic!("Expected LinkLayer action, got {:?}", action),
        }
        assert!(pane.linking_from.is_none());
    }

    #[test]
    fn link_mode_confirm_same_instrument_no_action() {
        use crate::ui::action_id::{ActionId, TrackListActionId};
        let mut state = AppState::new();
        let _id0 = state.add_track(SourceType::Saw);

        let mut pane = TrackListPane::new(Keymap::new());
        // Enter linking mode
        pane.handle_action(
            ActionId::TrackList(TrackListActionId::LinkLayer),
            &dummy_event(),
            &state,
        );

        // Press 'l' again without moving — same instrument selected
        let action = pane.handle_action(
            ActionId::TrackList(TrackListActionId::LinkLayer),
            &dummy_event(),
            &state,
        );
        assert!(matches!(action, Action::None));
        assert!(pane.linking_from.is_none());
    }

    #[test]
    fn link_mode_cancelled_by_other_action() {
        use crate::ui::action_id::{ActionId, TrackListActionId};
        let mut state = AppState::new();
        let _id0 = state.add_track(SourceType::Saw);

        let mut pane = TrackListPane::new(Keymap::new());
        // Enter linking mode
        pane.handle_action(
            ActionId::TrackList(TrackListActionId::LinkLayer),
            &dummy_event(),
            &state,
        );
        assert!(pane.linking_from.is_some());

        // Any non-nav, non-link action should cancel
        pane.handle_action(
            ActionId::TrackList(TrackListActionId::Delete),
            &dummy_event(),
            &state,
        );
        assert!(pane.linking_from.is_none());
    }

    #[test]
    fn layer_octave_up_returns_adjust_action() {
        use crate::ui::action_id::{ActionId, TrackListActionId};
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);
        let mut pane = TrackListPane::new(Keymap::new());

        let action = pane.handle_action(
            ActionId::TrackList(TrackListActionId::LayerOctaveUp),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Track(TrackAction::AdjustLayerOctaveOffset(got_id, delta)) => {
                assert_eq!(got_id, id);
                assert_eq!(delta, 1);
            }
            _ => panic!("Expected AdjustLayerOctaveOffset(+1), got {:?}", action),
        }
    }

    #[test]
    fn layer_octave_down_returns_adjust_action() {
        use crate::ui::action_id::{ActionId, TrackListActionId};
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);
        let mut pane = TrackListPane::new(Keymap::new());

        let action = pane.handle_action(
            ActionId::TrackList(TrackListActionId::LayerOctaveDown),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Track(TrackAction::AdjustLayerOctaveOffset(got_id, delta)) => {
                assert_eq!(got_id, id);
                assert_eq!(delta, -1);
            }
            _ => panic!("Expected AdjustLayerOctaveOffset(-1), got {:?}", action),
        }
    }
}
