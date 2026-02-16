use crate::action::{DispatchResult, MidiAction};
use crate::state::midi_recording::MidiCcMapping;
use crate::state::AppState;

pub(super) fn dispatch_midi(action: &MidiAction, state: &mut AppState) -> DispatchResult {
    match action {
        MidiAction::ConnectPort(_port_index) => {
            // Port connection is intercepted in main.rs (needs MidiInputManager)
            DispatchResult::none()
        }
        MidiAction::DisconnectPort => {
            // Port disconnection is intercepted in main.rs (needs MidiInputManager)
            state.midi.connected_port = None;
            DispatchResult::none()
        }
        MidiAction::AddCcMapping {
            cc,
            channel,
            target,
        } => {
            let mut mapping = MidiCcMapping::new(*cc, target.clone());
            mapping.channel = *channel;
            state.session.midi_recording.add_cc_mapping(mapping);
            DispatchResult::none()
        }
        MidiAction::RemoveCcMapping { cc, channel } => {
            state
                .session
                .midi_recording
                .remove_cc_mapping(*cc, *channel);
            DispatchResult::none()
        }
        MidiAction::SetChannelFilter(channel) => {
            state.session.midi_recording.channel_filter = *channel;
            DispatchResult::none()
        }
        MidiAction::SetLiveInputInstrument(instrument_id) => {
            state.session.midi_recording.live_input_instrument = *instrument_id;
            DispatchResult::none()
        }
        MidiAction::ToggleNotePassthrough => {
            state.session.midi_recording.note_passthrough =
                !state.session.midi_recording.note_passthrough;
            DispatchResult::none()
        }
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use crate::state::AutomationTarget;
    use imbolc_types::InstrumentId;

    fn setup() -> AppState {
        AppState::new()
    }

    // ── AddCcMapping ─────────────────────────────────────────────────

    #[test]
    fn add_cc_mapping_inserts_mapping() {
        let mut state = setup();
        let target = AutomationTarget::filter_cutoff(InstrumentId::new(0));
        let action = MidiAction::AddCcMapping {
            cc: 1,
            channel: None,
            target: target.clone(),
        };

        dispatch_midi(&action, &mut state);

        assert_eq!(state.session.midi_recording.cc_mappings.len(), 1);
        let m = &state.session.midi_recording.cc_mappings[0];
        assert_eq!(m.cc_number, 1);
        assert_eq!(m.channel, None);
        assert_eq!(m.target, target);
    }

    #[test]
    fn add_cc_mapping_with_channel() {
        let mut state = setup();
        let target = AutomationTarget::filter_cutoff(InstrumentId::new(1));
        let action = MidiAction::AddCcMapping {
            cc: 74,
            channel: Some(3),
            target: target.clone(),
        };

        dispatch_midi(&action, &mut state);

        assert_eq!(state.session.midi_recording.cc_mappings.len(), 1);
        let m = &state.session.midi_recording.cc_mappings[0];
        assert_eq!(m.cc_number, 74);
        assert_eq!(m.channel, Some(3));
    }

    #[test]
    fn add_cc_mapping_replaces_existing_same_cc_and_channel() {
        let mut state = setup();
        let target_a = AutomationTarget::filter_cutoff(InstrumentId::new(0));
        let target_b = AutomationTarget::filter_cutoff(InstrumentId::new(1));

        dispatch_midi(
            &MidiAction::AddCcMapping {
                cc: 1,
                channel: None,
                target: target_a,
            },
            &mut state,
        );
        dispatch_midi(
            &MidiAction::AddCcMapping {
                cc: 1,
                channel: None,
                target: target_b.clone(),
            },
            &mut state,
        );

        // Should replace, not accumulate
        assert_eq!(state.session.midi_recording.cc_mappings.len(), 1);
        assert_eq!(state.session.midi_recording.cc_mappings[0].target, target_b);
    }

    #[test]
    fn add_cc_mapping_different_cc_numbers_coexist() {
        let mut state = setup();
        let target = AutomationTarget::filter_cutoff(InstrumentId::new(0));

        dispatch_midi(
            &MidiAction::AddCcMapping {
                cc: 1,
                channel: None,
                target: target.clone(),
            },
            &mut state,
        );
        dispatch_midi(
            &MidiAction::AddCcMapping {
                cc: 2,
                channel: None,
                target: target.clone(),
            },
            &mut state,
        );

        assert_eq!(state.session.midi_recording.cc_mappings.len(), 2);
    }

    // ── RemoveCcMapping ──────────────────────────────────────────────

    #[test]
    fn remove_cc_mapping_removes_existing() {
        let mut state = setup();
        let target = AutomationTarget::filter_cutoff(InstrumentId::new(0));

        dispatch_midi(
            &MidiAction::AddCcMapping {
                cc: 7,
                channel: None,
                target,
            },
            &mut state,
        );
        assert_eq!(state.session.midi_recording.cc_mappings.len(), 1);

        dispatch_midi(
            &MidiAction::RemoveCcMapping {
                cc: 7,
                channel: None,
            },
            &mut state,
        );
        assert_eq!(state.session.midi_recording.cc_mappings.len(), 0);
    }

    #[test]
    fn remove_cc_mapping_no_op_when_missing() {
        let mut state = setup();

        dispatch_midi(
            &MidiAction::RemoveCcMapping {
                cc: 99,
                channel: None,
            },
            &mut state,
        );
        assert_eq!(state.session.midi_recording.cc_mappings.len(), 0);
    }

    #[test]
    fn remove_cc_mapping_respects_channel() {
        let mut state = setup();
        let target = AutomationTarget::filter_cutoff(InstrumentId::new(0));

        // Add mapping on channel 5
        dispatch_midi(
            &MidiAction::AddCcMapping {
                cc: 1,
                channel: Some(5),
                target,
            },
            &mut state,
        );

        // Remove on channel None (different key) — should not remove
        dispatch_midi(
            &MidiAction::RemoveCcMapping {
                cc: 1,
                channel: None,
            },
            &mut state,
        );
        assert_eq!(state.session.midi_recording.cc_mappings.len(), 1);

        // Remove on channel 5 — should remove
        dispatch_midi(
            &MidiAction::RemoveCcMapping {
                cc: 1,
                channel: Some(5),
            },
            &mut state,
        );
        assert_eq!(state.session.midi_recording.cc_mappings.len(), 0);
    }

    // ── SetChannelFilter ─────────────────────────────────────────────

    #[test]
    fn set_channel_filter_some() {
        let mut state = setup();
        assert_eq!(state.session.midi_recording.channel_filter, None);

        dispatch_midi(&MidiAction::SetChannelFilter(Some(9)), &mut state);
        assert_eq!(state.session.midi_recording.channel_filter, Some(9));
    }

    #[test]
    fn set_channel_filter_none_clears() {
        let mut state = setup();
        state.session.midi_recording.channel_filter = Some(4);

        dispatch_midi(&MidiAction::SetChannelFilter(None), &mut state);
        assert_eq!(state.session.midi_recording.channel_filter, None);
    }

    // ── SetLiveInputInstrument ───────────────────────────────────────

    #[test]
    fn set_live_input_instrument_some() {
        let mut state = setup();
        let id = InstrumentId::new(42);

        dispatch_midi(&MidiAction::SetLiveInputInstrument(Some(id)), &mut state);
        assert_eq!(state.session.midi_recording.live_input_instrument, Some(id));
    }

    #[test]
    fn set_live_input_instrument_none_clears() {
        let mut state = setup();
        state.session.midi_recording.live_input_instrument = Some(InstrumentId::new(7));

        dispatch_midi(&MidiAction::SetLiveInputInstrument(None), &mut state);
        assert_eq!(state.session.midi_recording.live_input_instrument, None);
    }

    // ── ToggleNotePassthrough ────────────────────────────────────────

    #[test]
    fn toggle_note_passthrough_flips_true_to_false() {
        let mut state = setup();
        // Default is true (from MidiRecordingState::new())
        assert!(state.session.midi_recording.note_passthrough);

        dispatch_midi(&MidiAction::ToggleNotePassthrough, &mut state);
        assert!(!state.session.midi_recording.note_passthrough);
    }

    #[test]
    fn toggle_note_passthrough_flips_false_to_true() {
        let mut state = setup();
        state.session.midi_recording.note_passthrough = false;

        dispatch_midi(&MidiAction::ToggleNotePassthrough, &mut state);
        assert!(state.session.midi_recording.note_passthrough);
    }

    #[test]
    fn toggle_note_passthrough_double_toggle_restores() {
        let mut state = setup();
        let original = state.session.midi_recording.note_passthrough;

        dispatch_midi(&MidiAction::ToggleNotePassthrough, &mut state);
        dispatch_midi(&MidiAction::ToggleNotePassthrough, &mut state);

        assert_eq!(state.session.midi_recording.note_passthrough, original);
    }

    // ── DisconnectPort ───────────────────────────────────────────────

    #[test]
    fn disconnect_port_clears_connected_port() {
        let mut state = setup();
        state.midi.connected_port = Some("My MIDI Device".to_string());

        dispatch_midi(&MidiAction::DisconnectPort, &mut state);
        assert_eq!(state.midi.connected_port, None);
    }

    #[test]
    fn disconnect_port_no_op_when_already_none() {
        let mut state = setup();
        assert_eq!(state.midi.connected_port, None);

        dispatch_midi(&MidiAction::DisconnectPort, &mut state);
        assert_eq!(state.midi.connected_port, None);
    }

    // ── DispatchResult ───────────────────────────────────────────────

    #[test]
    fn all_actions_return_dispatch_result_none() {
        let mut state = setup();
        let target = AutomationTarget::filter_cutoff(InstrumentId::new(0));

        let actions: Vec<MidiAction> = vec![
            MidiAction::ConnectPort(0),
            MidiAction::DisconnectPort,
            MidiAction::AddCcMapping {
                cc: 1,
                channel: None,
                target,
            },
            MidiAction::RemoveCcMapping {
                cc: 1,
                channel: None,
            },
            MidiAction::SetChannelFilter(Some(0)),
            MidiAction::SetLiveInputInstrument(None),
            MidiAction::ToggleNotePassthrough,
        ];

        for action in &actions {
            let result = dispatch_midi(action, &mut state);
            assert!(
                result.audio_effects.is_empty(),
                "Expected no audio effects for {:?}",
                action
            );
        }
    }
}
