use crate::action::{AudioEffect, DispatchResult, FilterParamKind};
use crate::dispatch::helpers::maybe_record_automation;
use crate::state::automation::AutomationTarget;
use crate::state::{AppState, InstrumentId};
use imbolc_types::{DomainAction, InstrumentAction};

fn reduce(state: &mut AppState, action: &InstrumentAction) {
    imbolc_types::reduce::reduce_action(
        &DomainAction::Instrument(action.clone()),
        &mut state.instruments,
        &mut state.session,
    );
}

pub(super) fn handle_set_filter(
    state: &mut AppState,
    id: InstrumentId,
    filter_type: Option<crate::state::FilterType>,
) -> DispatchResult {
    reduce(state, &InstrumentAction::SetFilter(id, filter_type));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    result
}

pub(super) fn handle_toggle_filter(state: &mut AppState, id: InstrumentId) -> DispatchResult {
    reduce(state, &InstrumentAction::ToggleFilter(id));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    result
}

pub(super) fn handle_cycle_filter_type(state: &mut AppState, id: InstrumentId) -> DispatchResult {
    reduce(state, &InstrumentAction::NextFilterType(id));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}

pub(super) fn handle_adjust_filter_cutoff(
    state: &mut AppState,
    id: InstrumentId,
    delta: f32,
) -> DispatchResult {
    reduce(state, &InstrumentAction::AdjustFilterCutoff(id, delta));

    let mut result = DispatchResult::none();
    // Read post-mutation value for automation recording and targeted param
    if let Some(instrument) = state.instruments.instrument(id) {
        if let Some(filter) = instrument.filter() {
            let cutoff = filter.cutoff.value;
            let target = AutomationTarget::filter_cutoff(id);
            let normalized = target.normalize_value(cutoff);
            maybe_record_automation(
                state,
                &mut result,
                AutomationTarget::filter_cutoff(id),
                normalized,
            );
            result.audio_effects.push(AudioEffect::SetFilterParam(
                id,
                FilterParamKind::Cutoff,
                cutoff,
            ));
        }
    }

    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}

pub(super) fn handle_adjust_filter_resonance(
    state: &mut AppState,
    id: InstrumentId,
    delta: f32,
) -> DispatchResult {
    reduce(state, &InstrumentAction::AdjustFilterResonance(id, delta));

    let mut result = DispatchResult::none();
    // Read post-mutation value for automation recording and targeted param
    if let Some(instrument) = state.instruments.instrument(id) {
        if let Some(filter) = instrument.filter() {
            let resonance = filter.resonance.value;
            let target = AutomationTarget::filter_resonance(id);
            let normalized = target.normalize_value(resonance);
            maybe_record_automation(
                state,
                &mut result,
                AutomationTarget::filter_resonance(id),
                normalized,
            );
            result.audio_effects.push(AudioEffect::SetFilterParam(
                id,
                FilterParamKind::Resonance,
                resonance,
            ));
        }
    }

    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use crate::state::SourceType;

    fn setup() -> (AppState, imbolc_types::InstrumentId) {
        let mut state = AppState::new();
        let id = state.add_instrument(SourceType::Saw);
        (state, id)
    }

    #[test]
    fn set_filter() {
        let (mut state, id) = setup();

        // No filter initially
        assert!(state.instruments.instrument(id).unwrap().filter().is_none());

        let result = handle_set_filter(&mut state, id, Some(imbolc_types::FilterType::Lpf));

        // Filter should now be set
        let inst = state.instruments.instrument(id).unwrap();
        let filter = inst.filter().expect("filter should be present");
        assert_eq!(filter.filter_type, imbolc_types::FilterType::Lpf);

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));
    }

    #[test]
    fn toggle_filter() {
        let (mut state, id) = setup();

        // Set a filter first
        handle_set_filter(&mut state, id, Some(imbolc_types::FilterType::Hpf));
        assert!(state.instruments.instrument(id).unwrap().filter().is_some());

        // Toggle removes the filter from the processing chain
        let result = handle_toggle_filter(&mut state, id);

        let inst = state.instruments.instrument(id).unwrap();
        assert!(inst.filter().is_none());

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));

        // Toggle again re-adds a default filter
        handle_toggle_filter(&mut state, id);
        assert!(state.instruments.instrument(id).unwrap().filter().is_some());
    }

    #[test]
    fn cycle_filter_type() {
        let (mut state, id) = setup();

        // Set a filter first
        handle_set_filter(&mut state, id, Some(imbolc_types::FilterType::Lpf));

        let result = handle_cycle_filter_type(&mut state, id);

        // Filter type should have changed from Lpf
        let inst = state.instruments.instrument(id).unwrap();
        let filter = inst.filter().unwrap();
        assert_ne!(filter.filter_type, imbolc_types::FilterType::Lpf);

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
    }

    #[test]
    fn adjust_filter_cutoff() {
        let (mut state, id) = setup();

        // Set a filter first
        handle_set_filter(&mut state, id, Some(imbolc_types::FilterType::Lpf));

        let original_cutoff = state
            .instruments
            .instrument(id)
            .unwrap()
            .filter()
            .unwrap()
            .cutoff
            .value;

        let result = handle_adjust_filter_cutoff(&mut state, id, 100.0);

        let new_cutoff = state
            .instruments
            .instrument(id)
            .unwrap()
            .filter()
            .unwrap()
            .cutoff
            .value;
        assert!((new_cutoff - original_cutoff).abs() > f32::EPSILON);

        // Verify SetFilterParam audio effect
        let has_set_filter = result.audio_effects.iter().any(|e| {
            matches!(e, AudioEffect::SetFilterParam(inst_id, FilterParamKind::Cutoff, _) if *inst_id == id)
        });
        assert!(
            has_set_filter,
            "expected SetFilterParam(Cutoff) in audio_effects"
        );
    }

    #[test]
    fn adjust_filter_resonance() {
        let (mut state, id) = setup();

        // Set a filter first
        handle_set_filter(&mut state, id, Some(imbolc_types::FilterType::Lpf));

        let original_resonance = state
            .instruments
            .instrument(id)
            .unwrap()
            .filter()
            .unwrap()
            .resonance
            .value;

        let result = handle_adjust_filter_resonance(&mut state, id, 0.1);

        let new_resonance = state
            .instruments
            .instrument(id)
            .unwrap()
            .filter()
            .unwrap()
            .resonance
            .value;
        assert!((new_resonance - original_resonance).abs() > f32::EPSILON);

        // Verify SetFilterParam audio effect
        let has_set_filter = result.audio_effects.iter().any(|e| {
            matches!(e, AudioEffect::SetFilterParam(inst_id, FilterParamKind::Resonance, _) if *inst_id == id)
        });
        assert!(
            has_set_filter,
            "expected SetFilterParam(Resonance) in audio_effects"
        );
    }
}
