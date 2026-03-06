//! Layer 2: State view evals.
//!
//! Construct known state objects, call formatting functions,
//! verify output contains expected patterns.

#[allow(dead_code)]
mod common;

use imbolc_mcp::state_view;
use imbolc_types::{EffectType, SourceType, TrackState};

// ============================================================================
// format_status
// ============================================================================

#[test]
fn format_status_basic() {
    let status = common::make_status(120.0, "C", "Major", 0, false);
    let out = state_view::format_status(&status);
    common::assert_contains_all(
        &out,
        &["BPM: 120", "Key: C", "Major", "Tracks: 0", "stopped"],
    );
}

#[test]
fn format_status_playing() {
    let status = common::make_status(140.0, "A", "Minor", 3, true);
    let out = state_view::format_status(&status);
    common::assert_contains(&out, "playing");
    common::assert_contains(&out, "BPM: 140");
}

// ============================================================================
// format_tracks
// ============================================================================

#[test]
fn format_tracks_empty() {
    let tracks = TrackState::new();
    let out = state_view::format_tracks(&tracks);
    assert_eq!(out, "No tracks.");
}

#[test]
fn format_tracks_multiple() {
    let mut tracks = TrackState::new();
    tracks.add_track(SourceType::Saw);
    tracks.add_track(SourceType::Saw);
    tracks.add_track(SourceType::Saw);
    let out = state_view::format_tracks(&tracks);
    common::assert_contains_all(&out, &["Track 1", "Track 2", "Track 3", "(Saw)"]);
}

// ============================================================================
// format_track_detail
// ============================================================================

#[test]
fn format_track_detail_basic() {
    let mut tracks = TrackState::new();
    tracks.add_track(SourceType::Saw);
    let track = &tracks.tracks[0];
    let out = state_view::format_track_detail(track);
    common::assert_contains_all(&out, &["Saw", "level=", "pan=", "LFO:", "Envelope:"]);
}

#[test]
fn format_track_detail_muted() {
    let mut tracks = TrackState::new();
    tracks.add_track(SourceType::Saw);
    tracks.tracks[0].channel_strip.mute = true;
    let track = &tracks.tracks[0];
    let out = state_view::format_track_detail(track);
    common::assert_contains(&out, "mute=true");
}

#[test]
fn format_track_detail_effects() {
    let mut tracks = TrackState::new();
    tracks.add_track(SourceType::Saw);
    tracks.tracks[0].add_effect(EffectType::Reverb);
    let track = &tracks.tracks[0];
    let out = state_view::format_track_detail(track);
    common::assert_contains(&out, "Effects:");
    common::assert_contains(&out, "Reverb");
}
