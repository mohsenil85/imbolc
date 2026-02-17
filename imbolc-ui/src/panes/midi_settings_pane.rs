use std::any::Any;

use crate::action::{Action, MidiAction};
use crate::state::automation::{AutomationTarget, AutomationTargetExt};
use crate::state::AppState;
use crate::ui::action_id::{ActionId, MidiSettingsActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::{Color, InputEvent, Keymap, Pane, Rect, RenderBuf, Style};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Section {
    Ports,
    CcMappings,
    Settings,
}

#[derive(Debug, Clone)]
enum TargetPickerState {
    Inactive,
    Active {
        options: Vec<AutomationTarget>,
        cursor: usize,
    },
}

pub struct MidiSettingsPane {
    keymap: Keymap,
    section: Section,
    port_cursor: usize,
    mapping_cursor: usize,
    target_picker: TargetPickerState,
}

impl MidiSettingsPane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            section: Section::Ports,
            port_cursor: 0,
            mapping_cursor: 0,
            target_picker: TargetPickerState::Inactive,
        }
    }

    fn handle_target_picker_action(&mut self, action: ActionId, _state: &AppState) -> Action {
        if let TargetPickerState::Active {
            ref options,
            ref mut cursor,
        } = self.target_picker
        {
            match action {
                ActionId::MidiSettings(MidiSettingsActionId::Up) => {
                    *cursor = cursor.saturating_sub(1);
                    Action::None
                }
                ActionId::MidiSettings(MidiSettingsActionId::Down) => {
                    let max = options.len().saturating_sub(1);
                    *cursor = (*cursor + 1).min(max);
                    Action::None
                }
                ActionId::MidiSettings(MidiSettingsActionId::Confirm)
                | ActionId::MidiSettings(MidiSettingsActionId::Connect) => {
                    if let Some(target) = options.get(*cursor).cloned() {
                        self.target_picker = TargetPickerState::Inactive;
                        Action::Midi(MidiAction::StartLearn(target))
                    } else {
                        Action::None
                    }
                }
                ActionId::MidiSettings(MidiSettingsActionId::Cancel) => {
                    self.target_picker = TargetPickerState::Inactive;
                    Action::None
                }
                _ => Action::None,
            }
        } else {
            Action::None
        }
    }

    fn render_target_picker(&self, buf: &mut RenderBuf, area: Rect, state: &AppState) {
        if let TargetPickerState::Active {
            ref options,
            cursor,
        } = self.target_picker
        {
            let picker_width = 40u16.min(area.width.saturating_sub(4));
            let picker_height = (options.len() as u16 + 2).min(area.height.saturating_sub(2));
            let picker_rect = center_rect(area, picker_width, picker_height);

            let clear_style = Style::new().bg(Color::new(20, 20, 30));
            for y in picker_rect.y..picker_rect.y + picker_rect.height {
                for x in picker_rect.x..picker_rect.x + picker_rect.width {
                    buf.set_cell(x, y, ' ', clear_style);
                }
            }

            let border_style = Style::new().fg(Color::CYAN);
            let title_style = Style::new().fg(Color::CYAN);
            let inner = buf.draw_block(
                picker_rect,
                " MIDI Learn Target ",
                border_style,
                title_style,
            );

            let inst = state.instruments.selected_instrument();
            let vst_registry = &state.session.vst_plugins;

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
                    Style::new().fg(Color::WHITE).bg(Color::SELECTION_BG).bold()
                } else {
                    Style::new().fg(Color::GRAY)
                };

                for (j, ch) in text.chars().enumerate() {
                    let x = inner.x + j as u16;
                    if x >= inner.x + inner.width {
                        break;
                    }
                    buf.set_cell(x, y, ch, style);
                }
                if is_selected {
                    for x_fill in (inner.x + text.len() as u16)..inner.x + inner.width {
                        buf.set_cell(x_fill, y, ' ', style);
                    }
                }
            }
        }
    }
}

impl Pane for MidiSettingsPane {
    fn id(&self) -> &'static str {
        "midi_settings"
    }

    fn handle_action(&mut self, action: ActionId, _event: &InputEvent, state: &AppState) -> Action {
        // If target picker is active, delegate to it
        if matches!(self.target_picker, TargetPickerState::Active { .. }) {
            return self.handle_target_picker_action(action, state);
        }

        // If in learn mode (waiting for CC), Escape cancels
        if state.session.midi_recording.is_learning() {
            if matches!(action, ActionId::MidiSettings(MidiSettingsActionId::Cancel)) {
                return Action::Midi(MidiAction::CancelLearn);
            }
            // Block other actions while listening
            return Action::None;
        }

        match action {
            ActionId::MidiSettings(MidiSettingsActionId::SwitchSection) => {
                self.section = match self.section {
                    Section::Ports => Section::CcMappings,
                    Section::CcMappings => Section::Settings,
                    Section::Settings => Section::Ports,
                };
                Action::None
            }
            ActionId::MidiSettings(MidiSettingsActionId::Up) => {
                match self.section {
                    Section::Ports => {
                        self.port_cursor = self.port_cursor.saturating_sub(1);
                    }
                    Section::CcMappings => {
                        self.mapping_cursor = self.mapping_cursor.saturating_sub(1);
                    }
                    Section::Settings => {}
                }
                Action::None
            }
            ActionId::MidiSettings(MidiSettingsActionId::Down) => {
                match self.section {
                    Section::Ports => {
                        let max = state.midi.port_names.len().saturating_sub(1);
                        self.port_cursor = (self.port_cursor + 1).min(max);
                    }
                    Section::CcMappings => {
                        let max = state
                            .session
                            .midi_recording
                            .cc_mappings
                            .len()
                            .saturating_sub(1);
                        self.mapping_cursor = (self.mapping_cursor + 1).min(max);
                    }
                    Section::Settings => {}
                }
                Action::None
            }
            ActionId::MidiSettings(MidiSettingsActionId::Connect) => {
                if self.section == Section::Ports && !state.midi.port_names.is_empty() {
                    Action::Midi(MidiAction::ConnectPort(self.port_cursor))
                } else {
                    Action::None
                }
            }
            ActionId::MidiSettings(MidiSettingsActionId::Disconnect) => {
                Action::Midi(MidiAction::DisconnectPort)
            }
            ActionId::MidiSettings(MidiSettingsActionId::RemoveMapping) => {
                if self.section == Section::CcMappings {
                    let mappings = &state.session.midi_recording.cc_mappings;
                    if let Some(m) = mappings.get(self.mapping_cursor) {
                        let cc = m.cc_number;
                        let ch = m.channel;
                        return Action::Midi(MidiAction::RemoveCcMapping { cc, channel: ch });
                    }
                }
                Action::None
            }
            ActionId::MidiSettings(MidiSettingsActionId::TogglePassthrough) => {
                Action::Midi(MidiAction::ToggleNotePassthrough)
            }
            ActionId::MidiSettings(MidiSettingsActionId::SetChannelAll) => {
                Action::Midi(MidiAction::SetChannelFilter(None))
            }
            ActionId::MidiSettings(MidiSettingsActionId::SetLiveInstrument) => {
                if let Some(inst) = state.instruments.selected_instrument() {
                    Action::Midi(MidiAction::SetLiveInputInstrument(Some(inst.id)))
                } else {
                    Action::None
                }
            }
            ActionId::MidiSettings(MidiSettingsActionId::ClearLiveInstrument) => {
                Action::Midi(MidiAction::SetLiveInputInstrument(None))
            }
            ActionId::MidiSettings(MidiSettingsActionId::Learn) => {
                if self.section != Section::CcMappings {
                    return Action::None;
                }
                // Build target list from selected instrument
                let mut options = Vec::new();
                if let Some(inst) = state.instruments.selected_instrument() {
                    options = AutomationTarget::targets_for_instrument_context(
                        inst,
                        &state.session.vst_plugins,
                    );
                }
                if options.is_empty() {
                    return Action::None;
                }
                self.target_picker = TargetPickerState::Active { options, cursor: 0 };
                Action::None
            }
            _ => Action::None,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let border_style = Style::new().fg(Color::CYAN);
        let inner = buf.draw_block(area, " MIDI Settings ", border_style, border_style);

        if inner.height < 3 || inner.width < 20 {
            return;
        }

        let section_style = |s: Section| {
            if s == self.section {
                Style::new().fg(Color::CYAN).bold()
            } else {
                Style::new().fg(Color::DARK_GRAY)
            }
        };
        let normal = Style::new().fg(Color::GRAY);
        let dim = Style::new().fg(Color::DARK_GRAY);
        let highlight = Style::new().fg(Color::WHITE).bg(Color::SELECTION_BG).bold();

        let mut y = inner.y;
        let x = inner.x + 1;
        let w = inner.width.saturating_sub(2);

        // Learn mode banner
        if state.session.midi_recording.is_learning() {
            let learn_style = Style::new().fg(Color::BLACK).bg(Color::MIDI_COLOR).bold();
            let target_name = state
                .session
                .midi_recording
                .learn_target
                .as_ref()
                .map(|t| t.name())
                .unwrap_or_default();
            let banner = format!(
                " MIDI Learn: move a CC knob for {} (Esc to cancel) ",
                target_name
            );
            buf.draw_line(Rect::new(x, y, w, 1), &[(&banner, learn_style)]);
            y += 2;
        }

        // Section: Ports
        if y >= inner.y + inner.height {
            return;
        }
        let conn_text = if let Some(ref name) = state.midi.connected_port {
            format!("  [Connected: {}]", name)
        } else {
            "  [Not connected]".to_string()
        };
        buf.draw_line(
            Rect::new(x, y, w, 1),
            &[
                (" Ports ", section_style(Section::Ports)),
                (&conn_text, dim),
            ],
        );
        y += 1;

        if self.section == Section::Ports {
            if state.midi.port_names.is_empty() {
                buf.draw_line(Rect::new(x, y, w, 1), &[("  (no MIDI ports found)", dim)]);
                y += 1;
            } else {
                for (i, name) in state.midi.port_names.iter().enumerate() {
                    if y >= inner.y + inner.height {
                        break;
                    }
                    let is_connected = state.midi.connected_port.as_deref() == Some(name);
                    let prefix = if is_connected { " * " } else { "   " };
                    let text = format!("{}{}", prefix, name);
                    let style = if i == self.port_cursor {
                        highlight
                    } else {
                        normal
                    };
                    buf.draw_line(Rect::new(x, y, w, 1), &[(&text, style)]);
                    y += 1;
                }
            }
        }
        y += 1;

        // Section: CC Mappings
        if y >= inner.y + inner.height {
            return;
        }
        let mapping_title = format!(
            " CC Mappings ({})  [l]earn",
            state.session.midi_recording.cc_mappings.len()
        );
        buf.draw_line(
            Rect::new(x, y, w, 1),
            &[(&mapping_title, section_style(Section::CcMappings))],
        );
        y += 1;

        if self.section == Section::CcMappings {
            if state.session.midi_recording.cc_mappings.is_empty() {
                if y < inner.y + inner.height {
                    buf.draw_line(
                        Rect::new(x, y, w, 1),
                        &[("  (no CC mappings — press l to learn)", dim)],
                    );
                    y += 1;
                }
            } else {
                for (i, mapping) in state.session.midi_recording.cc_mappings.iter().enumerate() {
                    if y >= inner.y + inner.height {
                        break;
                    }
                    let ch_str = match mapping.channel {
                        Some(ch) => format!("ch{}", ch + 1),
                        None => "any".to_string(),
                    };
                    let text = format!(
                        "  CC{:<3} {} -> {}",
                        mapping.cc_number,
                        ch_str,
                        mapping.target.name()
                    );
                    let style = if i == self.mapping_cursor {
                        highlight
                    } else {
                        normal
                    };
                    buf.draw_line(Rect::new(x, y, w, 1), &[(&text, style)]);
                    y += 1;
                }
            }
        }
        y += 1;

        // Section: Settings
        if y >= inner.y + inner.height {
            return;
        }
        buf.draw_line(
            Rect::new(x, y, w, 1),
            &[(" Settings", section_style(Section::Settings))],
        );
        y += 1;

        if self.section == Section::Settings {
            let settings = [
                format!(
                    "  Note passthrough: {}",
                    if state.session.midi_recording.note_passthrough {
                        "ON"
                    } else {
                        "OFF"
                    }
                ),
                format!(
                    "  Channel filter: {}",
                    match state.session.midi_recording.channel_filter {
                        Some(ch) => format!("Ch {}", ch + 1),
                        None => "All".to_string(),
                    }
                ),
                format!(
                    "  Live input instrument: {}",
                    match state.session.midi_recording.live_input_instrument {
                        Some(id) => {
                            state
                                .instruments
                                .instruments
                                .iter()
                                .find(|i| i.id == id)
                                .map(|i| i.name.clone())
                                .unwrap_or_else(|| format!("#{}", id))
                        }
                        None => "(selected)".to_string(),
                    }
                ),
            ];

            for line in &settings {
                if y >= inner.y + inner.height {
                    break;
                }
                buf.draw_line(Rect::new(x, y, w, 1), &[(line.as_str(), normal)]);
                y += 1;
            }
        }

        // Render target picker overlay (if active)
        self.render_target_picker(buf, area, state);
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
    use super::*;
    use crate::ui::action_id::ActionId;
    use crate::ui::input::{KeyCode, Modifiers};

    fn dummy_event() -> InputEvent {
        InputEvent::new(KeyCode::Char('x'), Modifiers::default())
    }

    #[test]
    fn midi_settings_pane_id() {
        let pane = MidiSettingsPane::new(Keymap::new());
        assert_eq!(pane.id(), "midi_settings");
    }

    #[test]
    fn learn_opens_target_picker_in_cc_section() {
        let mut pane = MidiSettingsPane::new(Keymap::new());
        let mut state = AppState::new();
        state.add_instrument(crate::state::SourceType::Saw);
        pane.section = Section::CcMappings;

        pane.handle_action(
            ActionId::MidiSettings(MidiSettingsActionId::Learn),
            &dummy_event(),
            &state,
        );
        assert!(matches!(
            pane.target_picker,
            TargetPickerState::Active { .. }
        ));
    }

    #[test]
    fn learn_does_nothing_outside_cc_section() {
        let mut pane = MidiSettingsPane::new(Keymap::new());
        let mut state = AppState::new();
        state.add_instrument(crate::state::SourceType::Saw);
        pane.section = Section::Ports;

        pane.handle_action(
            ActionId::MidiSettings(MidiSettingsActionId::Learn),
            &dummy_event(),
            &state,
        );
        assert!(matches!(pane.target_picker, TargetPickerState::Inactive));
    }

    #[test]
    fn target_picker_confirm_returns_start_learn() {
        let mut pane = MidiSettingsPane::new(Keymap::new());
        let mut state = AppState::new();
        state.add_instrument(crate::state::SourceType::Saw);
        pane.section = Section::CcMappings;

        pane.handle_action(
            ActionId::MidiSettings(MidiSettingsActionId::Learn),
            &dummy_event(),
            &state,
        );

        let action = pane.handle_action(
            ActionId::MidiSettings(MidiSettingsActionId::Confirm),
            &dummy_event(),
            &state,
        );
        assert!(matches!(action, Action::Midi(MidiAction::StartLearn(_))));
        assert!(matches!(pane.target_picker, TargetPickerState::Inactive));
    }

    #[test]
    fn target_picker_cancel_closes_picker() {
        let mut pane = MidiSettingsPane::new(Keymap::new());
        let mut state = AppState::new();
        state.add_instrument(crate::state::SourceType::Saw);
        pane.section = Section::CcMappings;

        pane.handle_action(
            ActionId::MidiSettings(MidiSettingsActionId::Learn),
            &dummy_event(),
            &state,
        );

        let action = pane.handle_action(
            ActionId::MidiSettings(MidiSettingsActionId::Cancel),
            &dummy_event(),
            &state,
        );
        assert!(matches!(action, Action::None));
        assert!(matches!(pane.target_picker, TargetPickerState::Inactive));
    }

    #[test]
    fn cancel_during_learn_mode_returns_cancel_learn() {
        let mut pane = MidiSettingsPane::new(Keymap::new());
        let mut state = AppState::new();
        state.add_instrument(crate::state::SourceType::Saw);
        let target = crate::state::automation::AutomationTarget::filter_cutoff(
            imbolc_types::InstrumentId::new(0),
        );
        state.session.midi_recording.start_learning(target);

        let action = pane.handle_action(
            ActionId::MidiSettings(MidiSettingsActionId::Cancel),
            &dummy_event(),
            &state,
        );
        assert!(matches!(action, Action::Midi(MidiAction::CancelLearn)));
    }

    #[test]
    fn actions_blocked_during_learn_mode() {
        let mut pane = MidiSettingsPane::new(Keymap::new());
        let mut state = AppState::new();
        state.add_instrument(crate::state::SourceType::Saw);
        let target = crate::state::automation::AutomationTarget::filter_cutoff(
            imbolc_types::InstrumentId::new(0),
        );
        state.session.midi_recording.start_learning(target);

        // Tab should be blocked during learn
        let action = pane.handle_action(
            ActionId::MidiSettings(MidiSettingsActionId::SwitchSection),
            &dummy_event(),
            &state,
        );
        assert!(matches!(action, Action::None));
    }
}
