use crate::{BufferId, TrackId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

/// Runtime recording state.
/// Tracks audio recording status and automation recording mode.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordingState {
    /// Whether audio recording is active
    #[serde(skip)]
    pub recording: bool,
    /// Duration of current recording in seconds
    #[serde(skip)]
    pub recording_secs: u64,
    /// Whether automation recording is enabled
    #[serde(skip)]
    pub automation_recording: bool,
    /// Path to a recently stopped recording, pending waveform load
    #[serde(skip)]
    pub pending_recording_path: Option<PathBuf>,
    /// Most recently completed recording path (used for waveform playback/append).
    #[serde(skip)]
    pub last_recording_path: Option<PathBuf>,
    /// Duration of the last completed recording, in seconds.
    #[serde(skip)]
    pub last_recording_duration_secs: Option<f32>,
    /// Display name for the last loaded/recorded waveform clip.
    #[serde(skip)]
    pub last_recording_name: Option<String>,
    /// If set, the just-stopped recording should be appended into this target WAV.
    #[serde(skip)]
    pub append_target_path: Option<PathBuf>,
    /// Temporary preview buffer for playback inside the waveform pane.
    #[serde(skip)]
    pub preview_buffer_id: Option<BufferId>,
    /// Active preview node ID (if waveform playback is currently running).
    #[serde(skip)]
    pub preview_node_id: Option<i32>,
    /// Wall-clock start time of the current preview playback.
    #[serde(skip)]
    pub preview_started_at: Option<Instant>,
    /// Expected preview duration in seconds.
    #[serde(skip)]
    pub preview_duration_secs: Option<f32>,
    /// Normalized start position (0..1) for current preview playback.
    #[serde(skip)]
    pub preview_start_norm: Option<f32>,
    /// Normalized end position (0..1) for current preview playback.
    #[serde(skip)]
    pub preview_end_norm: Option<f32>,
    /// Tracks armed for recording (instrument IDs)
    #[serde(skip)]
    pub armed_tracks: HashSet<TrackId>,
    /// Tracks currently recording (instrument IDs)
    #[serde(skip)]
    pub recording_tracks: HashSet<TrackId>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if either audio or automation recording is active
    pub fn is_any_recording(&self) -> bool {
        self.recording || self.automation_recording || !self.recording_tracks.is_empty()
    }

    /// Arm a track for recording
    pub fn arm_track(&mut self, id: TrackId) {
        self.armed_tracks.insert(id);
    }

    /// Disarm a track
    pub fn disarm_track(&mut self, id: TrackId) {
        self.armed_tracks.remove(&id);
    }

    /// Toggle track arm state
    pub fn toggle_arm(&mut self, id: TrackId) {
        if self.armed_tracks.contains(&id) {
            self.armed_tracks.remove(&id);
        } else {
            self.armed_tracks.insert(id);
        }
    }

    /// Check if a track is armed
    pub fn is_armed(&self, id: TrackId) -> bool {
        self.armed_tracks.contains(&id)
    }

    /// Check if a track is currently recording
    pub fn is_track_recording(&self, id: TrackId) -> bool {
        self.recording_tracks.contains(&id)
    }

    /// Start recording on a track
    pub fn start_track_recording(&mut self, id: TrackId) {
        self.recording_tracks.insert(id);
    }

    /// Stop recording on a track
    pub fn stop_track_recording(&mut self, id: TrackId) {
        self.recording_tracks.remove(&id);
    }

    /// Get all armed track IDs
    pub fn get_armed_tracks(&self) -> Vec<TrackId> {
        self.armed_tracks.iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_not_recording() {
        let state = RecordingState::default();
        assert!(!state.recording);
        assert!(!state.automation_recording);
        assert_eq!(state.recording_secs, 0);
        assert!(state.pending_recording_path.is_none());
        assert!(state.last_recording_path.is_none());
        assert!(state.last_recording_duration_secs.is_none());
        assert!(state.last_recording_name.is_none());
        assert!(state.append_target_path.is_none());
        assert!(state.preview_buffer_id.is_none());
        assert!(state.preview_node_id.is_none());
        assert!(state.preview_started_at.is_none());
        assert!(state.preview_duration_secs.is_none());
        assert!(state.preview_start_norm.is_none());
        assert!(state.preview_end_norm.is_none());
    }
}
