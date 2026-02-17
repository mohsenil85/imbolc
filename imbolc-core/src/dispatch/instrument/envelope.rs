use crate::action::{AudioEffect, DispatchResult};
use crate::dispatch::helpers::maybe_record_automation;
use crate::state::automation::AutomationTarget;
use crate::state::AppState;
use imbolc_types::{DomainAction, InstrumentAction, TrackId};

// Envelope parameter ranges
const ATTACK_MIN: f32 = 0.001;
const ATTACK_MAX: f32 = 2.0;
const DECAY_MIN: f32 = 0.001;
const DECAY_MAX: f32 = 2.0;
const RELEASE_MIN: f32 = 0.001;
const RELEASE_MAX: f32 = 5.0;

fn reduce(state: &mut AppState, action: &InstrumentAction) {
    imbolc_types::reduce::reduce_action(
        &DomainAction::Instrument(action.clone()),
        &mut state.instruments,
        &mut state.session,
    );
}

pub(super) fn handle_adjust_envelope_attack(
    state: &mut AppState,
    id: TrackId,
    delta: f32,
) -> DispatchResult {
    reduce(state, &InstrumentAction::AdjustEnvelopeAttack(id, delta));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    if let Some(inst) = state.instruments.instrument(id) {
        let v = inst.modulation.amp_envelope.attack;
        let normalized = (v - ATTACK_MIN) / (ATTACK_MAX - ATTACK_MIN);
        maybe_record_automation(state, &mut result, AutomationTarget::attack(id), normalized);
    }
    result
}

pub(super) fn handle_adjust_envelope_decay(
    state: &mut AppState,
    id: TrackId,
    delta: f32,
) -> DispatchResult {
    reduce(state, &InstrumentAction::AdjustEnvelopeDecay(id, delta));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    if let Some(inst) = state.instruments.instrument(id) {
        let v = inst.modulation.amp_envelope.decay;
        let normalized = (v - DECAY_MIN) / (DECAY_MAX - DECAY_MIN);
        maybe_record_automation(state, &mut result, AutomationTarget::decay(id), normalized);
    }
    result
}

pub(super) fn handle_adjust_envelope_sustain(
    state: &mut AppState,
    id: TrackId,
    delta: f32,
) -> DispatchResult {
    reduce(state, &InstrumentAction::AdjustEnvelopeSustain(id, delta));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    if let Some(inst) = state.instruments.instrument(id) {
        let v = inst.modulation.amp_envelope.sustain;
        maybe_record_automation(state, &mut result, AutomationTarget::sustain(id), v);
    }
    result
}

pub(super) fn handle_adjust_envelope_release(
    state: &mut AppState,
    id: TrackId,
    delta: f32,
) -> DispatchResult {
    reduce(state, &InstrumentAction::AdjustEnvelopeRelease(id, delta));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    if let Some(inst) = state.instruments.instrument(id) {
        let v = inst.modulation.amp_envelope.release;
        let normalized = (v - RELEASE_MIN) / (RELEASE_MAX - RELEASE_MIN);
        maybe_record_automation(
            state,
            &mut result,
            AutomationTarget::release(id),
            normalized,
        );
    }
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
        let id = state.add_instrument(SourceType::Saw);
        (state, id)
    }

    #[test]
    fn adjust_attack() {
        let (mut state, id) = setup();
        let original = state
            .instruments
            .instrument(id)
            .unwrap()
            .modulation
            .amp_envelope
            .attack;

        let result = handle_adjust_envelope_attack(&mut state, id, 0.1);

        let new_val = state
            .instruments
            .instrument(id)
            .unwrap()
            .modulation
            .amp_envelope
            .attack;
        assert!((new_val - original).abs() > f32::EPSILON);

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));
    }

    #[test]
    fn adjust_decay() {
        let (mut state, id) = setup();
        let original = state
            .instruments
            .instrument(id)
            .unwrap()
            .modulation
            .amp_envelope
            .decay;

        let result = handle_adjust_envelope_decay(&mut state, id, 0.1);

        let new_val = state
            .instruments
            .instrument(id)
            .unwrap()
            .modulation
            .amp_envelope
            .decay;
        assert!((new_val - original).abs() > f32::EPSILON);

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));
    }

    #[test]
    fn adjust_sustain() {
        let (mut state, id) = setup();
        let original = state
            .instruments
            .instrument(id)
            .unwrap()
            .modulation
            .amp_envelope
            .sustain;

        let result = handle_adjust_envelope_sustain(&mut state, id, 0.2);

        let new_val = state
            .instruments
            .instrument(id)
            .unwrap()
            .modulation
            .amp_envelope
            .sustain;
        assert!((new_val - original).abs() > f32::EPSILON);

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));
    }

    #[test]
    fn adjust_release() {
        let (mut state, id) = setup();
        let original = state
            .instruments
            .instrument(id)
            .unwrap()
            .modulation
            .amp_envelope
            .release;

        let result = handle_adjust_envelope_release(&mut state, id, 0.2);

        let new_val = state
            .instruments
            .instrument(id)
            .unwrap()
            .modulation
            .amp_envelope
            .release;
        assert!((new_val - original).abs() > f32::EPSILON);

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));
    }
}
