use crate::action::{AudioEffect, DispatchResult};
use crate::state::AppState;
use crate::state::TrackId;
use imbolc_types::{DomainAction, TrackAction};

pub(super) fn handle_link_layer(state: &mut AppState, a: TrackId, b: TrackId) -> DispatchResult {
    if a == b {
        return DispatchResult::none();
    }
    imbolc_types::reduce::reduce_action(
        &DomainAction::Track(TrackAction::LinkLayer(a, b)),
        &mut state.tracks,
        &mut state.session,
    );
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildRouting);
    result.audio_effects.push(AudioEffect::RebuildSession);
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}

pub(super) fn handle_unlink_layer(state: &mut AppState, id: TrackId) -> DispatchResult {
    let had_group = state.tracks.track(id).and_then(|i| i.layer.group).is_some();
    imbolc_types::reduce::reduce_action(
        &DomainAction::Track(TrackAction::UnlinkLayer(id)),
        &mut state.tracks,
        &mut state.session,
    );
    let mut result = DispatchResult::none();
    if had_group {
        result.audio_effects.push(AudioEffect::RebuildRouting);
        result.audio_effects.push(AudioEffect::RebuildSession);
        result.audio_effects.push(AudioEffect::RebuildInstruments);
    }
    result
}

pub(super) fn handle_adjust_layer_octave_offset(
    state: &mut AppState,
    id: TrackId,
    delta: i8,
) -> DispatchResult {
    imbolc_types::reduce::reduce_action(
        &DomainAction::Track(TrackAction::AdjustLayerOctaveOffset(id, delta)),
        &mut state.tracks,
        &mut state.session,
    );
    DispatchResult::none()
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use crate::state::SourceType;

    #[test]
    fn adjust_layer_octave_offset_increments() {
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);
        assert_eq!(state.tracks.track(id).unwrap().layer.octave_offset, 0);

        handle_adjust_layer_octave_offset(&mut state, id, 1);
        assert_eq!(state.tracks.track(id).unwrap().layer.octave_offset, 1);

        handle_adjust_layer_octave_offset(&mut state, id, 1);
        assert_eq!(state.tracks.track(id).unwrap().layer.octave_offset, 2);
    }

    #[test]
    fn adjust_layer_octave_offset_decrements() {
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);

        handle_adjust_layer_octave_offset(&mut state, id, -1);
        assert_eq!(state.tracks.track(id).unwrap().layer.octave_offset, -1);
    }

    #[test]
    fn adjust_layer_octave_offset_clamps_high() {
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);

        for _ in 0..10 {
            handle_adjust_layer_octave_offset(&mut state, id, 1);
        }
        assert_eq!(state.tracks.track(id).unwrap().layer.octave_offset, 4);
    }

    #[test]
    fn adjust_layer_octave_offset_clamps_low() {
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);

        for _ in 0..10 {
            handle_adjust_layer_octave_offset(&mut state, id, -1);
        }
        assert_eq!(state.tracks.track(id).unwrap().layer.octave_offset, -4);
    }

    #[test]
    fn adjust_layer_octave_offset_no_audio_effects() {
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);

        let result = handle_adjust_layer_octave_offset(&mut state, id, 1);
        assert!(!result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(!result.audio_effects.contains(&AudioEffect::RebuildRouting));
    }
}
