use crate::action::{AudioEffect, DispatchResult};
use crate::state::AppState;
use imbolc_types::{DomainAction, TrackAction};

pub(super) fn dispatch(state: &mut AppState, action: &TrackAction) -> DispatchResult {
    imbolc_types::reduce::reduce_action(
        &DomainAction::Track(action.clone()),
        &mut state.tracks,
        &mut state.session,
    );
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}
