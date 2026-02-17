use crate::action::{AudioEffect, DispatchResult, LfoParamKind};
use crate::dispatch::helpers::maybe_record_automation;
use crate::state::automation::AutomationTarget;
use crate::state::AppState;
use imbolc_types::{DomainAction, InstrumentAction, LfoShape, ParameterTarget, TrackId};

// LFO parameter ranges
const LFO_RATE_MIN: f32 = 0.1;
const LFO_RATE_MAX: f32 = 20.0;

fn reduce(state: &mut AppState, action: &InstrumentAction) {
    imbolc_types::reduce::reduce_action(
        &DomainAction::Track(action.clone()),
        &mut state.tracks,
        &mut state.session,
    );
}

pub(super) fn handle_toggle_lfo(state: &mut AppState, id: TrackId) -> DispatchResult {
    reduce(state, &InstrumentAction::ToggleLfo(id));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    result
}

pub(super) fn handle_adjust_lfo_rate(
    state: &mut AppState,
    id: TrackId,
    delta: f32,
) -> DispatchResult {
    reduce(state, &InstrumentAction::AdjustLfoRate(id, delta));

    let mut result = DispatchResult::none();
    if let Some(instrument) = state.tracks.track(id) {
        let rate = instrument.modulation.lfo.rate;
        let normalized = (rate - LFO_RATE_MIN) / (LFO_RATE_MAX - LFO_RATE_MIN);
        maybe_record_automation(
            state,
            &mut result,
            AutomationTarget::lfo_rate(id),
            normalized,
        );
        result
            .audio_effects
            .push(AudioEffect::SetLfoParam(id, LfoParamKind::Rate, rate));
    }

    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}

pub(super) fn handle_adjust_lfo_depth(
    state: &mut AppState,
    id: TrackId,
    delta: f32,
) -> DispatchResult {
    reduce(state, &InstrumentAction::AdjustLfoDepth(id, delta));

    let mut result = DispatchResult::none();
    if let Some(instrument) = state.tracks.track(id) {
        let depth = instrument.modulation.lfo.depth;
        maybe_record_automation(state, &mut result, AutomationTarget::lfo_depth(id), depth);
        result
            .audio_effects
            .push(AudioEffect::SetLfoParam(id, LfoParamKind::Depth, depth));
    }

    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}

pub(super) fn handle_set_lfo_shape(
    state: &mut AppState,
    id: TrackId,
    shape: LfoShape,
) -> DispatchResult {
    reduce(state, &InstrumentAction::SetLfoShape(id, shape));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    result
}

pub(super) fn handle_set_lfo_target(
    state: &mut AppState,
    id: TrackId,
    target: ParameterTarget,
) -> DispatchResult {
    reduce(state, &InstrumentAction::SetLfoTarget(id, target));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    result
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use crate::state::SourceType;

    fn setup() -> (AppState, imbolc_types::TrackId) {
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);
        (state, id)
    }

    #[test]
    fn toggle_lfo() {
        let (mut state, id) = setup();

        // LFO starts disabled
        assert!(!state.tracks.track(id).unwrap().modulation.lfo.enabled);

        let result = handle_toggle_lfo(&mut state, id);

        // LFO should now be enabled
        assert!(state.tracks.track(id).unwrap().modulation.lfo.enabled);

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));

        // Toggle again to disable
        handle_toggle_lfo(&mut state, id);
        assert!(!state.tracks.track(id).unwrap().modulation.lfo.enabled);
    }

    #[test]
    fn adjust_lfo_rate() {
        let (mut state, id) = setup();
        let original = state.tracks.track(id).unwrap().modulation.lfo.rate;

        let result = handle_adjust_lfo_rate(&mut state, id, 1.0);

        let new_val = state.tracks.track(id).unwrap().modulation.lfo.rate;
        assert!((new_val - original).abs() > f32::EPSILON);

        // Verify SetLfoParam audio effect
        let has_set_lfo = result.audio_effects.iter().any(|e| {
            matches!(e, AudioEffect::SetLfoParam(inst_id, LfoParamKind::Rate, _) if *inst_id == id)
        });
        assert!(has_set_lfo, "expected SetLfoParam(Rate) in audio_effects");

        // Also has RebuildInstruments
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
    }

    #[test]
    fn adjust_lfo_depth() {
        let (mut state, id) = setup();
        let original = state.tracks.track(id).unwrap().modulation.lfo.depth;

        let result = handle_adjust_lfo_depth(&mut state, id, 0.1);

        let new_val = state.tracks.track(id).unwrap().modulation.lfo.depth;
        assert!((new_val - original).abs() > f32::EPSILON);

        // Verify SetLfoParam audio effect
        let has_set_lfo = result.audio_effects.iter().any(|e| {
            matches!(e, AudioEffect::SetLfoParam(inst_id, LfoParamKind::Depth, _) if *inst_id == id)
        });
        assert!(has_set_lfo, "expected SetLfoParam(Depth) in audio_effects");

        // Also has RebuildInstruments
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
    }

    #[test]
    fn set_lfo_shape() {
        let (mut state, id) = setup();

        // Default shape is Sine
        assert_eq!(
            state.tracks.track(id).unwrap().modulation.lfo.shape,
            LfoShape::Sine
        );

        let result = handle_set_lfo_shape(&mut state, id, LfoShape::Square);

        assert_eq!(
            state.tracks.track(id).unwrap().modulation.lfo.shape,
            LfoShape::Square
        );

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));
    }

    #[test]
    fn set_lfo_target() {
        let (mut state, id) = setup();

        // Default target is FilterCutoff
        assert_eq!(
            state.tracks.track(id).unwrap().modulation.lfo.target,
            ParameterTarget::FilterCutoff
        );

        let result = handle_set_lfo_target(&mut state, id, ParameterTarget::Pan);

        assert_eq!(
            state.tracks.track(id).unwrap().modulation.lfo.target,
            ParameterTarget::Pan
        );

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));
    }
}
