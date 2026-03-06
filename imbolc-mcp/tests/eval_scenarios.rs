//! Layer 3: Scenario evals.
//!
//! End-to-end tests using MockDaw — an in-process DAW mock that processes
//! IpcRequest → IpcResponse using reduce_action().

#[allow(dead_code)]
mod common;

use common::MockDaw;
use imbolc_mcp::state_view;
use imbolc_types::ipc::*;
use imbolc_types::state::session::MusicalSettings;
use imbolc_types::{
    DomainAction, EffectType, MixerAction, SessionAction, SourceType, TrackAction, TrackId,
};

// ============================================================================
// Track lifecycle
// ============================================================================

#[test]
fn scenario_add_track_and_query() {
    let mut daw = MockDaw::new();
    daw.dispatch(DomainAction::Track(TrackAction::Add(SourceType::Saw)));

    match daw.process(IpcRequest::GetTracks) {
        IpcResponse::Tracks(tracks) => {
            assert_eq!(tracks.tracks.len(), 1);
            let out = state_view::format_tracks(&tracks);
            common::assert_contains(&out, "Saw");
        }
        other => panic!("Expected Tracks, got: {:?}", other),
    }
}

#[test]
fn scenario_add_multiple_tracks() {
    let mut daw = MockDaw::new();
    daw.dispatch(DomainAction::Track(TrackAction::Add(SourceType::Saw)));
    daw.dispatch(DomainAction::Track(TrackAction::Add(SourceType::Kick)));
    daw.dispatch(DomainAction::Track(TrackAction::Add(SourceType::EPiano)));

    match daw.process(IpcRequest::GetTracks) {
        IpcResponse::Tracks(tracks) => {
            assert_eq!(tracks.tracks.len(), 3);
            let out = state_view::format_tracks(&tracks);
            common::assert_contains(&out, "Saw");
            common::assert_contains(&out, "Kick");
            common::assert_contains(&out, "Electric Piano");
        }
        other => panic!("Expected Tracks, got: {:?}", other),
    }
}

// ============================================================================
// Track detail
// ============================================================================

#[test]
fn scenario_get_track_detail_after_add() {
    let mut daw = MockDaw::new();
    daw.dispatch(DomainAction::Track(TrackAction::Add(SourceType::Saw)));
    let id = daw.first_track_id();

    match daw.process(IpcRequest::GetTrackDetail { track_id: id }) {
        IpcResponse::TrackDetail(track) => {
            let out = state_view::format_track_detail(&track);
            common::assert_contains(&out, "Saw");
            common::assert_contains(&out, "level=");
        }
        other => panic!("Expected TrackDetail, got: {:?}", other),
    }
}

#[test]
fn scenario_track_not_found() {
    let mut daw = MockDaw::new();
    let resp = daw.process(IpcRequest::GetTrackDetail {
        track_id: TrackId::new(999),
    });
    assert!(
        matches!(resp, IpcResponse::TrackNotFound(_)),
        "Expected TrackNotFound, got: {:?}",
        resp
    );
}

// ============================================================================
// Effects
// ============================================================================

#[test]
fn scenario_add_effect_and_query() {
    let mut daw = MockDaw::new();
    daw.dispatch(DomainAction::Track(TrackAction::Add(SourceType::Saw)));
    let id = daw.first_track_id();

    daw.dispatch(DomainAction::Track(TrackAction::AddEffect(
        id,
        EffectType::Reverb,
    )));

    match daw.process(IpcRequest::GetEffects { track_id: id }) {
        IpcResponse::Effects(effects) => {
            assert_eq!(effects.len(), 1);
            assert_eq!(effects[0].effect_type, EffectType::Reverb);
        }
        other => panic!("Expected Effects, got: {:?}", other),
    }
}

// ============================================================================
// Session
// ============================================================================

#[test]
fn scenario_set_bpm_and_query_status() {
    let mut daw = MockDaw::new();
    daw.dispatch(DomainAction::Session(SessionAction::SetSession(
        MusicalSettings {
            bpm: 140,
            ..Default::default()
        },
    )));

    match daw.process(IpcRequest::GetStatus) {
        IpcResponse::Status(info) => {
            assert_eq!(info.bpm, 140.0);
        }
        other => panic!("Expected Status, got: {:?}", other),
    }
}

// ============================================================================
// Mixer
// ============================================================================

#[test]
fn scenario_mute_track_and_query() {
    let mut daw = MockDaw::new();
    daw.dispatch(DomainAction::Track(TrackAction::Add(SourceType::Saw)));

    // ToggleMute operates on the mixer selection index (0-based)
    daw.dispatch(DomainAction::Mixer(MixerAction::ToggleMute));

    match daw.process(IpcRequest::GetTracks) {
        IpcResponse::Tracks(tracks) => {
            assert!(tracks.tracks[0].channel_strip.mute);
        }
        other => panic!("Expected Tracks, got: {:?}", other),
    }
}
