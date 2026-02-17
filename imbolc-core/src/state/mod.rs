pub mod arpeggiator;
pub mod arrangement;
pub mod audio_feedback;
pub mod automation;
pub mod clipboard;
pub mod custom_synthdef;
pub mod drum_sequencer;
pub mod grid;
pub mod instrument;
pub mod instrument_state;
pub mod midi_connection;
pub mod midi_recording;
pub mod music;
pub mod param;
pub mod persistence;
pub mod piano_roll;
pub mod recent_projects;
pub mod sampler;
pub mod session;
pub mod undo;
pub mod vst_plugin;

pub use arrangement::{ArrangementState, Clip, ClipId, ClipPlacement, PlacementId, PlayMode};
pub use audio_feedback::AudioFeedbackState;
pub use automation::AutomationTarget;
pub use clipboard::{Clipboard, ClipboardContents, ClipboardNote};
pub use custom_synthdef::{CustomSynthDef, CustomSynthDefRegistry, ParamSpec};
pub use instrument::*;
pub use instrument::{track_row_count, track_row_info, track_section_for_row, TrackSection};
pub use instrument::{EffectTypeExt, SourceTypeExt};
pub use instrument_state::TrackState;
pub use midi_connection::MidiConnectionState;
pub use param::{adjust_freq_semitone, adjust_musical_step, is_freq_param, Param, ParamValue};
pub use sampler::{BufferId, SampleBuffer, SampleRegistry, SamplerConfig, Slice, SliceId};
pub use session::{
    MixerSelection, MixerState, MusicalSettings, SessionState, DEFAULT_BUS_COUNT, MAX_BUSES,
};
pub use undo::UndoHistory;
pub use vst_plugin::{VstParamSpec, VstPlugin, VstPluginId, VstPluginKind, VstPluginRegistry};

// Re-export types moved to imbolc-types
pub use imbolc_types::{
    BusId, ClientDisplayInfo, IoGeneration, IoState, KeyboardLayout, NetworkConnectionStatus,
    NetworkDisplayContext, OwnershipDisplayStatus, PendingExport, PendingRender, ProjectMeta,
    RecordingState, VisualizationState,
};

/// Top-level application state, owned by main.rs and passed to panes by reference.
pub struct AppState {
    pub session: SessionState,
    pub tracks: TrackState,
    pub clipboard: Clipboard,
    /// I/O state for render and export operations
    pub io: IoState,
    pub keyboard_layout: KeyboardLayout,
    /// Recording state (audio recording + automation recording)
    pub recording: RecordingState,
    /// Audio feedback state (visualization, playhead, bpm, server_status)
    pub audio: AudioFeedbackState,
    pub recorded_waveform_peaks: Option<Vec<f32>>,
    /// Undo/redo history (owned by state so dispatch can manage it)
    pub undo_history: UndoHistory,
    /// Project metadata (path, dirty flag, default settings)
    pub project: ProjectMeta,
    /// MIDI hardware connection state
    pub midi: MidiConnectionState,
    /// Network collaboration context (None when running standalone)
    pub network: Option<NetworkDisplayContext>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            session: SessionState::new(),
            tracks: TrackState::new(),
            clipboard: Clipboard::default(),
            io: IoState::default(),
            keyboard_layout: KeyboardLayout::default(),
            recording: RecordingState::default(),
            audio: AudioFeedbackState::default(),
            recorded_waveform_peaks: None,
            undo_history: UndoHistory::new(500),
            project: ProjectMeta::default(),
            midi: MidiConnectionState::default(),
            network: None,
        }
    }

    pub fn new_with_defaults(defaults: MusicalSettings) -> Self {
        Self {
            session: SessionState::new_with_defaults(defaults.clone(), session::DEFAULT_BUS_COUNT),
            tracks: TrackState::new(),
            clipboard: Clipboard::default(),
            io: IoState::default(),
            keyboard_layout: KeyboardLayout::default(),
            recording: RecordingState::default(),
            audio: AudioFeedbackState::default(),
            recorded_waveform_peaks: None,
            undo_history: UndoHistory::new(500),
            project: ProjectMeta::new_with_defaults(defaults),
            midi: MidiConnectionState::default(),
            network: None,
        }
    }

    /// Get the ownership status for a track (for UI display).
    pub fn ownership_status(&self, track_id: TrackId) -> OwnershipDisplayStatus {
        match &self.network {
            Some(ctx) => ctx
                .ownership
                .get(&track_id)
                .cloned()
                .unwrap_or(OwnershipDisplayStatus::Unowned),
            None => OwnershipDisplayStatus::Local,
        }
    }

    /// Add a track, with custom synthdef param setup and piano roll sequence auto-creation.
    pub fn add_track(&mut self, source: SourceType) -> TrackId {
        let id = self.tracks.add_track(source);
        imbolc_types::reduce::initialize_track_from_registries(
            id,
            source,
            &mut self.tracks,
            &self.session,
        );
        self.session.piano_roll.add_sequence(id);
        id
    }

    /// Remove a track and its piano roll sequence.
    pub fn remove_track(&mut self, id: TrackId) {
        self.tracks.remove_track(id);
        self.session.piano_roll.remove_sequence(id);
        self.session.automation.remove_lanes_for_instrument(id);
        self.session.arrangement.remove_instrument_data(id);
    }

    /// Compute effective mute for a track, considering solo state and master mute.
    pub fn effective_track_mute(&self, inst: &Track) -> bool {
        if self.tracks.any_track_solo() {
            !inst.channel_strip.solo
        } else {
            inst.channel_strip.mute || self.session.mixer.master_mute
        }
    }
}

impl imbolc_audio::AudioStateProvider for AppState {
    fn session(&self) -> &SessionState {
        &self.session
    }
    fn tracks(&self) -> &TrackState {
        &self.tracks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_track_clears_automation_lanes() {
        let mut state = AppState::new();
        let track_id = state.add_track(SourceType::Saw);

        assert_eq!(state.session.piano_roll.sequence_order.len(), 1);
        assert_eq!(state.session.piano_roll.sequence_order[0], track_id);

        state
            .session
            .automation
            .add_lane(AutomationTarget::level(track_id));
        state
            .session
            .automation
            .add_lane(AutomationTarget::pan(track_id));

        assert_eq!(
            state
                .session
                .automation
                .lanes_for_instrument(track_id)
                .len(),
            2
        );

        state.remove_track(track_id);

        assert!(state
            .session
            .automation
            .lanes_for_instrument(track_id)
            .is_empty());
        assert!(state.session.piano_roll.sequence_order.is_empty());
    }

    #[test]
    fn effective_track_mute_no_solo() {
        let mut state = AppState::new();
        state.add_track(SourceType::Saw);
        let inst = &state.tracks.tracks[0];
        // Not muted, no solo, no master mute
        assert!(!state.effective_track_mute(inst));

        // Mute the track
        state.tracks.tracks[0].channel_strip.mute = true;
        let inst = &state.tracks.tracks[0];
        assert!(state.effective_track_mute(inst));

        // Unmute track but mute master
        state.tracks.tracks[0].channel_strip.mute = false;
        state.session.mixer.master_mute = true;
        let inst = &state.tracks.tracks[0];
        assert!(state.effective_track_mute(inst));
    }

    #[test]
    fn effective_track_mute_with_solo() {
        let mut state = AppState::new();
        state.add_track(SourceType::Saw);
        state.add_track(SourceType::Sin);
        state.tracks.tracks[0].channel_strip.solo = true;

        let inst0 = &state.tracks.tracks[0];
        assert!(!state.effective_track_mute(inst0)); // soloed — not muted

        let inst1 = &state.tracks.tracks[1];
        assert!(state.effective_track_mute(inst1)); // not soloed — muted
    }

    #[test]
    fn add_track_creates_piano_roll_sequence() {
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);
        assert_eq!(state.session.piano_roll.sequence_order.len(), 1);
        assert!(state.session.piano_roll.sequences.contains_key(&id));
    }

    #[test]
    fn remove_track_cleans_up_all() {
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);
        state
            .session
            .automation
            .add_lane(AutomationTarget::level(id));
        assert_eq!(state.session.automation.lanes.len(), 1);

        state.remove_track(id);
        assert!(state.session.piano_roll.sequence_order.is_empty());
        assert!(state.session.automation.lanes.is_empty());
    }
}
