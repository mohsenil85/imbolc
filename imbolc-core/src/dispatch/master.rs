use crate::action::{AudioEffect, DispatchResult, EqualizerParamKind, NavIntent};
use crate::state::AppState;
use imbolc_audio::AudioHandle;
use imbolc_types::DomainAction;
use imbolc_types::MasterAction;

fn reduce(state: &mut AppState, action: &MasterAction) {
    imbolc_types::reduce::reduce_action(
        &DomainAction::Master(action.clone()),
        &mut state.tracks,
        &mut state.session,
    );
}

/// Dispatch master bus actions (effect chain + EQ on master output)
pub fn dispatch_master(
    action: &MasterAction,
    state: &mut AppState,
    audio: &mut AudioHandle,
) -> DispatchResult {
    // Delegate pure state mutation to the shared reducer
    reduce(state, action);

    // Orchestration: AudioEffects and navigation
    let mut result = DispatchResult::none();

    match action {
        MasterAction::AddEffect(_) => {
            result
                .audio_effects
                .push(AudioEffect::RebuildMasterProcessing);
            result.audio_effects.push(AudioEffect::RebuildSession);
            result.nav.push(NavIntent::Pop);
        }

        MasterAction::RemoveEffect(_) => {
            result
                .audio_effects
                .push(AudioEffect::RebuildMasterProcessing);
            result.audio_effects.push(AudioEffect::RebuildSession);
        }

        MasterAction::MoveEffect(_, _) => {
            result
                .audio_effects
                .push(AudioEffect::RebuildMasterProcessing);
            result.audio_effects.push(AudioEffect::RebuildSession);
        }

        MasterAction::ToggleEffectBypass(_) => {
            result
                .audio_effects
                .push(AudioEffect::RebuildMasterProcessing);
            result.audio_effects.push(AudioEffect::RebuildSession);
        }

        MasterAction::AdjustEffectParam(effect_id, param_idx, _delta) => {
            result.audio_effects.push(AudioEffect::RebuildSession);
            // Read back the param value after reducer mutation for targeted audio update
            let targeted_value = state
                .session
                .mixer
                .master_channel_strip
                .effect_by_id(*effect_id)
                .and_then(|effect| effect.params.get(param_idx.get()))
                .map(|param| param.value.to_f32());
            if let Some(value) = targeted_value {
                result.audio_effects.push(AudioEffect::SetMasterEffectParam(
                    *effect_id, *param_idx, value,
                ));
            }
        }

        MasterAction::ToggleEqualizer => {
            result
                .audio_effects
                .push(AudioEffect::RebuildMasterProcessing);
            result.audio_effects.push(AudioEffect::RebuildSession);
        }

        MasterAction::SetEqualizerParam(effect_id, band_idx, param, value) => {
            if audio.is_running() {
                let sc_param = format!("b{}_{}", band_idx, param.as_str());
                let sc_value = match param {
                    EqualizerParamKind::Q => 1.0 / value,
                    EqualizerParamKind::Slope => {
                        imbolc_types::FilterSlope::from_normalized(*value).sc_slope_value() as f32
                    }
                    EqualizerParamKind::BandType => {
                        let allowed = imbolc_types::EqBandType::allowed_types(*band_idx);
                        let idx = (*value as usize).min(allowed.len().saturating_sub(1));
                        allowed[idx].sc_type_index() as f32
                    }
                    _ => *value,
                };
                let _ = audio.set_master_eq_param(*effect_id, &sc_param, sc_value);
            }
            result.audio_effects.push(AudioEffect::RebuildSession);
        }

        MasterAction::AddEqualizer | MasterAction::RemoveEqualizer(_) => {
            result
                .audio_effects
                .push(AudioEffect::RebuildMasterProcessing);
            result.audio_effects.push(AudioEffect::RebuildSession);
        }
    }

    result
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use crate::state::EffectType;
    use imbolc_types::ParamIndex;

    fn setup() -> (AppState, AudioHandle) {
        (AppState::new(), AudioHandle::new())
    }

    #[test]
    fn master_add_effect_dispatch() {
        let (mut state, mut audio) = setup();
        let result = dispatch_master(
            &MasterAction::AddEffect(EffectType::Limiter),
            &mut state,
            &mut audio,
        );
        let effects = state.session.mixer.master_channel_strip.effects_vec();
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].effect_type, EffectType::Limiter);
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildMasterProcessing));
        assert!(result.audio_effects.contains(&AudioEffect::RebuildSession));
        assert!(!result.nav.is_empty()); // Pop after add
    }

    #[test]
    fn master_remove_effect_dispatch() {
        let (mut state, mut audio) = setup();
        dispatch_master(
            &MasterAction::AddEffect(EffectType::Limiter),
            &mut state,
            &mut audio,
        );
        let effect_id = state.session.mixer.master_channel_strip.effects_vec()[0].id;

        let result = dispatch_master(
            &MasterAction::RemoveEffect(effect_id),
            &mut state,
            &mut audio,
        );
        assert!(state
            .session
            .mixer
            .master_channel_strip
            .effects_vec()
            .is_empty());
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildMasterProcessing));
    }

    #[test]
    fn master_toggle_bypass_dispatch() {
        let (mut state, mut audio) = setup();
        dispatch_master(
            &MasterAction::AddEffect(EffectType::Reverb),
            &mut state,
            &mut audio,
        );
        let effect_id = state.session.mixer.master_channel_strip.effects_vec()[0].id;
        assert!(state.session.mixer.master_channel_strip.effects_vec()[0].enabled);

        dispatch_master(
            &MasterAction::ToggleEffectBypass(effect_id),
            &mut state,
            &mut audio,
        );
        assert!(!state.session.mixer.master_channel_strip.effects_vec()[0].enabled);
    }

    #[test]
    fn master_adjust_effect_param_dispatch() {
        let (mut state, mut audio) = setup();
        dispatch_master(
            &MasterAction::AddEffect(EffectType::Reverb),
            &mut state,
            &mut audio,
        );
        let effect_id = state.session.mixer.master_channel_strip.effects_vec()[0].id;

        let result = dispatch_master(
            &MasterAction::AdjustEffectParam(effect_id, ParamIndex::new(0), 1.0),
            &mut state,
            &mut audio,
        );
        assert!(result
            .audio_effects
            .iter()
            .any(|e| matches!(e, AudioEffect::SetMasterEffectParam(..))));
    }

    #[test]
    fn master_toggle_eq_dispatch() {
        let (mut state, mut audio) = setup();
        // new_bus() starts with no EQ
        assert!(state.session.mixer.master_eq().is_none());

        let result = dispatch_master(&MasterAction::ToggleEqualizer, &mut state, &mut audio);
        assert!(state.session.mixer.master_eq().is_some());
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildMasterProcessing));

        dispatch_master(&MasterAction::ToggleEqualizer, &mut state, &mut audio);
        assert!(state.session.mixer.master_eq().is_none());
    }

    #[test]
    fn master_set_eq_param_dispatch() {
        let (mut state, mut audio) = setup();
        // Enable EQ first
        dispatch_master(&MasterAction::ToggleEqualizer, &mut state, &mut audio);
        let eq_id = state
            .session
            .mixer
            .master_channel_strip
            .first_eq_id()
            .unwrap();

        let result = dispatch_master(
            &MasterAction::SetEqualizerParam(eq_id, 0, EqualizerParamKind::Gain, 6.0),
            &mut state,
            &mut audio,
        );
        let eq = state.session.mixer.master_eq().unwrap();
        assert_eq!(eq.bands[0].gain, 6.0);
        assert!(result.audio_effects.contains(&AudioEffect::RebuildSession));
    }
}
