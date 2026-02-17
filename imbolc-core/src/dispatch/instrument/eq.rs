use crate::action::{AudioEffect, DispatchResult, EqualizerParamKind};
use crate::state::automation::AutomationTarget;
use crate::state::AppState;
use imbolc_audio::AudioHandle;
use imbolc_types::{DomainAction, InstrumentAction};

use super::super::automation::record_automation_point;

pub(super) fn handle_set_eq_param(
    state: &mut AppState,
    audio: &mut AudioHandle,
    instrument_id: crate::state::InstrumentId,
    band_idx: usize,
    param: EqualizerParamKind,
    value: f32,
) -> DispatchResult {
    imbolc_types::reduce::reduce_action(
        &DomainAction::Instrument(InstrumentAction::SetEqualizerParam(
            instrument_id,
            band_idx,
            param,
            value,
        )),
        &mut state.instruments,
        &mut state.session,
    );

    // Send real-time param update to audio engine
    if audio.is_running() {
        let sc_param = format!("b{}_{}", band_idx, param.as_str());
        let sc_value = if param == EqualizerParamKind::Q {
            1.0 / value
        } else {
            value
        };
        let _ = audio.set_eq_param(instrument_id, &sc_param, sc_value);
    }

    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);

    // Automation recording
    if state.recording.automation_recording && state.audio.playing {
        let target = match param {
            EqualizerParamKind::Freq => {
                Some(AutomationTarget::eq_band_freq(instrument_id, band_idx))
            }
            EqualizerParamKind::Gain => {
                Some(AutomationTarget::eq_band_gain(instrument_id, band_idx))
            }
            EqualizerParamKind::Q => Some(AutomationTarget::eq_band_q(instrument_id, band_idx)),
            EqualizerParamKind::Enabled => None,
        };
        if let Some(t) = target {
            let normalized = t.normalize_value(value);
            record_automation_point(state, t, normalized);
            result.audio_effects.push(AudioEffect::UpdateAutomation);
        }
    }
    result
}

pub(super) fn handle_toggle_eq(
    state: &mut AppState,
    instrument_id: crate::state::InstrumentId,
) -> DispatchResult {
    imbolc_types::reduce::reduce_action(
        &DomainAction::Instrument(InstrumentAction::ToggleEqualizer(instrument_id)),
        &mut state.instruments,
        &mut state.session,
    );
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(instrument_id));
    result
}
