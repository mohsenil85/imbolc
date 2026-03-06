//! Shared test utilities for imbolc-mcp evals.

use imbolc_types::ipc::*;
use imbolc_types::reduce::reduce_action;
use imbolc_types::{
    DomainAction, EffectSlot, ProcessingStage, SessionState, SourceExtra, TrackId, TrackState,
};

// ============================================================================
// Assertion helpers
// ============================================================================

/// Assert that `haystack` contains `needle`, with a descriptive panic message.
pub fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "Expected output to contain {:?}, but got:\n{}",
        needle,
        haystack
    );
}

/// Assert that `haystack` contains all given substrings.
pub fn assert_contains_all(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert_contains(haystack, needle);
    }
}

/// Assert that `haystack` does NOT contain `needle`.
#[allow(dead_code)]
pub fn assert_not_contains(haystack: &str, needle: &str) {
    assert!(
        !haystack.contains(needle),
        "Expected output to NOT contain {:?}, but got:\n{}",
        needle,
        haystack
    );
}

// ============================================================================
// State builders
// ============================================================================

/// Build a `StatusInfo` with known values.
pub fn make_status(
    bpm: f64,
    key: &str,
    scale: &str,
    track_count: usize,
    playing: bool,
) -> StatusInfo {
    StatusInfo {
        bpm,
        key: key.to_string(),
        scale: scale.to_string(),
        track_count,
        playing,
        project_path: Some("/test/project.sqlite".to_string()),
    }
}

// ============================================================================
// MockDaw
// ============================================================================

/// In-process DAW mock for scenario tests.
///
/// Processes `IpcRequest` → `IpcResponse` using `reduce_action()` for mutations.
/// Mirrors the logic in `imbolc-ui/src/mcp_listener.rs` without needing a real
/// DAW, Unix socket, or audio engine.
pub struct MockDaw {
    pub tracks: TrackState,
    pub session: SessionState,
}

impl MockDaw {
    pub fn new() -> Self {
        Self {
            tracks: TrackState::new(),
            session: SessionState::new(),
        }
    }

    /// Process an IPC request and return a response.
    pub fn process(&mut self, request: IpcRequest) -> IpcResponse {
        match request {
            IpcRequest::Hello(_info) => IpcResponse::Welcome { client_id: 1 },
            IpcRequest::Ping => IpcResponse::Pong,

            // --- Queries ---
            IpcRequest::GetStatus => IpcResponse::Status(StatusInfo {
                bpm: self.session.bpm as f64,
                key: format!("{:?}", self.session.key),
                scale: format!("{:?}", self.session.scale),
                track_count: self.tracks.tracks.len(),
                playing: false,
                project_path: None,
            }),
            IpcRequest::GetTracks => IpcResponse::Tracks(self.tracks.clone()),
            IpcRequest::GetTrackDetail { track_id } => match self.tracks.track(track_id) {
                Some(t) => IpcResponse::TrackDetail(Box::new(t.clone())),
                None => IpcResponse::TrackNotFound(track_id),
            },
            IpcRequest::GetSession => IpcResponse::Session(Box::new(self.session.clone())),
            IpcRequest::GetMixer => IpcResponse::Mixer(Box::new(self.session.mixer.clone())),
            IpcRequest::GetPianoRoll => {
                IpcResponse::PianoRoll(Box::new(self.session.piano_roll.clone()))
            }
            IpcRequest::GetSequencer { track_id } => match self.tracks.track(track_id) {
                Some(t) => match &t.source_extra {
                    SourceExtra::Kit(seq) => IpcResponse::Sequencer(Box::new(seq.clone())),
                    _ => IpcResponse::SequencerNotFound(track_id),
                },
                None => IpcResponse::SequencerNotFound(track_id),
            },
            IpcRequest::GetAutomation => {
                IpcResponse::Automation(Box::new(self.session.automation.clone()))
            }
            IpcRequest::GetArrangement => {
                IpcResponse::Arrangement(Box::new(self.session.arrangement.clone()))
            }
            IpcRequest::GetEffects { track_id } => match self.tracks.track(track_id) {
                Some(t) => {
                    let effects: Vec<EffectSlot> = t
                        .channel_strip
                        .processing_chain
                        .iter()
                        .filter_map(|stage| {
                            if let ProcessingStage::Effect(slot) = stage {
                                Some(slot.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    IpcResponse::Effects(effects)
                }
                None => IpcResponse::TrackNotFound(track_id),
            },

            // --- Mutations ---
            IpcRequest::DispatchAction(action) => {
                let reduced = reduce_action(&action, &mut self.tracks, &mut self.session);
                if reduced {
                    IpcResponse::ActionOk {
                        status_messages: vec![],
                    }
                } else {
                    IpcResponse::ActionRejected("Action not reducible in MockDaw".to_string())
                }
            }
            IpcRequest::DispatchCommand(_) => IpcResponse::CommandError(
                "MockDaw does not support REPL parsing; use DispatchAction".to_string(),
            ),
        }
    }

    /// Dispatch a domain action and assert it succeeds.
    pub fn dispatch(&mut self, action: DomainAction) -> IpcResponse {
        self.process(IpcRequest::DispatchAction(action))
    }

    /// Get the first track ID (useful after adding a single track).
    pub fn first_track_id(&self) -> TrackId {
        self.tracks.tracks[0].id
    }
}
