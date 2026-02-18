//! Audio command and feedback types for the audio thread abstraction.
//!
//! Commands are sent from the main thread to the audio thread via priority and
//! normal channels. State synchronization (ForwardAction, FullStateSync,
//! PianoRollUpdate, AutomationUpdate) is handled by the event log instead.
//! See `event_log.rs` for the log-based state sync mechanism.

use std::path::PathBuf;
use std::sync::mpsc::Sender;

use imbolc_types::AutomationTarget;
use imbolc_types::VstTarget;
use imbolc_types::{BufferId, BusId, EffectId, GroupId, TrackId};

/// Commands sent from the main thread to the audio engine.
///
/// State synchronization is handled by the event log (EventLogWriter/Reader),
/// not by AudioCmd variants. These commands cover server lifecycle, playback
/// control, routing, voice management, samples, recording, automation, and VST.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AudioCmd {
    // ── Server lifecycle ──────────────────────────────────────────
    Connect {
        server_addr: String,
        reply: Sender<std::io::Result<()>>,
    },
    Disconnect,
    StartServer {
        input_device: Option<String>,
        output_device: Option<String>,
        buffer_size: u32,
        sample_rate: u32,
        scsynth_args: String,
        reply: Sender<Result<(), String>>,
    },
    StopServer,
    RestartServer {
        input_device: Option<String>,
        output_device: Option<String>,
        server_addr: String,
        buffer_size: u32,
        sample_rate: u32,
        scsynth_args: String,
    },
    CompileSynthDefs {
        scd_path: PathBuf,
        reply: Sender<Result<(), String>>,
    },
    LoadSynthDefs {
        dir: PathBuf,
    },
    LoadSynthDefFile {
        path: PathBuf,
    },

    // ── Playback control ──────────────────────────────────────────
    SetPlaying {
        playing: bool,
    },
    ResetPlayhead,
    SetBpm {
        bpm: f32,
    },

    // ── Routing & mixing ──────────────────────────────────────────
    RebuildRouting,
    RebuildInstrumentRouting {
        instrument_id: TrackId,
    },
    UpdateMixerParams,
    SetBusMixerParams {
        bus_id: BusId,
        level: f32,
        mute: bool,
        pan: f32,
    },
    SetGroupMixerParams {
        group_id: GroupId,
        level: f32,
        mute: bool,
        pan: f32,
    },
    SetSourceParam {
        instrument_id: TrackId,
        param: String,
        value: f32,
    },
    SetEqParam {
        instrument_id: TrackId,
        effect_id: EffectId,
        param: String,
        value: f32,
    },
    /// Targeted /n_set to filter node (no routing rebuild).
    SetFilterParam {
        instrument_id: TrackId,
        param: String,
        value: f32,
    },
    /// Targeted /n_set to effect node (no routing rebuild).
    SetEffectParam {
        instrument_id: TrackId,
        effect_id: EffectId,
        param: String,
        value: f32,
    },
    /// Targeted /n_set to LFO node (no routing rebuild).
    SetLfoParam {
        instrument_id: TrackId,
        param: String,
        value: f32,
    },
    /// Targeted /n_set to bus effect node (no routing rebuild).
    SetBusEffectParam {
        bus_id: BusId,
        effect_id: EffectId,
        param: String,
        value: f32,
    },
    /// Targeted /n_set to layer group effect node (no routing rebuild).
    SetGroupEffectParam {
        group_id: GroupId,
        effect_id: EffectId,
        param: String,
        value: f32,
    },
    /// Targeted /n_set to bus EQ node (no routing rebuild).
    SetBusEqParam {
        bus_id: BusId,
        effect_id: EffectId,
        param: String,
        value: f32,
    },
    /// Targeted /n_set to layer group EQ node (no routing rebuild).
    SetLayerGroupEqParam {
        group_id: GroupId,
        effect_id: EffectId,
        param: String,
        value: f32,
    },
    /// Targeted /n_set to master EQ node (no routing rebuild).
    SetMasterEqParam {
        effect_id: EffectId,
        param: String,
        value: f32,
    },
    /// Targeted /n_set to master effect node (no routing rebuild).
    SetMasterEffectParam {
        effect_id: EffectId,
        param: String,
        value: f32,
    },
    SetInstrumentMixerParams {
        instrument_id: TrackId,
        level: f32,
        pan: f32,
        mute: bool,
        solo: bool,
    },
    SetMasterParams {
        level: f32,
        mute: bool,
    },

    // ── Voice management ──────────────────────────────────────────
    SpawnVoice {
        instrument_id: TrackId,
        pitch: u8,
        velocity: f32,
        offset_secs: f64,
    },
    ReleaseVoice {
        instrument_id: TrackId,
        pitch: u8,
        offset_secs: f64,
    },
    RegisterActiveNote {
        instrument_id: TrackId,
        pitch: u8,
        duration_ticks: u32,
    },
    ClearActiveNotes,
    ReleaseAllVoices,
    PlayDrumHit {
        buffer_id: BufferId,
        amp: f32,
        instrument_id: TrackId,
        slice_start: f32,
        slice_end: f32,
        rate: f32,
        offset_secs: f64,
    },
    PlayDrumHitWithReply {
        buffer_id: BufferId,
        amp: f32,
        instrument_id: TrackId,
        slice_start: f32,
        slice_end: f32,
        rate: f32,
        offset_secs: f64,
        reply: Sender<Result<i32, String>>,
    },
    PlaySamplePreviewWithReply {
        buffer_id: BufferId,
        amp: f32,
        slice_start: f32,
        slice_end: f32,
        rate: f32,
        offset_secs: f64,
        reply: Sender<Result<i32, String>>,
    },
    FreeNode {
        node_id: i32,
    },

    // ── Samples ───────────────────────────────────────────────────
    LoadSample {
        buffer_id: BufferId,
        path: String,
        reply: Sender<Result<i32, String>>,
    },
    FreeSamples {
        buffer_ids: Vec<BufferId>,
    },

    // ── Recording ─────────────────────────────────────────────────
    StartRecording {
        bus: i32,
        path: PathBuf,
        reply: Sender<Result<(), String>>,
    },
    StopRecording {
        reply: Sender<Option<PathBuf>>,
    },
    StartInstrumentRender {
        instrument_id: TrackId,
        path: PathBuf,
        reply: Sender<Result<(), String>>,
    },
    StartMasterBounce {
        path: PathBuf,
        reply: Sender<Result<(), String>>,
    },
    StartStemExport {
        stems: Vec<(TrackId, PathBuf)>,
        reply: Sender<Result<(), String>>,
    },
    CancelExport,

    // ── Automation ────────────────────────────────────────────────
    ApplyAutomation {
        target: AutomationTarget,
        value: f32,
    },

    // ── VST parameter control ──────────────────────────────────
    QueryVstParams {
        instrument_id: TrackId,
        target: VstTarget,
    },
    SetVstParam {
        instrument_id: TrackId,
        target: VstTarget,
        param_index: u32,
        value: f32,
    },
    SaveVstState {
        instrument_id: TrackId,
        target: VstTarget,
        path: PathBuf,
    },
    LoadVstState {
        instrument_id: TrackId,
        target: VstTarget,
        path: PathBuf,
    },

    // ── Tuner tone ────────────────────────────────────────────────
    StartTunerTone {
        freq: f32,
    },
    StopTunerTone,

    // ── Click track ──────────────────────────────────────────────
    SetClickEnabled {
        enabled: bool,
    },
    SetClickVolume {
        volume: f32,
    },
    SetClickMuted {
        muted: bool,
    },

    // ── Lifecycle ─────────────────────────────────────────────────
    Shutdown,
}

impl AudioCmd {
    /// Returns true if this command is time-critical and should use the priority channel.
    ///
    /// Priority commands are those requiring minimal latency:
    /// - Voice spawn/release (MIDI keyboard responsiveness)
    /// - Individual param changes (knob tweaks, real-time control)
    /// - Playback control (play/stop/seek)
    /// - Automation application (during playback)
    ///
    /// Normal commands (handled after priority queue is drained):
    /// - Bulk mixer updates (SetInstrumentMixerParams, SetMasterParams, SetBusMixerParams)
    /// - Active note tracking (RegisterActiveNote, ClearActiveNotes, ReleaseAllVoices)
    /// - Routing rebuilds, recording, server lifecycle
    pub fn is_priority(&self) -> bool {
        matches!(
            self,
            // Voice management (most time-critical)
            AudioCmd::SpawnVoice { .. }
                | AudioCmd::ReleaseVoice { .. }
                | AudioCmd::PlayDrumHit { .. }
                | AudioCmd::PlayDrumHitWithReply { .. }
                | AudioCmd::PlaySamplePreviewWithReply { .. }
                | AudioCmd::FreeNode { .. }
                // Individual param changes (need low latency for knob tweaks)
                | AudioCmd::SetSourceParam { .. }
                | AudioCmd::SetEqParam { .. }
                | AudioCmd::SetFilterParam { .. }
                | AudioCmd::SetEffectParam { .. }
                | AudioCmd::SetLfoParam { .. }
                | AudioCmd::SetBusEffectParam { .. }
                | AudioCmd::SetBusEqParam { .. }
                | AudioCmd::SetGroupEffectParam { .. }
                | AudioCmd::SetLayerGroupEqParam { .. }
                | AudioCmd::SetMasterEqParam { .. }
                | AudioCmd::SetMasterEffectParam { .. }
                | AudioCmd::SetVstParam { .. }
                // Playback control
                | AudioCmd::SetPlaying { .. }
                | AudioCmd::SetBpm { .. }
                | AudioCmd::ResetPlayhead
                // Automation (applied during playback)
                | AudioCmd::ApplyAutomation { .. }
        )
        // NOTE: SetInstrumentMixerParams, SetMasterParams, SetBusMixerParams,
        // RegisterActiveNote, ClearActiveNotes, ReleaseAllVoices are deliberately
        // NOT priority. They're bulk updates that can tolerate slightly higher latency.
    }
}

// Re-export AudioFeedback and ExportKind from imbolc-types
pub use imbolc_types::{AudioFeedback, ExportKind};
