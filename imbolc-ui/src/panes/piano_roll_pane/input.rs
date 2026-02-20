use crate::state::automation::{AutomationTarget, AutomationTargetExt, CurveType};
use crate::state::AppState;
use crate::ui::action_id::{ActionId, ModeActionId, PianoRollActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::{
    translate_key, Action, ArrangementAction, AutomationAction, InputEvent, KeyCode, MouseButton,
    MouseEvent, MouseEventKind, PianoRollAction, Rect,
};
use imbolc_types::TrackId;

use super::{PianoRollPane, TargetPickerState};

impl PianoRollPane {
    /// Get the instrument ID for the current track from state
    fn current_instrument_id(&self, state: &AppState) -> TrackId {
        state
            .session
            .piano_roll
            .sequence_order
            .get(self.current_track)
            .copied()
            .unwrap_or(TrackId::new(0))
    }
}

impl PianoRollPane {
    pub(super) fn handle_action_impl(
        &mut self,
        action: ActionId,
        event: &InputEvent,
        state: &AppState,
    ) -> Action {
        match action {
            // Target picker intercepts all actions
            _ if matches!(self.target_picker, TargetPickerState::Active { .. }) => {
                self.handle_target_picker_action(action, event, state)
            }
            // When automation is visible, let automation command keys win over
            // piano/pad performance key events from mode layers.
            ActionId::Mode(mode_action @ (ModeActionId::PianoKey | ModeActionId::PadKey))
                if self.automation_visible =>
            {
                if matches!(
                    event.key,
                    KeyCode::Char('d' | 'c' | 'C' | 'a' | 'x' | 'e' | 'r' | 'R')
                ) {
                    self.handle_automation_raw_key(event, state)
                } else {
                    self.handle_piano_mode_action(ActionId::Mode(mode_action), event, state)
                }
            }
            // Piano mode actions always pass through (from piano layer)
            ActionId::Mode(_) => self.handle_piano_mode_action(action, event, state),
            // Automation focus routes to automation handler
            _ if self.automation_visible && self.automation_focus => {
                self.handle_automation_action(action, event, state)
            }
            // Note editor handling
            _ => self.handle_note_editor_action(action, event, state),
        }
    }

    fn handle_piano_mode_action(
        &mut self,
        action: ActionId,
        event: &InputEvent,
        state: &AppState,
    ) -> Action {
        match action {
            ActionId::Mode(ModeActionId::PianoEscape) => {
                self.piano.deactivate();
                Action::ExitPerformanceMode
            }
            ActionId::Mode(ModeActionId::PianoOctaveDown) => {
                if self.piano.octave_down() {
                    self.center_view_on_piano_octave();
                }
                Action::None
            }
            ActionId::Mode(ModeActionId::PianoOctaveUp) => {
                if self.piano.octave_up() {
                    self.center_view_on_piano_octave();
                }
                Action::None
            }
            ActionId::Mode(ModeActionId::PianoSpace) => {
                Action::PianoRoll(PianoRollAction::ToggleRecordPlayback)
            }
            ActionId::Mode(ModeActionId::PianoKey) => {
                if let KeyCode::Char(c) = event.key {
                    let c = translate_key(c, state.keyboard_layout);
                    if let Some(pitches) = self.piano.key_to_pitches(c) {
                        if let Some(new_pitches) = self.piano.key_pressed(
                            c,
                            pitches.clone(),
                            event.timestamp,
                            event.is_repeat,
                        ) {
                            let instrument_id = self.current_instrument_id(state);
                            let track = self.current_track;
                            if new_pitches.len() == 1 {
                                return Action::PianoRoll(PianoRollAction::PlayNote {
                                    pitch: new_pitches[0],
                                    velocity: 100,
                                    instrument_id,
                                    track,
                                });
                            } else {
                                return Action::PianoRoll(PianoRollAction::PlayNotes {
                                    pitches: new_pitches,
                                    velocity: 100,
                                    instrument_id,
                                    track,
                                });
                            }
                        }
                    }
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    /// Handle actions when automation half has focus.
    fn handle_automation_action(
        &mut self,
        action: ActionId,
        event: &InputEvent,
        state: &AppState,
    ) -> Action {
        match action {
            // Navigation
            ActionId::PianoRoll(PianoRollActionId::Up) => {
                self.automation_cursor_value = (self.automation_cursor_value + 0.05).min(1.0);
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::Down) => {
                self.automation_cursor_value = (self.automation_cursor_value - 0.05).max(0.0);
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::Right) => {
                self.automation_selection_anchor_tick = None;
                let tpc = self.ticks_per_cell();
                self.automation_cursor_tick += tpc;
                self.scroll_automation_cursor_into_view();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::Left) => {
                self.automation_selection_anchor_tick = None;
                let tpc = self.ticks_per_cell();
                self.automation_cursor_tick = self.automation_cursor_tick.saturating_sub(tpc);
                self.scroll_automation_cursor_into_view();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::SelectRight) => {
                if self.automation_selection_anchor_tick.is_none() {
                    self.automation_selection_anchor_tick = Some(self.automation_cursor_tick);
                }
                let tpc = self.ticks_per_cell();
                self.automation_cursor_tick += tpc;
                self.scroll_automation_cursor_into_view();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::SelectLeft) => {
                if self.automation_selection_anchor_tick.is_none() {
                    self.automation_selection_anchor_tick = Some(self.automation_cursor_tick);
                }
                let tpc = self.ticks_per_cell();
                self.automation_cursor_tick = self.automation_cursor_tick.saturating_sub(tpc);
                self.scroll_automation_cursor_into_view();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::Home) => {
                self.automation_cursor_tick = 0;
                self.view_start_tick = 0;
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::End) => {
                // Jump to last point
                if let Some(lane) = state.session.automation.selected() {
                    if let Some(last) = lane.points.last() {
                        self.automation_cursor_tick = last.tick;
                        self.scroll_automation_cursor_into_view();
                    }
                }
                Action::None
            }

            // Place/remove point (Enter)
            ActionId::PianoRoll(PianoRollActionId::ToggleNote) => {
                if let Some(id) = self.selected_lane_id(state) {
                    let tick = crate::state::grid::snap_to_grid(
                        self.automation_cursor_tick,
                        self.zoom_level,
                    );
                    if let Some(lane) = state.session.automation.lane(id) {
                        if lane.point_at(tick).is_some() {
                            return Action::Automation(AutomationAction::RemovePoint(id, tick));
                        } else {
                            return Action::Automation(AutomationAction::AddPoint(
                                id,
                                tick,
                                self.automation_cursor_value,
                            ));
                        }
                    }
                }
                Action::None
            }

            // Zoom (shared with note editor)
            ActionId::PianoRoll(PianoRollActionId::ZoomIn) => {
                if self.zoom_level > 1 {
                    self.zoom_level -= 1;
                    self.cursor_tick = self.snap_tick(self.cursor_tick);
                    self.scroll_to_cursor();
                }
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::ZoomOut) => {
                if self.zoom_level < 5 {
                    self.zoom_level += 1;
                    self.cursor_tick = self.snap_tick(self.cursor_tick);
                    self.scroll_to_cursor();
                }
                Action::None
            }

            // Passthrough actions
            ActionId::PianoRoll(PianoRollActionId::TogglePlayback) => {
                Action::PianoRoll(PianoRollAction::TogglePlayback)
            }
            ActionId::PianoRoll(PianoRollActionId::Loop) => {
                Action::PianoRoll(PianoRollAction::ToggleLoop)
            }
            ActionId::PianoRoll(PianoRollActionId::LoopStart) => {
                Action::PianoRoll(PianoRollAction::SetLoopStart(self.automation_cursor_tick))
            }
            ActionId::PianoRoll(PianoRollActionId::LoopEnd) => {
                Action::PianoRoll(PianoRollAction::SetLoopEnd(self.automation_cursor_tick))
            }

            // Toggle automation off
            ActionId::PianoRoll(PianoRollActionId::ToggleAutomation) => {
                self.automation_visible = false;
                self.automation_focus = false;
                Action::None
            }

            // Lane navigation
            ActionId::PianoRoll(PianoRollActionId::AutomationLanePrev) => {
                Action::Automation(AutomationAction::SelectLane(-1))
            }
            ActionId::PianoRoll(PianoRollActionId::AutomationLaneNext) => {
                Action::Automation(AutomationAction::SelectLane(1))
            }

            // Switch focus back to notes
            ActionId::PianoRoll(PianoRollActionId::AutomationSwitchFocus) => {
                self.automation_focus = false;
                Action::None
            }

            // Automation-specific actions — handled via raw key check
            // since they share keys with note-editor bindings
            _ => self.handle_automation_raw_key(event, state),
        }
    }

    /// Handle automation-specific keys that conflict with note-editor bindings.
    /// These use raw key inspection since the keybinding layer can't have duplicate keys.
    pub(super) fn handle_automation_raw_key(
        &mut self,
        event: &InputEvent,
        state: &AppState,
    ) -> Action {
        match event.key {
            KeyCode::Char('d') => {
                // Delete point
                if let Some(id) = self.selected_lane_id(state) {
                    let tick = crate::state::grid::snap_to_grid(
                        self.automation_cursor_tick,
                        self.zoom_level,
                    );
                    Action::Automation(AutomationAction::RemovePoint(id, tick))
                } else {
                    Action::None
                }
            }
            KeyCode::Char('c') => {
                // Cycle curve type
                if let Some(id) = self.selected_lane_id(state) {
                    let tick = crate::state::grid::snap_to_grid(
                        self.automation_cursor_tick,
                        self.zoom_level,
                    );
                    if let Some(lane) = state.session.automation.lane(id) {
                        if let Some(point) = lane.point_at(tick) {
                            let new_curve = match point.curve {
                                CurveType::Linear => CurveType::Exponential,
                                CurveType::Exponential => CurveType::Step,
                                CurveType::Step => CurveType::SCurve,
                                CurveType::SCurve => CurveType::Linear,
                            };
                            return Action::Automation(AutomationAction::SetCurveType(
                                id, tick, new_curve,
                            ));
                        }
                    }
                }
                Action::None
            }
            KeyCode::Char('C') => {
                // Clear lane
                if let Some(id) = self.selected_lane_id(state) {
                    Action::Automation(AutomationAction::ClearLane(id))
                } else {
                    Action::None
                }
            }
            KeyCode::Char('a') => {
                // Add lane — open target picker
                self.open_target_picker(state);
                Action::None
            }
            KeyCode::Char('x') => {
                // Remove lane
                if let Some(id) = self.selected_lane_id(state) {
                    Action::Automation(AutomationAction::RemoveLane(id))
                } else {
                    Action::None
                }
            }
            KeyCode::Char('e') => {
                // Toggle enabled
                if let Some(id) = self.selected_lane_id(state) {
                    Action::Automation(AutomationAction::ToggleLaneEnabled(id))
                } else {
                    Action::None
                }
            }
            KeyCode::Char('r') => {
                // Toggle recording
                Action::Automation(AutomationAction::ToggleRecording)
            }
            KeyCode::Char('R') => {
                // Toggle arm
                if let Some(id) = self.selected_lane_id(state) {
                    Action::Automation(AutomationAction::ToggleLaneArm(id))
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        }
    }

    fn open_target_picker(&mut self, state: &AppState) {
        let editing_clip = state.session.arrangement.editing_clip.is_some();
        let mut options: Vec<AutomationTarget> = Vec::new();
        if let Some(inst) = state.tracks.selected_track() {
            options =
                AutomationTarget::targets_for_instrument_context(inst, &state.session.vst_plugins);
            let id = inst.id;
            for &bus_id in inst.channel_strip.sends.keys() {
                options.push(AutomationTarget::send_level(id, bus_id));
            }
        }
        if !editing_clip {
            for bus_id in state.session.bus_ids() {
                options.push(AutomationTarget::bus_level(bus_id));
            }
            options.push(AutomationTarget::bpm());
            options.extend(AutomationTarget::targets_for_generative());
        }
        self.target_picker = TargetPickerState::Active { options, cursor: 0 };
    }

    /// Handle actions while the target picker is active
    fn handle_target_picker_action(
        &mut self,
        action: ActionId,
        _event: &InputEvent,
        _state: &AppState,
    ) -> Action {
        if let TargetPickerState::Active {
            ref options,
            ref mut cursor,
        } = self.target_picker
        {
            match action {
                ActionId::PianoRoll(PianoRollActionId::Up) => {
                    if *cursor > 0 {
                        *cursor -= 1;
                    }
                    Action::None
                }
                ActionId::PianoRoll(PianoRollActionId::Down) => {
                    if *cursor + 1 < options.len() {
                        *cursor += 1;
                    }
                    Action::None
                }
                ActionId::PianoRoll(PianoRollActionId::ToggleNote) => {
                    // Enter confirms
                    if let Some(target) = options.get(*cursor).cloned() {
                        self.target_picker = TargetPickerState::Inactive;
                        Action::Automation(AutomationAction::AddLane(target))
                    } else {
                        Action::None
                    }
                }
                ActionId::Global(crate::ui::action_id::GlobalActionId::Escape) => {
                    self.target_picker = TargetPickerState::Inactive;
                    Action::None
                }
                _ => Action::None,
            }
        } else {
            Action::None
        }
    }

    /// Handle actions in normal note editor mode (original behavior).
    fn handle_note_editor_action(
        &mut self,
        action: ActionId,
        _event: &InputEvent,
        state: &AppState,
    ) -> Action {
        // Kit tracks are read-only — block edit actions
        if self.is_kit_track(state) {
            match action {
                // Allow navigation and view actions
                ActionId::PianoRoll(
                    PianoRollActionId::Up
                    | PianoRollActionId::Down
                    | PianoRollActionId::Left
                    | PianoRollActionId::Right
                    | PianoRollActionId::OctaveUp
                    | PianoRollActionId::OctaveDown
                    | PianoRollActionId::Home
                    | PianoRollActionId::End
                    | PianoRollActionId::ZoomIn
                    | PianoRollActionId::ZoomOut
                    | PianoRollActionId::TogglePlayback
                    | PianoRollActionId::Loop
                    | PianoRollActionId::LoopStart
                    | PianoRollActionId::LoopEnd
                    | PianoRollActionId::ToggleAutomation
                    | PianoRollActionId::AutomationLanePrev
                    | PianoRollActionId::AutomationLaneNext
                    | PianoRollActionId::AutomationSwitchFocus
                    | PianoRollActionId::BounceToWav
                    | PianoRollActionId::ExportStems,
                ) => {} // fall through to normal handling
                // Block all edit actions
                _ => return Action::None,
            }
        }

        match action {
            ActionId::PianoRoll(PianoRollActionId::Up) => {
                self.selection_anchor = None;
                if self.cursor_pitch < 127 {
                    self.cursor_pitch += 1;
                    self.scroll_to_cursor();
                }
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::Down) => {
                self.selection_anchor = None;
                if self.cursor_pitch > 0 {
                    self.cursor_pitch -= 1;
                    self.scroll_to_cursor();
                }
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::Right) => {
                self.selection_anchor = None;
                self.cursor_tick += self.ticks_per_cell();
                self.scroll_to_cursor();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::Left) => {
                self.selection_anchor = None;
                let step = self.ticks_per_cell();
                self.cursor_tick = self.cursor_tick.saturating_sub(step);
                self.scroll_to_cursor();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::SelectUp) => {
                if self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_tick, self.cursor_pitch));
                }
                if self.cursor_pitch < 127 {
                    self.cursor_pitch += 1;
                    self.scroll_to_cursor();
                }
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::SelectDown) => {
                if self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_tick, self.cursor_pitch));
                }
                if self.cursor_pitch > 0 {
                    self.cursor_pitch -= 1;
                    self.scroll_to_cursor();
                }
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::SelectRight) => {
                if self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_tick, self.cursor_pitch));
                }
                self.cursor_tick += self.ticks_per_cell();
                self.scroll_to_cursor();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::SelectLeft) => {
                if self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_tick, self.cursor_pitch));
                }
                let step = self.ticks_per_cell();
                self.cursor_tick = self.cursor_tick.saturating_sub(step);
                self.scroll_to_cursor();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::ToggleNote) => {
                Action::PianoRoll(PianoRollAction::ToggleNote {
                    pitch: self.cursor_pitch,
                    tick: self.cursor_tick,
                    duration: self.default_duration,
                    velocity: self.default_velocity,
                    track: self.current_track,
                })
            }
            ActionId::PianoRoll(PianoRollActionId::GrowDuration) => {
                self.adjust_default_duration(self.ticks_per_cell() as i32);
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::ShrinkDuration) => {
                self.adjust_default_duration(-(self.ticks_per_cell() as i32));
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::VelUp) => {
                self.adjust_default_velocity(10);
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::VelDown) => {
                self.adjust_default_velocity(-10);
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::TogglePlayback) => {
                Action::PianoRoll(PianoRollAction::TogglePlayback)
            }
            ActionId::PianoRoll(PianoRollActionId::Loop) => {
                Action::PianoRoll(PianoRollAction::ToggleLoop)
            }
            ActionId::PianoRoll(PianoRollActionId::LoopStart) => {
                Action::PianoRoll(PianoRollAction::SetLoopStart(self.cursor_tick))
            }
            ActionId::PianoRoll(PianoRollActionId::LoopEnd) => {
                Action::PianoRoll(PianoRollAction::SetLoopEnd(self.cursor_tick))
            }
            ActionId::PianoRoll(PianoRollActionId::OctaveUp) => {
                self.selection_anchor = None;
                self.cursor_pitch = (self.cursor_pitch as i16 + 12).min(127) as u8;
                self.scroll_to_cursor();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::OctaveDown) => {
                self.selection_anchor = None;
                self.cursor_pitch = (self.cursor_pitch as i16 - 12).max(0) as u8;
                self.scroll_to_cursor();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::Home) => {
                self.selection_anchor = None;
                self.cursor_tick = 0;
                self.view_start_tick = 0;
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::End) => {
                self.selection_anchor = None;
                self.jump_to_end();
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::ZoomIn) => {
                if self.zoom_level > 1 {
                    self.zoom_level -= 1;
                    self.cursor_tick = self.snap_tick(self.cursor_tick);
                    self.scroll_to_cursor();
                }
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::ZoomOut) => {
                if self.zoom_level < 5 {
                    self.zoom_level += 1;
                    self.cursor_tick = self.snap_tick(self.cursor_tick);
                    self.scroll_to_cursor();
                }
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::TimeSig) => {
                Action::PianoRoll(PianoRollAction::NextTimeSig)
            }
            ActionId::PianoRoll(PianoRollActionId::TogglePoly) => {
                Action::PianoRoll(PianoRollAction::TogglePolyMode(self.current_track))
            }
            ActionId::PianoRoll(PianoRollActionId::RenderToWav) => Action::PianoRoll(
                PianoRollAction::RenderToWav(self.current_instrument_id(state)),
            ),
            ActionId::PianoRoll(PianoRollActionId::BounceToWav) => {
                if state.io.pending_export.is_some() {
                    Action::PianoRoll(PianoRollAction::CancelExport)
                } else {
                    Action::PianoRoll(PianoRollAction::BounceToWav)
                }
            }
            ActionId::PianoRoll(PianoRollActionId::ExportStems) => {
                if state.io.pending_export.is_some() {
                    Action::PianoRoll(PianoRollAction::CancelExport)
                } else {
                    Action::PianoRoll(PianoRollAction::ExportStems)
                }
            }
            ActionId::PianoRoll(PianoRollActionId::ToggleAutomation) => {
                self.automation_visible = !self.automation_visible;
                if self.automation_visible {
                    self.automation_focus = true;
                    self.automation_cursor_tick = self.cursor_tick;
                } else {
                    self.automation_focus = false;
                }
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::AutomationLanePrev) => {
                if self.automation_visible {
                    Action::Automation(AutomationAction::SelectLane(-1))
                } else {
                    Action::None
                }
            }
            ActionId::PianoRoll(PianoRollActionId::AutomationLaneNext) => {
                if self.automation_visible {
                    Action::Automation(AutomationAction::SelectLane(1))
                } else {
                    Action::None
                }
            }
            ActionId::PianoRoll(PianoRollActionId::AutomationSwitchFocus) => {
                if self.automation_visible {
                    self.automation_focus = true;
                    self.automation_cursor_tick = self.cursor_tick;
                }
                Action::None
            }
            ActionId::PianoRoll(PianoRollActionId::NextClip) => {
                let instrument_id = self.current_instrument_id(state);
                Action::Arrangement(ArrangementAction::NextClip(instrument_id))
            }
            ActionId::PianoRoll(PianoRollActionId::PrevClip) => {
                let instrument_id = self.current_instrument_id(state);
                Action::Arrangement(ArrangementAction::PrevClip(instrument_id))
            }
            // Automation-specific action IDs are no-ops when notes are focused
            ActionId::PianoRoll(
                PianoRollActionId::AutomationDeletePoint
                | PianoRollActionId::AutomationCycleCurve
                | PianoRollActionId::AutomationClearLane
                | PianoRollActionId::AutomationAddLane
                | PianoRollActionId::AutomationRemoveLane
                | PianoRollActionId::AutomationToggleEnabled
                | PianoRollActionId::AutomationToggleRecording
                | PianoRollActionId::AutomationToggleArm,
            ) => Action::None,
            _ => Action::None,
        }
    }

    pub(super) fn handle_mouse_impl(
        &mut self,
        event: &MouseEvent,
        area: Rect,
        state: &AppState,
    ) -> Action {
        if self.automation_visible {
            // Compute split rects
            let total_height = 42u16.min(area.height.saturating_sub(2));
            let rect = center_rect(area, 97, total_height);
            let note_height = (total_height * 3 / 5)
                .max(15)
                .min(total_height.saturating_sub(6));
            let auto_y = rect.y + note_height + 1;

            if event.row >= auto_y {
                // Click in automation area
                self.automation_focus = true;
                // Map click to tick/value
                let key_col_width: u16 = 5;
                let grid_x = rect.x + key_col_width;
                let grid_width = rect.width.saturating_sub(key_col_width + 1);
                let auto_height = total_height.saturating_sub(note_height + 1);
                let graph_height = auto_height.saturating_sub(2);

                if event.column >= grid_x && event.column < grid_x + grid_width {
                    let col = event.column - grid_x;
                    let tpc = self.ticks_per_cell();
                    self.automation_cursor_tick = self.view_start_tick + col as u32 * tpc;

                    if graph_height > 0 {
                        let row = event.row.saturating_sub(auto_y);
                        self.automation_cursor_value =
                            1.0 - (row as f32 / graph_height.saturating_sub(1).max(1) as f32);
                        self.automation_cursor_value = self.automation_cursor_value.clamp(0.0, 1.0);
                    }
                }
                return Action::None;
            }
            // Click in note area
            self.automation_focus = false;
            let note_area = Rect {
                height: note_height,
                ..rect
            };
            return self.handle_mouse_note_editor(event, note_area, self.is_kit_track(state));
        }

        self.handle_mouse_note_editor(event, area, self.is_kit_track(state))
    }

    fn handle_mouse_note_editor(&mut self, event: &MouseEvent, area: Rect, is_kit: bool) -> Action {
        let rect = center_rect(area, 97, 29.min(area.height));
        let key_col_width: u16 = 5;
        let header_height: u16 = 2;
        let footer_height: u16 = 2;
        let grid_x = rect.x + key_col_width;
        let grid_y = rect.y + header_height;
        let grid_width = rect.width.saturating_sub(key_col_width + 1);
        let grid_height = rect
            .height
            .saturating_sub(header_height + footer_height + 1);

        let col = event.column;
        let row = event.row;

        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.selection_anchor = None;
                if col >= grid_x
                    && col < grid_x + grid_width
                    && row >= grid_y
                    && row < grid_y + grid_height
                {
                    let grid_col = col - grid_x;
                    let grid_row = row - grid_y;
                    let pitch = self
                        .view_bottom_pitch
                        .saturating_add((grid_height - 1 - grid_row) as u8);
                    let tick = self.view_start_tick + grid_col as u32 * self.ticks_per_cell();

                    if pitch <= 127 {
                        self.cursor_pitch = pitch;
                        self.cursor_tick = tick;
                        if !is_kit {
                            return Action::PianoRoll(PianoRollAction::ToggleNote {
                                pitch,
                                tick,
                                duration: self.default_duration,
                                velocity: self.default_velocity,
                                track: self.current_track,
                            });
                        }
                    }
                }
                if col >= rect.x && col < grid_x && row >= grid_y && row < grid_y + grid_height {
                    let grid_row = row - grid_y;
                    let pitch = self
                        .view_bottom_pitch
                        .saturating_add((grid_height - 1 - grid_row) as u8);
                    if pitch <= 127 {
                        self.cursor_pitch = pitch;
                    }
                }
                Action::None
            }
            MouseEventKind::Down(MouseButton::Right) => {
                if col >= grid_x
                    && col < grid_x + grid_width
                    && row >= grid_y
                    && row < grid_y + grid_height
                {
                    let grid_col = col - grid_x;
                    let grid_row = row - grid_y;
                    let pitch = self
                        .view_bottom_pitch
                        .saturating_add((grid_height - 1 - grid_row) as u8);
                    let tick = self.view_start_tick + grid_col as u32 * self.ticks_per_cell();
                    if pitch <= 127 {
                        self.cursor_pitch = pitch;
                        self.cursor_tick = tick;
                    }
                }
                Action::None
            }
            MouseEventKind::ScrollUp => {
                if event.modifiers.shift {
                    let step = self.ticks_per_cell() * 4;
                    self.view_start_tick = self.view_start_tick.saturating_sub(step);
                } else {
                    self.view_bottom_pitch = self.view_bottom_pitch.saturating_add(3).min(127);
                }
                Action::None
            }
            MouseEventKind::ScrollDown => {
                if event.modifiers.shift {
                    let step = self.ticks_per_cell() * 4;
                    self.view_start_tick += step;
                } else {
                    self.view_bottom_pitch = self.view_bottom_pitch.saturating_sub(3);
                }
                Action::None
            }
            _ => Action::None,
        }
    }
}
