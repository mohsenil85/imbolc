use crate::action::{
    ArrangementAction, AudioEffect, AutomationAction, ClickAction, DomainAction, MixerAction,
    PaneId as NavPaneId, PianoRollAction, RoutedAction, SequencerAction, UiAction,
};
use crate::audio::AudioHandle;
use crate::dispatch::LocalDispatcher;
use crate::panes::{
    CommandPalettePane, ConfirmPane, FileBrowserPane, FrameEditPane, HelpPane, PaneSwitcherPane,
    PendingAction, PianoRollPane, SaveAsPane, SequencerPane, ServerPane, TrackEditPane, TrackPane,
    VstParamPane, WaveformPane,
};
use crate::state::{AppState, ClipboardContents, MixerSelection};
use crate::ui::action_id::{ActionId, GlobalActionId};
use crate::ui::{
    self, DispatchResult, Frame, LayerStack, NavIntent, PaneManager, RatatuiBackend, Rect,
    RenderBuf, SessionAction, StatusEvent, ToggleResult, ViewState,
};

/// Two-digit track selection state machine
pub(crate) enum TrackSelectMode {
    Normal,
    WaitingFirstDigit,
    WaitingSecondDigit(u8),
}

pub(crate) enum GlobalResult {
    Quit,
    RefreshScreen,
    Handled,
    NotHandled,
}

/// Select instrument by 1-based number (1=first, 10=tenth) and sync piano roll
pub(crate) fn select_instrument(
    number: usize,
    dispatcher: &mut LocalDispatcher,
    panes: &mut PaneManager,
    audio: &mut AudioHandle,
) {
    let idx = number.saturating_sub(1); // Convert 1-based to 0-based
    if idx < dispatcher.state().tracks.tracks.len() {
        dispatch_side_effect_free(
            dispatcher,
            &DomainAction::Track(ui::TrackAction::Select(idx)),
            audio,
        );
        sync_piano_roll_to_selection(dispatcher, panes, audio);
        sync_instrument_edit(dispatcher.state(), panes);
    }
}

/// Sync piano roll's current track to match the globally selected instrument,
/// and re-route the active pane if on a F2-family pane (piano_roll/sequencer/waveform).
pub(crate) fn sync_piano_roll_to_selection(
    dispatcher: &mut LocalDispatcher,
    panes: &mut PaneManager,
    audio: &mut AudioHandle,
) {
    let state = dispatcher.state();
    if let Some(selected_idx) = state.tracks.selected {
        // Extract data from instrument before any mutable borrows
        let inst_data = state.tracks.tracks.get(selected_idx).map(|inst| {
            (
                inst.id,
                inst.source.is_kit(),
                inst.source.is_audio_input(),
                inst.source.is_bus_in(),
            )
        });

        if let Some((inst_id, is_kit, is_audio_in, is_bus_in)) = inst_data {
            // Find which track index corresponds to this instrument
            if let Some(track_idx) = state
                .session
                .piano_roll
                .sequence_order
                .iter()
                .position(|&id| id == inst_id)
            {
                if let Some(pr_pane) = panes.get_pane_mut::<PianoRollPane>("piano_roll") {
                    pr_pane.set_current_track(track_idx);
                }
            }

            // Sync mixer selection via dispatch
            let active = panes.active().id().0;
            if active == "mixer" {
                if let MixerSelection::Track(_) = dispatcher.state().session.mixer.selection {
                    if let Some(track_id) = dispatcher.state().tracks.track_id_at(selected_idx) {
                        dispatch_side_effect_free(
                            dispatcher,
                            &DomainAction::Mixer(MixerAction::SelectAt(MixerSelection::Track(
                                track_id,
                            ))),
                            audio,
                        );
                    }
                }
            }

            // Re-route if currently on a F2-family pane
            if active == "piano_roll" || active == "sequencer" || active == "waveform" {
                let target = if is_audio_in || is_bus_in {
                    NavPaneId::Waveform
                } else if is_kit {
                    NavPaneId::Sequencer
                } else {
                    NavPaneId::PianoRoll
                };
                if active != target.as_str() {
                    panes.switch_to(target, dispatcher.state());
                }
            }
        }
    }
}

/// If the instrument edit pane is active, reload it with the currently selected instrument.
pub(crate) fn sync_instrument_edit(state: &AppState, panes: &mut PaneManager) {
    if panes.active().id().0 == "instrument_edit" {
        if let Some(inst) = state.tracks.selected_track() {
            if let Some(edit_pane) = panes.get_pane_mut::<TrackEditPane>("instrument_edit") {
                edit_pane.set_instrument(inst);
            }
        }
    }
}

fn dispatch_side_effect_free(
    dispatcher: &mut LocalDispatcher,
    action: &DomainAction,
    audio: &mut AudioHandle,
) {
    let result = dispatcher.dispatch_domain(action, audio);
    debug_assert!(
        !result.quit
            && result.nav.is_empty()
            && result.status.is_empty()
            && result.project_name.is_none()
            && result.audio_effects.is_empty()
            && !result.stop_playback
            && !result.reset_playhead
            && !result.needs_full_sync,
        "Expected side-effect-free dispatch for {:?}, got {:?}",
        action,
        result
    );
}

/// Sync layer stack pane layer and performance mode state after pane switch.
pub(crate) fn sync_pane_layer(panes: &mut PaneManager, layer_stack: &mut LayerStack) {
    let had_piano = layer_stack.has_layer("piano_mode");
    let had_pad = layer_stack.has_layer("pad_mode");
    layer_stack.set_pane_layer(panes.active());

    if had_piano || had_pad {
        if panes.active_mut().supports_performance_mode() {
            if had_piano {
                panes.active_mut().activate_piano();
            }
            if had_pad {
                panes.active_mut().activate_pad();
            }
        } else {
            layer_stack.pop("piano_mode");
            layer_stack.pop("pad_mode");
            panes.active_mut().deactivate_performance();
        }
    }
}

/// Process layer management from a routed action. Called by both standalone and network client.
pub(crate) fn process_layer_actions(
    action: &RoutedAction,
    layer_stack: &mut LayerStack,
    panes: &mut PaneManager,
) {
    match action {
        RoutedAction::Ui(UiAction::PushLayer(name)) => layer_stack.push(name),
        RoutedAction::Ui(UiAction::PopLayer(name)) => layer_stack.pop(name),
        RoutedAction::Ui(UiAction::ExitPerformanceMode) => {
            layer_stack.pop("piano_mode");
            layer_stack.pop("pad_mode");
            panes.active_mut().deactivate_performance();
        }
        _ => {}
    }
}

/// Auto-pop text_edit layer when pane is no longer editing.
pub(crate) fn process_text_edit_auto_pop(panes: &mut PaneManager, layer_stack: &mut LayerStack) {
    if layer_stack.has_layer("text_edit") {
        let still_editing = match panes.active().id().0 {
            "instrument_edit" => panes
                .get_pane_mut::<TrackEditPane>("instrument_edit")
                .is_some_and(|p| p.is_editing()),
            "frame_edit" => panes
                .get_pane_mut::<FrameEditPane>("frame_edit")
                .is_some_and(|p| p.is_editing()),
            "server" => panes
                .get_pane_mut::<ServerPane>("server")
                .is_some_and(|p| p.is_editing_scsynth_args()),
            "track" => panes
                .get_pane_mut::<TrackPane>("track")
                .is_some_and(|p| p.is_editing()),
            "waveform" => panes
                .get_pane_mut::<WaveformPane>("waveform")
                .is_some_and(|p| p.is_editing()),
            _ => false,
        };
        if !still_editing {
            layer_stack.pop("text_edit");
        }
    }
}

/// Process navigation from a routed action and sync the layer stack.
pub(crate) fn process_nav_and_sync(
    action: &RoutedAction,
    panes: &mut PaneManager,
    layer_stack: &mut LayerStack,
    state: &AppState,
) {
    panes.process_nav(action, state);
    if matches!(action, RoutedAction::Ui(UiAction::Nav(_))) {
        sync_pane_layer(panes, layer_stack);
    }
}

/// Auto-pop pane_switcher layer and switch to selected pane.
pub(crate) fn process_pane_switcher_auto_pop(
    panes: &mut PaneManager,
    layer_stack: &mut LayerStack,
    state: &AppState,
) {
    if layer_stack.has_layer("pane_switcher") && panes.active().id().0 != "pane_switcher" {
        layer_stack.pop("pane_switcher");
        if let Some(switcher) = panes.get_pane_mut::<PaneSwitcherPane>("pane_switcher") {
            if let Some(pane_id) = switcher.take_pane() {
                panes.switch_to(pane_id, state);
                sync_pane_layer(panes, layer_stack);
            }
        }
    }
}

/// Render a single frame (header + active pane). Used by both standalone and network client.
pub(crate) fn render_frame(
    backend: &mut RatatuiBackend,
    app_frame: &Frame,
    panes: &mut PaneManager,
    state: &AppState,
    last_area: &mut Rect,
) -> std::io::Result<()> {
    let mut frame = backend.begin_frame()?;
    let area = frame.area();
    *last_area = area;
    let cursor_pos;
    {
        let mut rbuf = RenderBuf::new(frame.buffer_mut());
        // Fill with theme background before rendering any content
        let bg: ui::Color = state.session.theme.background.into();
        rbuf.fill_bg(area, bg);
        app_frame.render_buf(area, &mut rbuf, state);
        if ui::Frame::is_size_ok(area) {
            panes.render(area, &mut rbuf, state);
        }
        cursor_pos = rbuf.cursor_position();
    }
    if let Some((x, y)) = cursor_pos {
        frame.set_cursor_position(x, y);
    }
    backend.end_frame(frame)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_global_action(
    action: ActionId,
    dispatcher: &mut LocalDispatcher,
    panes: &mut PaneManager,
    audio: &mut AudioHandle,
    app_frame: &mut Frame,
    select_mode: &mut TrackSelectMode,
    pending_audio_effects: &mut Vec<AudioEffect>,
    needs_full_sync: &mut bool,
    layer_stack: &mut LayerStack,
) -> GlobalResult {
    // Helper to capture current view state
    let capture_view = |panes: &mut PaneManager, state: &AppState| -> ViewState {
        let pane_id = panes.active().id().0.to_string();
        let inst_selection = state.tracks.selected;
        let edit_tab = panes
            .get_pane_mut::<TrackEditPane>("instrument_edit")
            .map(|ep| ep.tab_index())
            .unwrap_or(0);
        ViewState {
            pane_id,
            inst_selection,
            edit_tab,
        }
    };

    // Helper to restore view state
    let restore_view =
        |panes: &mut PaneManager, dispatcher: &mut LocalDispatcher, view: &ViewState| {
            dispatcher.state_mut().tracks.selected = view.inst_selection;
            if let Some(edit_pane) = panes.get_pane_mut::<TrackEditPane>("instrument_edit") {
                edit_pane.set_tab_index(view.edit_tab);
            }
            if let Some(pane_id) = NavPaneId::from_str(&view.pane_id) {
                panes.switch_to(pane_id, dispatcher.state());
            }
        };

    // Helper for pane switching with view history
    let switch_to_pane = |target: NavPaneId,
                          panes: &mut PaneManager,
                          dispatcher: &mut LocalDispatcher,
                          audio: &mut AudioHandle,
                          app_frame: &mut Frame,
                          layer_stack: &mut LayerStack| {
        // Stop tuner tone when leaving tuner pane
        if panes.active().id().0 == "tuner" {
            audio.stop_tuner_tone();
        }
        let current = capture_view(panes, dispatcher.state());
        if app_frame.view_history.is_empty() {
            app_frame.view_history.push(current);
        } else {
            app_frame.view_history[app_frame.history_cursor] = current;
        }
        // Truncate forward history
        app_frame
            .view_history
            .truncate(app_frame.history_cursor + 1);
        // Switch and record new view
        panes.switch_to(target, dispatcher.state());
        sync_pane_layer(panes, layer_stack);
        // Sync mixer highlight to global instrument selection on entry
        if target == NavPaneId::Mixer {
            if let Some(selected_idx) = dispatcher.state().tracks.selected {
                if let Some(track_id) = dispatcher.state().tracks.track_id_at(selected_idx) {
                    dispatch_side_effect_free(
                        dispatcher,
                        &DomainAction::Mixer(MixerAction::SelectAt(MixerSelection::Track(
                            track_id,
                        ))),
                        audio,
                    );
                }
            }
        }
        let new_view = capture_view(panes, dispatcher.state());
        app_frame.view_history.push(new_view);
        app_frame.history_cursor = app_frame.view_history.len() - 1;
    };

    match action {
        ActionId::Global(g) => match g {
            GlobalActionId::Quit => {
                return handle_quit_intent(dispatcher, panes, layer_stack);
            }
            GlobalActionId::Undo => {
                let mut r = dispatcher.dispatch_domain(&DomainAction::Undo, audio);
                if !r.audio_effects.is_empty() {
                    *needs_full_sync = true;
                }
                pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                sync_piano_roll_to_selection(dispatcher, panes, audio);
                sync_instrument_edit(dispatcher.state(), panes);
            }
            GlobalActionId::Redo => {
                let mut r = dispatcher.dispatch_domain(&DomainAction::Redo, audio);
                if !r.audio_effects.is_empty() {
                    *needs_full_sync = true;
                }
                pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                sync_piano_roll_to_selection(dispatcher, panes, audio);
                sync_instrument_edit(dispatcher.state(), panes);
            }
            GlobalActionId::Save => {
                if dispatcher.state().project.path.is_none() {
                    // Unnamed project — open SaveAs
                    let default_name = "untitled".to_string();
                    if let Some(sa) = panes.get_pane_mut::<SaveAsPane>("save_as") {
                        sa.reset(&default_name);
                    }
                    panes.push_to(NavPaneId::SaveAs, dispatcher.state());
                    sync_pane_layer(panes, layer_stack);
                } else {
                    let mut r = dispatcher
                        .dispatch_domain(&DomainAction::Session(SessionAction::Save), audio);
                    if r.needs_full_sync {
                        *needs_full_sync = true;
                    }
                    pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                    apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                }
            }
            GlobalActionId::Load => {
                if dispatcher.state().project.dirty {
                    if let Some(confirm) = panes.get_pane_mut::<ConfirmPane>("confirm") {
                        confirm.set_confirm(
                            "Discard unsaved changes and reload?",
                            PendingAction::LoadDefault,
                        );
                    }
                    panes.push_to(NavPaneId::Confirm, dispatcher.state());
                    sync_pane_layer(panes, layer_stack);
                } else {
                    let mut r = dispatcher
                        .dispatch_domain(&DomainAction::Session(SessionAction::Load), audio);
                    if r.needs_full_sync {
                        *needs_full_sync = true;
                    }
                    pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                    apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                }
            }
            GlobalActionId::SaveAs => {
                let default_name = dispatcher
                    .state()
                    .project
                    .path
                    .as_ref()
                    .and_then(|p| p.file_stem())
                    .and_then(|s| s.to_str())
                    .unwrap_or("untitled")
                    .to_string();
                if let Some(sa) = panes.get_pane_mut::<SaveAsPane>("save_as") {
                    sa.reset(&default_name);
                }
                panes.push_to(NavPaneId::SaveAs, dispatcher.state());
                sync_pane_layer(panes, layer_stack);
            }
            GlobalActionId::OpenProjectBrowser => {
                panes.push_to(NavPaneId::ProjectBrowser, dispatcher.state());
                sync_pane_layer(panes, layer_stack);
            }
            GlobalActionId::MasterMute => {
                let mut r = dispatcher.dispatch_domain(
                    &DomainAction::Session(SessionAction::ToggleMasterMute),
                    audio,
                );
                pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
            }
            GlobalActionId::ClickTrackToggle => {
                let mut r =
                    dispatcher.dispatch_domain(&DomainAction::Click(ClickAction::Toggle), audio);
                pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
            }
            GlobalActionId::ToggleRecordMaster => {
                let active_pane = panes.active().id().0;
                if active_pane == "piano_roll" {
                    // On the piano roll, C-r does MIDI recording (record notes to piano roll)
                    let mut r = dispatcher.dispatch_domain(
                        &DomainAction::PianoRoll(PianoRollAction::ToggleRecordPlayback),
                        audio,
                    );
                    pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                    apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                } else {
                    // Elsewhere (waveform, etc.), do audio recording as before
                    let mut r = dispatcher.dispatch_domain(
                        &DomainAction::Server(ui::ServerAction::ToggleRecordMaster),
                        audio,
                    );
                    if r.needs_full_sync {
                        *needs_full_sync = true;
                    }
                    pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                    apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                }
            }
            GlobalActionId::Copy => {
                copy_from_active_pane(dispatcher, panes, audio);
            }
            GlobalActionId::Cut => {
                let action = cut_from_active_pane(dispatcher, panes, audio);
                if let Some(action) = action {
                    let mut r = dispatcher.dispatch_domain(&action, audio);
                    if r.needs_full_sync {
                        *needs_full_sync = true;
                    }
                    pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                    apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                }
            }
            GlobalActionId::Paste => {
                let action = paste_to_active_pane(dispatcher.state_mut(), panes);
                if let Some(action) = action {
                    let mut r = dispatcher.dispatch_domain(&action, audio);
                    if r.needs_full_sync {
                        *needs_full_sync = true;
                    }
                    pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                    apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                }
            }
            GlobalActionId::SelectAll => {
                select_all_in_active_pane(dispatcher.state_mut(), panes);
            }
            GlobalActionId::SwitchPane(NavPaneId::InstrumentEdit) => {
                let target = if dispatcher.state().tracks.tracks.is_empty() {
                    NavPaneId::Add
                } else {
                    NavPaneId::InstrumentEdit
                };
                switch_to_pane(target, panes, dispatcher, audio, app_frame, layer_stack);
            }
            GlobalActionId::SwitchPianoRollOrSequencer => {
                let target = if let Some(inst) = dispatcher.state().tracks.selected_track() {
                    if inst.source.is_audio_input() || inst.source.is_bus_in() {
                        NavPaneId::Waveform
                    } else if inst.source.is_kit() {
                        // Kit: cycle between Sequencer and PianoRoll
                        if panes.active().id().0 == "sequencer" {
                            NavPaneId::PianoRoll
                        } else {
                            NavPaneId::Sequencer
                        }
                    } else {
                        NavPaneId::PianoRoll
                    }
                } else {
                    NavPaneId::PianoRoll
                };
                switch_to_pane(target, panes, dispatcher, audio, app_frame, layer_stack);

                // Auto-enter clip edit when switching to piano roll or sequencer
                if (target == NavPaneId::PianoRoll || target == NavPaneId::Sequencer)
                    && dispatcher
                        .state()
                        .session
                        .arrangement
                        .editing_clip
                        .is_none()
                {
                    if let Some(inst) = dispatcher.state().tracks.selected_track() {
                        let instrument_id = inst.id;
                        let mut r = dispatcher.dispatch_domain(
                            &DomainAction::Arrangement(ArrangementAction::AutoEnterClipEdit(
                                instrument_id,
                            )),
                            audio,
                        );
                        r.nav.clear();
                        if r.needs_full_sync {
                            *needs_full_sync = true;
                        }
                        pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                        apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                    }
                }
            }
            GlobalActionId::SwitchPane(NavPaneId::Track) => {
                switch_to_pane(
                    NavPaneId::Track,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
            }
            GlobalActionId::SwitchPane(NavPaneId::Mixer) => {
                switch_to_pane(
                    NavPaneId::Mixer,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
            }
            GlobalActionId::SwitchPane(NavPaneId::Server) => {
                switch_to_pane(
                    NavPaneId::Server,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
            }
            GlobalActionId::SwitchPane(NavPaneId::Automation) => {
                // F7: Navigate to piano roll with automation split visible
                switch_to_pane(
                    NavPaneId::PianoRoll,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
                if let Some(pr) = panes.get_pane_mut::<PianoRollPane>("piano_roll") {
                    pr.show_automation();
                }
            }
            GlobalActionId::SwitchPane(NavPaneId::Eq) => {
                switch_to_pane(
                    NavPaneId::Eq,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
            }
            GlobalActionId::SwitchPane(NavPaneId::MidiSettings) => {
                switch_to_pane(
                    NavPaneId::MidiSettings,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
            }
            GlobalActionId::SwitchPane(NavPaneId::Groove) => {
                // Clear pad context so F9 always opens track groove, not pad groove
                if let Some(seq) = dispatcher.state_mut().tracks.selected_drum_sequencer_mut() {
                    seq.groove_editing_pad = None;
                }
                switch_to_pane(
                    NavPaneId::Groove,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
            }
            GlobalActionId::SwitchPane(NavPaneId::Arpeggiator) => {
                switch_to_pane(
                    NavPaneId::Arpeggiator,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
            }
            GlobalActionId::SwitchPane(NavPaneId::Generative) => {
                switch_to_pane(
                    NavPaneId::Generative,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
            }
            GlobalActionId::SwitchPane(NavPaneId::Tuner) => {
                switch_to_pane(
                    NavPaneId::Tuner,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
            }
            GlobalActionId::SwitchPane(NavPaneId::FrameEdit) => {
                if panes.active().id().0 == "frame_edit" {
                    panes.pop(dispatcher.state());
                } else {
                    panes.push_to(NavPaneId::FrameEdit, dispatcher.state());
                }
            }
            GlobalActionId::SwitchPane(target) => {
                switch_to_pane(target, panes, dispatcher, audio, app_frame, layer_stack);
            }
            GlobalActionId::NavBack => {
                let history = &mut app_frame.view_history;
                if !history.is_empty() && app_frame.history_cursor > 0 {
                    let current = capture_view(panes, dispatcher.state());
                    history[app_frame.history_cursor] = current;
                    app_frame.history_cursor -= 1;
                    let view = history[app_frame.history_cursor].clone();
                    restore_view(panes, dispatcher, &view);
                    sync_pane_layer(panes, layer_stack);
                }
            }
            GlobalActionId::NavForward => {
                let history = &mut app_frame.view_history;
                if !history.is_empty() && app_frame.history_cursor < history.len() - 1 {
                    let current = capture_view(panes, dispatcher.state());
                    history[app_frame.history_cursor] = current;
                    app_frame.history_cursor += 1;
                    let view = history[app_frame.history_cursor].clone();
                    restore_view(panes, dispatcher, &view);
                    sync_pane_layer(panes, layer_stack);
                }
            }
            GlobalActionId::Help => {
                if panes.active().id().0 != "help" {
                    let current_id = panes.active().id().0;
                    let current_keymap = panes.active().keymap().clone();
                    let title = match current_id {
                        "instrument" => "Tracks",
                        "mixer" => "Mixer",
                        "server" => "Server",
                        "piano_roll" => "Piano Roll",
                        "sequencer" => "Step Sequencer",
                        "add" => "Add Track",
                        "instrument_edit" => "Edit Track",
                        "track" => "Track",
                        "waveform" => "Waveform",
                        "automation" => "Automation",
                        "eq" => "12-Band EQ",
                        _ => current_id,
                    };
                    if let Some(help) = panes.get_pane_mut::<HelpPane>("help") {
                        help.set_context(current_id, title, &current_keymap);
                    }
                    panes.push_to(NavPaneId::Help, dispatcher.state());
                }
            }
            GlobalActionId::SelectInstrument(n) => {
                select_instrument(n as usize, dispatcher, panes, audio);
                re_enter_clip_edit_if_needed(
                    dispatcher,
                    panes,
                    audio,
                    pending_audio_effects,
                    needs_full_sync,
                    app_frame,
                );
            }
            GlobalActionId::SelectPrevInstrument => {
                dispatch_side_effect_free(
                    dispatcher,
                    &DomainAction::Track(ui::TrackAction::SelectPrev),
                    audio,
                );
                sync_piano_roll_to_selection(dispatcher, panes, audio);
                sync_instrument_edit(dispatcher.state(), panes);
                re_enter_clip_edit_if_needed(
                    dispatcher,
                    panes,
                    audio,
                    pending_audio_effects,
                    needs_full_sync,
                    app_frame,
                );
            }
            GlobalActionId::SelectNextInstrument => {
                dispatch_side_effect_free(
                    dispatcher,
                    &DomainAction::Track(ui::TrackAction::SelectNext),
                    audio,
                );
                sync_piano_roll_to_selection(dispatcher, panes, audio);
                sync_instrument_edit(dispatcher.state(), panes);
                re_enter_clip_edit_if_needed(
                    dispatcher,
                    panes,
                    audio,
                    pending_audio_effects,
                    needs_full_sync,
                    app_frame,
                );
            }
            GlobalActionId::SelectTwoDigit => {
                *select_mode = TrackSelectMode::WaitingFirstDigit;
            }
            GlobalActionId::TogglePianoMode => {
                let result = panes
                    .active_mut()
                    .toggle_performance_mode(dispatcher.state());
                match result {
                    ToggleResult::ActivatedPiano => {
                        layer_stack.push("piano_mode");
                    }
                    ToggleResult::ActivatedPad => {
                        layer_stack.push("pad_mode");
                    }
                    ToggleResult::Deactivated => {
                        layer_stack.pop("piano_mode");
                        layer_stack.pop("pad_mode");
                    }
                    ToggleResult::CycledLayout | ToggleResult::NotSupported => {}
                }
            }
            GlobalActionId::AddInstrument => {
                switch_to_pane(
                    NavPaneId::Add,
                    panes,
                    dispatcher,
                    audio,
                    app_frame,
                    layer_stack,
                );
            }
            GlobalActionId::DeleteInstrument => {
                if let Some(instrument) = dispatcher.state().tracks.selected_track() {
                    let id = instrument.id;
                    let mut r = dispatcher
                        .dispatch_domain(&DomainAction::Track(ui::TrackAction::Delete(id)), audio);
                    pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                    apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                    // Re-sync edit pane after deletion
                    sync_instrument_edit(dispatcher.state(), panes);
                }
            }
            GlobalActionId::CommandLine => {
                panes.push_to(NavPaneId::CommandLine, dispatcher.state());
                sync_pane_layer(panes, layer_stack);
            }
            GlobalActionId::CommandPalette => {
                let commands = layer_stack.collect_commands();
                if let Some(palette) = panes.get_pane_mut::<CommandPalettePane>("command_palette") {
                    palette.open(commands);
                }
                panes.push_to(NavPaneId::CommandPalette, dispatcher.state());
                layer_stack.push("command_palette");
            }
            GlobalActionId::PaneSwitcher => {
                if let Some(switcher) = panes.get_pane_mut::<PaneSwitcherPane>("pane_switcher") {
                    switcher.open();
                }
                panes.push_to(NavPaneId::PaneSwitcher, dispatcher.state());
                layer_stack.push("pane_switcher");
            }
            GlobalActionId::TogglePlayback => {
                if panes.active().id().0 == "waveform" {
                    // In waveform view, Space acts on recording first.
                    if audio.is_recording() {
                        let mut r = dispatcher.dispatch_domain(
                            &DomainAction::Server(ui::ServerAction::ToggleRecordMaster),
                            audio,
                        );
                        if r.needs_full_sync {
                            *needs_full_sync = true;
                        }
                        pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                        apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
                        return GlobalResult::Handled;
                    }

                    // Wait until disk buffer flush completes before allowing preview.
                    if dispatcher
                        .state()
                        .recording
                        .pending_recording_path
                        .is_some()
                    {
                        app_frame.status_bar.push(
                            "Finalizing recording...",
                            crate::ui::status_bar::StatusLevel::Info,
                        );
                        return GlobalResult::Handled;
                    }

                    // After recording, Space toggles preview playback of the last recording.
                    if let Some(path) = dispatcher.state().recording.last_recording_path.clone() {
                        // Auto-clear stale preview flags once expected playback duration elapsed.
                        {
                            let rec = &mut dispatcher.state_mut().recording;
                            if let (Some(started), Some(duration)) =
                                (rec.preview_started_at, rec.preview_duration_secs)
                            {
                                if started.elapsed().as_secs_f32() >= duration.max(0.0) {
                                    rec.preview_node_id = None;
                                    rec.preview_started_at = None;
                                    rec.preview_duration_secs = None;
                                    rec.preview_start_norm = None;
                                    rec.preview_end_norm = None;
                                }
                            }
                        }

                        // If preview is currently active, Space stops it.
                        if let Some(node_id) = dispatcher.state().recording.preview_node_id {
                            let stop_result = audio.free_node(node_id);
                            let rec = &mut dispatcher.state_mut().recording;
                            rec.preview_node_id = None;
                            rec.preview_started_at = None;
                            rec.preview_duration_secs = None;
                            rec.preview_start_norm = None;
                            rec.preview_end_norm = None;

                            match stop_result {
                                Ok(()) => app_frame.status_bar.push(
                                    "Stopped recording playback",
                                    crate::ui::status_bar::StatusLevel::Info,
                                ),
                                Err(err) => app_frame.status_bar.push(
                                    format!("Stopped playback (node already finished: {})", err),
                                    crate::ui::status_bar::StatusLevel::Info,
                                ),
                            }
                            return GlobalResult::Handled;
                        }

                        if !audio.is_running() {
                            app_frame.status_bar.push(
                                "Audio engine not running",
                                crate::ui::status_bar::StatusLevel::Info,
                            );
                            return GlobalResult::Handled;
                        }
                        if !path.exists() {
                            app_frame.status_bar.push(
                                "Recording file not found",
                                crate::ui::status_bar::StatusLevel::Info,
                            );
                            return GlobalResult::Handled;
                        }

                        let (play_start_norm, play_end_norm) = if let Some(waveform) =
                            panes.get_pane_mut::<WaveformPane>("waveform")
                        {
                            waveform.playback_range_norm(dispatcher.state())
                        } else {
                            (0.0, 1.0)
                        };

                        let total_duration_secs = dispatcher
                            .state()
                            .recording
                            .last_recording_duration_secs
                            .unwrap_or(0.0)
                            .max(0.0);
                        let preview_duration_secs =
                            (total_duration_secs * (play_end_norm - play_start_norm)).max(0.0);

                        let buffer_id =
                            if let Some(id) = dispatcher.state().recording.preview_buffer_id {
                                id
                            } else {
                                let id = {
                                    let state = dispatcher.state_mut();
                                    let id = state.tracks.next_sampler_buffer_id;
                                    state.tracks.next_sampler_buffer_id = id.next();
                                    state.recording.preview_buffer_id = Some(id);
                                    id
                                };
                                let path_str = path.to_string_lossy().to_string();
                                match audio.load_sample(id, &path_str) {
                                    Ok(_) => id,
                                    Err(err) => {
                                        let rec = &mut dispatcher.state_mut().recording;
                                        rec.preview_buffer_id = None;
                                        rec.preview_node_id = None;
                                        rec.preview_started_at = None;
                                        rec.preview_duration_secs = None;
                                        rec.preview_start_norm = None;
                                        rec.preview_end_norm = None;
                                        app_frame.status_bar.push(
                                            format!("Failed to load recording: {}", err),
                                            crate::ui::status_bar::StatusLevel::Info,
                                        );
                                        return GlobalResult::Handled;
                                    }
                                }
                            };

                        {
                            let rec = &mut dispatcher.state_mut().recording;
                            rec.preview_node_id = None;
                            rec.preview_started_at = None;
                            rec.preview_duration_secs = None;
                            rec.preview_start_norm = None;
                            rec.preview_end_norm = None;
                        }

                        match audio.play_sample_preview_with_node(
                            buffer_id,
                            1.0,
                            play_start_norm,
                            play_end_norm,
                            1.0,
                            0.0,
                        ) {
                            Ok(node_id) => {
                                let rec = &mut dispatcher.state_mut().recording;
                                rec.preview_node_id = Some(node_id);
                                rec.preview_started_at = Some(std::time::Instant::now());
                                rec.preview_duration_secs = Some(preview_duration_secs.max(0.01));
                                rec.preview_start_norm = Some(play_start_norm);
                                rec.preview_end_norm = Some(play_end_norm);
                                app_frame.status_bar.push(
                                    "Playing recording (Space to stop)",
                                    crate::ui::status_bar::StatusLevel::Info,
                                );
                            }
                            Err(err) => {
                                let rec = &mut dispatcher.state_mut().recording;
                                rec.preview_node_id = None;
                                rec.preview_started_at = None;
                                rec.preview_duration_secs = None;
                                rec.preview_start_norm = None;
                                rec.preview_end_norm = None;
                                app_frame.status_bar.push(
                                    format!("Failed to play recording: {}", err),
                                    crate::ui::status_bar::StatusLevel::Info,
                                );
                            }
                        }
                        return GlobalResult::Handled;
                    }
                }

                // Skip during export/render
                if dispatcher.state().io.pending_export.is_some()
                    || dispatcher.state().io.pending_render.is_some()
                {
                    return GlobalResult::Handled;
                }
                let state = dispatcher.state_mut();
                let pr = &mut state.session.piano_roll;
                pr.playing = !pr.playing;
                let playing = pr.playing;
                audio.set_playing(playing);
                if !playing {
                    dispatcher.state_mut().audio.playhead = 0;
                    audio.reset_playhead();
                    if audio.is_running() {
                        audio.release_all_voices();
                    }
                    audio.clear_active_notes();
                }
                dispatcher.state_mut().session.piano_roll.recording = false;

                // Unify: toggle all drum sequencers
                for inst in &mut dispatcher.state_mut().tracks.tracks {
                    if let Some(seq) = inst.drum_sequencer_mut() {
                        seq.playing = playing;
                        if !playing {
                            seq.current_step = 0;
                            seq.step_accumulator = 0.0;
                        }
                    }
                }
                pending_audio_effects.push(AudioEffect::RebuildInstruments);
                *needs_full_sync = true;
            }
            GlobalActionId::Escape => {
                // Global escape — falls through to pane when no mode layer handles it
                return GlobalResult::NotHandled;
            }
            GlobalActionId::RefreshScreen => {
                return GlobalResult::RefreshScreen;
            }
            GlobalActionId::NextTheme => {
                let mut r = dispatcher
                    .dispatch_domain(&DomainAction::Session(SessionAction::NextTheme), audio);
                pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
                apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
            }
            GlobalActionId::OpenCheckpointList => {
                panes.push_to(NavPaneId::CheckpointList, dispatcher.state());
                sync_pane_layer(panes, layer_stack);
            }
            GlobalActionId::RequestPrivilege => {
                // No-op in standalone mode (handled in network client loop)
            }
        },
        _ => return GlobalResult::NotHandled,
    }
    GlobalResult::Handled
}

/// If on the piano roll or sequencer and currently in clip edit, exit the old clip and
/// auto-enter clip edit for the newly selected instrument.
#[allow(clippy::too_many_arguments)]
pub(crate) fn re_enter_clip_edit_if_needed(
    dispatcher: &mut LocalDispatcher,
    panes: &mut PaneManager,
    audio: &mut AudioHandle,
    pending_audio_effects: &mut Vec<AudioEffect>,
    needs_full_sync: &mut bool,
    app_frame: &mut Frame,
) {
    let active = panes.active().id().0;
    if (active == "piano_roll" || active == "sequencer")
        && dispatcher
            .state()
            .session
            .arrangement
            .editing_clip
            .is_some()
    {
        // Exit old clip edit
        let mut exit_r = dispatcher.dispatch_domain(
            &DomainAction::Arrangement(ArrangementAction::ExitClipEdit),
            audio,
        );
        exit_r.nav.clear();
        if exit_r.needs_full_sync {
            *needs_full_sync = true;
        }
        pending_audio_effects.extend(std::mem::take(&mut exit_r.audio_effects));
        apply_dispatch_result(exit_r, dispatcher, panes, app_frame, audio);

        // Enter new clip edit for the new instrument
        if let Some(inst) = dispatcher.state().tracks.selected_track() {
            let instrument_id = inst.id;
            let mut enter_r = dispatcher.dispatch_domain(
                &DomainAction::Arrangement(ArrangementAction::AutoEnterClipEdit(instrument_id)),
                audio,
            );
            enter_r.nav.clear();
            if enter_r.needs_full_sync {
                *needs_full_sync = true;
            }
            pending_audio_effects.extend(std::mem::take(&mut enter_r.audio_effects));
            apply_dispatch_result(enter_r, dispatcher, panes, app_frame, audio);
        }
    }
}

/// Handle a quit intent: check dirty state and show prompt if needed.
/// Returns `GlobalResult::Quit` for immediate exit, `GlobalResult::Handled` if prompt was shown.
pub(crate) fn handle_quit_intent(
    dispatcher: &mut LocalDispatcher,
    panes: &mut PaneManager,
    layer_stack: &mut LayerStack,
) -> GlobalResult {
    if dispatcher.state().project.dirty {
        panes.push_to(NavPaneId::QuitPrompt, dispatcher.state());
        sync_pane_layer(panes, layer_stack);
        GlobalResult::Handled
    } else {
        GlobalResult::Quit
    }
}

/// Handle SaveAndQuit: save (or prompt for name), then quit.
/// Sets `quit_after_save` so the runtime exits after save completes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_save_and_quit(
    dispatcher: &mut LocalDispatcher,
    panes: &mut PaneManager,
    audio: &mut AudioHandle,
    app_frame: &mut Frame,
    pending_audio_effects: &mut Vec<AudioEffect>,
    needs_full_sync: &mut bool,
    layer_stack: &mut LayerStack,
    quit_after_save: &mut bool,
) {
    if dispatcher.state().project.path.is_some() {
        let mut r = dispatcher.dispatch_domain(&DomainAction::Session(SessionAction::Save), audio);
        if r.needs_full_sync {
            *needs_full_sync = true;
        }
        pending_audio_effects.extend(std::mem::take(&mut r.audio_effects));
        apply_dispatch_result(r, dispatcher, panes, app_frame, audio);
        *quit_after_save = true;
    } else {
        let default_name = "untitled".to_string();
        if let Some(sa) = panes.get_pane_mut::<SaveAsPane>("save_as") {
            sa.reset(&default_name);
        }
        panes.pop(dispatcher.state());
        panes.push_to(NavPaneId::SaveAs, dispatcher.state());
        sync_pane_layer(panes, layer_stack);
        *quit_after_save = true;
    }
}

/// Apply status events from dispatch or setup to the server pane and status bar
pub(crate) fn apply_status_events(
    events: &[StatusEvent],
    panes: &mut PaneManager,
    app_frame: &mut Frame,
) {
    use crate::ui::status_bar::StatusLevel;
    for event in events {
        if let Some(server) = panes.get_pane_mut::<ServerPane>("server") {
            server.set_status(event.status, &event.message);
            if let Some(running) = event.server_running {
                server.set_server_running(running);
            }
        }
        app_frame.status_bar.push(&event.message, StatusLevel::Info);
    }
}

/// Apply a DispatchResult to the UI layer: process nav intents, status events, project name,
/// and audio control signals (stop_playback, reset_playhead).
pub(crate) fn apply_dispatch_result(
    result: DispatchResult,
    dispatcher: &mut LocalDispatcher,
    panes: &mut PaneManager,
    app_frame: &mut Frame,
    audio: &mut AudioHandle,
) {
    // Process nav intents
    for intent in &result.nav {
        match intent {
            NavIntent::OpenFileBrowser(file_action) => {
                if let Some(fb) = panes.get_pane_mut::<FileBrowserPane>("file_browser") {
                    fb.open_for(file_action.clone(), None);
                }
                panes.push_to(NavPaneId::FileBrowser, dispatcher.state());
            }
            NavIntent::OpenVstParams(instrument_id, target) => {
                if let Some(vp) = panes.get_pane_mut::<VstParamPane>("vst_params") {
                    vp.set_target(*instrument_id, *target);
                }
                panes.push_to(NavPaneId::VstParams, dispatcher.state());
            }
            NavIntent::OpenTagPicker(_) => {
                // pending_target is already set by the reducer
                panes.push_to(NavPaneId::TagPicker, dispatcher.state());
            }
            _ => {}
        }
    }
    panes.process_nav_intents(&result.nav, dispatcher.state());

    // Process status events
    apply_status_events(&result.status, panes, app_frame);

    // Process project name
    if let Some(ref name) = result.project_name {
        app_frame.set_project_name(name.to_string());
    }

    // Process audio control signals (avoids circular dispatch → audio → dispatch pattern)
    if result.stop_playback {
        audio.set_playing(false);
    }
    if result.reset_playhead {
        audio.reset_playhead();
    }
}

fn copy_from_active_pane(
    dispatcher: &mut LocalDispatcher,
    panes: &mut PaneManager,
    audio: &mut AudioHandle,
) {
    let pane_id = panes.active().id().0;
    match pane_id {
        "piano_roll" => {
            if let Some(pane) = panes.get_pane_mut::<PianoRollPane>("piano_roll") {
                if pane.automation_visible && pane.automation_focus {
                    // Copy automation points
                    if let Some((lane_id, start_tick, end_tick)) =
                        pane.automation_selection_region(dispatcher.state())
                    {
                        dispatch_side_effect_free(
                            dispatcher,
                            &DomainAction::Automation(AutomationAction::CopyPoints(
                                lane_id, start_tick, end_tick,
                            )),
                            audio,
                        );
                    }
                } else {
                    // Copy notes
                    if pane.is_kit_track(dispatcher.state()) {
                        return;
                    }
                    let (track, start_tick, end_tick, start_pitch, end_pitch) =
                        pane.selection_region();
                    dispatch_side_effect_free(
                        dispatcher,
                        &DomainAction::PianoRoll(PianoRollAction::CopyNotes {
                            track,
                            start_tick,
                            end_tick,
                            start_pitch,
                            end_pitch,
                        }),
                        audio,
                    );
                }
            }
        }
        "sequencer" => {
            if let Some(pane) = panes.get_pane_mut::<SequencerPane>("sequencer") {
                let (start_pad, end_pad, start_step, end_step) = pane.selection_region();
                dispatch_side_effect_free(
                    dispatcher,
                    &DomainAction::Sequencer(SequencerAction::CopySteps {
                        start_pad,
                        end_pad,
                        start_step,
                        end_step,
                    }),
                    audio,
                );
            }
        }
        _ => {}
    }
}

fn cut_from_active_pane(
    dispatcher: &mut LocalDispatcher,
    panes: &mut PaneManager,
    audio: &mut AudioHandle,
) -> Option<DomainAction> {
    // Copy first
    copy_from_active_pane(dispatcher, panes, audio);

    // Then return delete action
    let pane_id = panes.active().id().0;
    match pane_id {
        "piano_roll" => {
            if let Some(pane) = panes.get_pane_mut::<PianoRollPane>("piano_roll") {
                if pane.automation_visible && pane.automation_focus {
                    // Cut automation points
                    if let Some(anchor_tick) = pane.automation_selection_anchor_tick {
                        let (tick_start, tick_end) = if anchor_tick <= pane.automation_cursor_tick {
                            (anchor_tick, pane.automation_cursor_tick)
                        } else {
                            (pane.automation_cursor_tick, anchor_tick)
                        };
                        if let Some(lane_id) = pane.selected_lane_id(dispatcher.state()) {
                            pane.automation_selection_anchor_tick = None;
                            return Some(DomainAction::Automation(
                                AutomationAction::DeletePointsInRange(
                                    lane_id, tick_start, tick_end,
                                ),
                            ));
                        }
                    }
                } else {
                    // Cut notes
                    if pane.is_kit_track(dispatcher.state()) {
                        return None;
                    }
                    if let Some((anchor_tick, anchor_pitch)) = pane.selection_anchor {
                        let (tick_start, tick_end) = if anchor_tick <= pane.cursor_tick {
                            (anchor_tick, pane.cursor_tick + pane.ticks_per_cell())
                        } else {
                            (pane.cursor_tick, anchor_tick + pane.ticks_per_cell())
                        };
                        let (pitch_start, pitch_end) = if anchor_pitch <= pane.cursor_pitch {
                            (anchor_pitch, pane.cursor_pitch)
                        } else {
                            (pane.cursor_pitch, anchor_pitch)
                        };

                        pane.selection_anchor = None;

                        return Some(DomainAction::PianoRoll(
                            PianoRollAction::DeleteNotesInRegion {
                                track: pane.current_track,
                                start_tick: tick_start,
                                end_tick: tick_end,
                                start_pitch: pitch_start,
                                end_pitch: pitch_end,
                            },
                        ));
                    }
                }
            }
        }
        "sequencer" => {
            if let Some(pane) = panes.get_pane_mut::<SequencerPane>("sequencer") {
                if let Some((anchor_pad, anchor_step)) = pane.selection_anchor {
                    let (pad_start, pad_end) = if anchor_pad <= pane.cursor_pad {
                        (anchor_pad, pane.cursor_pad)
                    } else {
                        (pane.cursor_pad, anchor_pad)
                    };
                    let (step_start, step_end) = if anchor_step <= pane.cursor_step {
                        (anchor_step, pane.cursor_step)
                    } else {
                        (pane.cursor_step, anchor_step)
                    };
                    pane.selection_anchor = None;

                    return Some(DomainAction::Sequencer(
                        SequencerAction::DeleteStepsInRegion {
                            start_pad: pad_start,
                            end_pad: pad_end,
                            start_step: step_start,
                            end_step: step_end,
                        },
                    ));
                }
            }
        }
        _ => {}
    }
    None
}

fn paste_to_active_pane(state: &mut AppState, panes: &mut PaneManager) -> Option<DomainAction> {
    if let Some(contents) = &state.clipboard.contents {
        match contents {
            ClipboardContents::PianoRollNotes(notes) => {
                if panes.active().id().0 == "piano_roll" {
                    if let Some(pane) = panes.get_pane_mut::<PianoRollPane>("piano_roll") {
                        if pane.is_kit_track(state) {
                            return None;
                        }
                        // anchor is cursor position
                        let action = DomainAction::PianoRoll(PianoRollAction::PasteNotes {
                            track: pane.current_track,
                            anchor_tick: pane.cursor_tick,
                            anchor_pitch: pane.cursor_pitch,
                            notes: notes.clone(),
                        });
                        // Clear selection if any (optional, but good UX)
                        pane.selection_anchor = None;
                        return Some(action);
                    }
                }
            }
            ClipboardContents::DrumSteps { steps } => {
                if panes.active().id().0 == "sequencer" {
                    if let Some(pane) = panes.get_pane_mut::<SequencerPane>("sequencer") {
                        let action = DomainAction::Sequencer(SequencerAction::PasteSteps {
                            anchor_pad: pane.cursor_pad,
                            anchor_step: pane.cursor_step,
                            steps: steps.clone(),
                        });
                        pane.selection_anchor = None;
                        return Some(action);
                    }
                }
            }
            ClipboardContents::AutomationPoints { points } => {
                if panes.active().id().0 == "piano_roll" {
                    if let Some(pane) = panes.get_pane_mut::<PianoRollPane>("piano_roll") {
                        if pane.automation_visible && pane.automation_focus {
                            if let Some(lane_id) = pane.selected_lane_id(state) {
                                let action =
                                    DomainAction::Automation(AutomationAction::PastePoints(
                                        lane_id,
                                        pane.automation_cursor_tick,
                                        points.clone(),
                                    ));
                                pane.automation_selection_anchor_tick = None;
                                return Some(action);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn select_all_in_active_pane(state: &mut AppState, panes: &mut PaneManager) {
    let pane_id = panes.active().id().0;
    match pane_id {
        "piano_roll" => {
            if let Some(pane) = panes.get_pane_mut::<PianoRollPane>("piano_roll") {
                if pane.is_kit_track(state) {
                    return;
                }
                if let Some(seq) = state.session.piano_roll.sequence_at(pane.current_track) {
                    if let Some(min_tick) = seq.notes.iter().map(|n| n.tick).min() {
                        let max_tick = seq
                            .notes
                            .iter()
                            .map(|n| n.tick + n.duration)
                            .max()
                            .unwrap_or(min_tick);
                        let min_pitch = seq.notes.iter().map(|n| n.pitch).min().unwrap_or(0);
                        let max_pitch = seq.notes.iter().map(|n| n.pitch).max().unwrap_or(127);

                        pane.selection_anchor = Some((min_tick, min_pitch));
                        pane.cursor_tick = max_tick;
                        pane.cursor_pitch = max_pitch;
                        pane.scroll_to_cursor();
                    }
                }
            }
        }
        "sequencer" => {
            if let Some(pane) = panes.get_pane_mut::<SequencerPane>("sequencer") {
                if let Some(seq) = state.tracks.selected_drum_sequencer() {
                    let pattern = seq.pattern();
                    pane.selection_anchor = Some((0, 0));
                    pane.cursor_pad = crate::state::drum_sequencer::NUM_PADS - 1;
                    pane.cursor_step = pattern.length - 1;
                }
            }
        }
        _ => {}
    }
}
