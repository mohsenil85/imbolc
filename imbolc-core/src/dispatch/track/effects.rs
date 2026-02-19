use crate::action::{AudioEffect, DispatchResult, NavIntent, VstTarget};
use crate::state::automation::AutomationTarget;
use crate::state::AppState;
use imbolc_audio::AudioHandle;
use imbolc_types::{DomainAction, ParamValue, TrackAction};

use super::super::automation::record_automation_point;

fn reduce(state: &mut AppState, action: &TrackAction) {
    imbolc_types::reduce::reduce_action(
        &DomainAction::Track(action.clone()),
        &mut state.tracks,
        &mut state.session,
    );
}

pub(super) fn handle_add_effect(
    state: &mut AppState,
    id: crate::state::TrackId,
    effect_type: crate::state::EffectType,
) -> DispatchResult {
    reduce(state, &TrackAction::AddEffect(id, effect_type));
    let mut result = DispatchResult::with_nav(NavIntent::Pop);
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    result
}

pub(super) fn handle_remove_effect(
    state: &mut AppState,
    id: crate::state::TrackId,
    effect_id: crate::state::EffectId,
) -> DispatchResult {
    reduce(state, &TrackAction::RemoveEffect(id, effect_id));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    result
}

pub(super) fn handle_toggle_effect_bypass(
    state: &mut AppState,
    id: crate::state::TrackId,
    effect_id: crate::state::EffectId,
) -> DispatchResult {
    reduce(state, &TrackAction::ToggleEffectBypass(id, effect_id));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}

pub(super) fn handle_adjust_effect_param(
    state: &mut AppState,
    id: crate::state::TrackId,
    effect_id: crate::state::EffectId,
    param_idx: imbolc_types::ParamIndex,
    delta: f32,
) -> DispatchResult {
    reduce(
        state,
        &TrackAction::AdjustEffectParam(id, effect_id, param_idx, delta),
    );

    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);

    // Read post-mutation value for targeted param + automation recording
    // Extract value first to avoid borrow conflict with record_automation_point
    let param_value = state
        .tracks
        .track(id)
        .and_then(|inst| inst.effects().find(|e| e.id == effect_id))
        .and_then(|effect| effect.params.get(param_idx.get()))
        .and_then(|param| match param.value {
            ParamValue::Float(v) => Some(v),
            _ => None,
        });

    if let Some(value) = param_value {
        result
            .audio_effects
            .push(AudioEffect::SetEffectParam(id, effect_id, param_idx, value));
        if state.recording.automation_recording && state.audio.playing {
            let target = AutomationTarget::effect_param(id, effect_id, param_idx);
            let normalized = target.normalize_value(value);
            record_automation_point(state, target, normalized);
            result.audio_effects.push(AudioEffect::UpdateAutomation);
        }
    }
    result
}

pub(super) fn handle_load_ir_result(
    state: &mut AppState,
    audio: &mut AudioHandle,
    instrument_id: crate::state::TrackId,
    effect_id: crate::state::EffectId,
    path: &std::path::Path,
) -> DispatchResult {
    // Copy to project assets if project is saved
    let asset_path = if let Some(ref project_path) = state.project.path {
        match super::super::helpers::copy_to_project_assets(path, project_path) {
            Ok(p) => p,
            Err(e) => {
                return DispatchResult::with_status(
                    audio.status(),
                    format!("Asset copy failed: {}", e),
                );
            }
        }
    } else {
        return DispatchResult::with_status(
            audio.status(),
            "Save project before importing samples",
        );
    };

    // Load sample into audio engine before reducer increments the buffer_id
    let buffer_id = state.tracks.next_sampler_buffer_id;
    if audio.is_running() {
        let _ = audio.load_sample(buffer_id, &asset_path.to_string_lossy());
    }

    reduce(
        state,
        &TrackAction::LoadIRResult(instrument_id, effect_id, asset_path),
    );

    let mut result = DispatchResult::with_nav(NavIntent::Pop);
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(instrument_id));
    result
}

pub(super) fn handle_open_vst_effect_params(
    instrument_id: crate::state::TrackId,
    effect_id: crate::state::EffectId,
) -> DispatchResult {
    DispatchResult::with_nav(NavIntent::OpenVstParams(
        instrument_id,
        VstTarget::Effect(effect_id),
    ))
}

// --- Note effect handlers ---

pub(super) fn handle_add_note_effect(
    state: &mut AppState,
    id: crate::state::TrackId,
    effect_type: imbolc_types::NoteEffectType,
) -> DispatchResult {
    reduce(state, &TrackAction::AddNoteEffect(id, effect_type));
    let mut result = DispatchResult::with_nav(NavIntent::Pop);
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}

pub(super) fn handle_remove_note_effect(
    state: &mut AppState,
    id: crate::state::TrackId,
    effect_id: crate::state::EffectId,
) -> DispatchResult {
    reduce(state, &TrackAction::RemoveNoteEffect(id, effect_id));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}

pub(super) fn handle_toggle_note_effect_bypass(
    state: &mut AppState,
    id: crate::state::TrackId,
    effect_id: crate::state::EffectId,
) -> DispatchResult {
    reduce(state, &TrackAction::ToggleNoteEffectBypass(id, effect_id));
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}

pub(super) fn handle_adjust_note_effect_param(
    state: &mut AppState,
    id: crate::state::TrackId,
    effect_id: crate::state::EffectId,
    param_idx: imbolc_types::ParamIndex,
    delta: f32,
) -> DispatchResult {
    reduce(
        state,
        &TrackAction::AdjustNoteEffectParam(id, effect_id, param_idx, delta),
    );
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
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
    fn add_effect() {
        let (mut state, id) = setup();
        let result = handle_add_effect(&mut state, id, crate::state::EffectType::Delay);

        // Verify effect was added
        let inst = state.tracks.track(id).unwrap();
        assert_eq!(inst.effects().count(), 1);
        let effect = inst.effects().next().unwrap();
        assert_eq!(effect.effect_type, crate::state::EffectType::Delay);
        assert!(effect.enabled);

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));

        // Verify nav intent (Pop)
        assert_eq!(result.nav.len(), 1);
    }

    #[test]
    fn remove_effect() {
        let (mut state, id) = setup();
        handle_add_effect(&mut state, id, crate::state::EffectType::Delay);

        let effect_id = state.tracks.track(id).unwrap().effects().next().unwrap().id;

        let result = handle_remove_effect(&mut state, id, effect_id);

        // Verify effect was removed
        let inst = state.tracks.track(id).unwrap();
        assert_eq!(inst.effects().count(), 0);

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildRoutingForInstrument(id)));
    }

    #[test]
    fn toggle_effect_bypass() {
        let (mut state, id) = setup();
        handle_add_effect(&mut state, id, crate::state::EffectType::Reverb);

        let effect_id = state.tracks.track(id).unwrap().effects().next().unwrap().id;

        // Effect starts enabled
        assert!(
            state
                .tracks
                .track(id)
                .unwrap()
                .effect_by_id(effect_id)
                .unwrap()
                .enabled
        );

        let result = handle_toggle_effect_bypass(&mut state, id, effect_id);

        // Effect should now be bypassed
        assert!(
            !state
                .tracks
                .track(id)
                .unwrap()
                .effect_by_id(effect_id)
                .unwrap()
                .enabled
        );

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));
    }

    #[test]
    fn adjust_effect_param() {
        let (mut state, id) = setup();
        handle_add_effect(&mut state, id, crate::state::EffectType::Delay);

        let effect_id = state.tracks.track(id).unwrap().effects().next().unwrap().id;

        let param_idx = imbolc_types::ParamIndex::new(0);
        let result = handle_adjust_effect_param(&mut state, id, effect_id, param_idx, 0.1);

        // Verify audio effects
        assert!(result
            .audio_effects
            .contains(&AudioEffect::RebuildInstruments));

        // Should have SetEffectParam for targeted param update
        let has_set_effect = result.audio_effects.iter().any(|e| {
            matches!(
                e,
                AudioEffect::SetEffectParam(inst_id, eff_id, p_idx, _)
                    if *inst_id == id && *eff_id == effect_id && *p_idx == param_idx
            )
        });
        assert!(has_set_effect, "expected SetEffectParam in audio_effects");
    }
}
