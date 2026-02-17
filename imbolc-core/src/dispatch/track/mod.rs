mod arpeggiator;
mod crud;
mod effects;
mod envelope;
mod eq;
mod filter;
mod groove;
mod layer;
mod lfo;
mod playback;
mod sample;
mod selection;

use crate::action::{AudioEffect, DispatchResult, TrackAction};
use crate::audio::AudioHandle;
use crate::state::AppState;

pub(super) fn dispatch_instrument(
    action: &TrackAction,
    state: &mut AppState,
    audio: &mut AudioHandle,
) -> DispatchResult {
    match action {
        TrackAction::Add(source_type) => crud::handle_add(state, *source_type),
        TrackAction::Delete(inst_id) => crud::handle_delete(state, audio, *inst_id),
        TrackAction::Edit(id) => crud::handle_edit(state, *id),
        TrackAction::SetState(update) => crud::handle_update(state, update),
        TrackAction::PlayNote(pitch, velocity) => {
            playback::handle_play_note(state, audio, *pitch, *velocity)
        }
        TrackAction::PlayNotes(ref pitches, velocity) => {
            playback::handle_play_notes(state, audio, pitches, *velocity)
        }
        TrackAction::Select(_)
        | TrackAction::SelectNext
        | TrackAction::SelectPrev
        | TrackAction::SelectFirst
        | TrackAction::SelectLast => selection::handle_select(state, action),
        TrackAction::PlayDrumPad(pad_idx, velocity) => {
            playback::handle_play_drum_pad(state, audio, *pad_idx, *velocity)
        }
        TrackAction::LoadSampleResult(instrument_id, ref path) => {
            sample::handle_load_sample_result(state, audio, *instrument_id, path)
        }
        TrackAction::AddEffect(id, ref effect_type) => {
            effects::handle_add_effect(state, *id, *effect_type)
        }
        TrackAction::RemoveEffect(id, effect_id) => {
            effects::handle_remove_effect(state, *id, *effect_id)
        }
        TrackAction::SetFilter(id, filter_type) => {
            filter::handle_set_filter(state, *id, *filter_type)
        }
        TrackAction::ToggleEffectBypass(id, effect_id) => {
            effects::handle_toggle_effect_bypass(state, *id, *effect_id)
        }
        TrackAction::ToggleFilter(id) => filter::handle_toggle_filter(state, *id),
        TrackAction::NextFilterType(id) => filter::handle_cycle_filter_type(state, *id),
        TrackAction::AdjustFilterCutoff(id, delta) => {
            filter::handle_adjust_filter_cutoff(state, *id, *delta)
        }
        TrackAction::AdjustFilterResonance(id, delta) => {
            filter::handle_adjust_filter_resonance(state, *id, *delta)
        }
        TrackAction::AdjustEffectParam(id, effect_id, param_idx, delta) => {
            effects::handle_adjust_effect_param(state, *id, *effect_id, *param_idx, *delta)
        }
        TrackAction::ToggleArpeggiator(_)
        | TrackAction::NextArpeggiatorDirection(_)
        | TrackAction::PrevArpeggiatorDirection(_)
        | TrackAction::NextArpeggiatorRate(_)
        | TrackAction::PrevArpeggiatorRate(_)
        | TrackAction::AdjustArpeggiatorOctaves(_, _)
        | TrackAction::AdjustArpeggiatorGate(_, _)
        | TrackAction::NextChordShape(_)
        | TrackAction::PrevChordShape(_)
        | TrackAction::ClearChordShape(_) => arpeggiator::dispatch(state, action),
        TrackAction::LoadIRResult(instrument_id, effect_id, ref path) => {
            effects::handle_load_ir_result(state, audio, *instrument_id, *effect_id, path)
        }
        TrackAction::ShowVstEffectParams(instrument_id, effect_id) => {
            effects::handle_open_vst_effect_params(*instrument_id, *effect_id)
        }
        TrackAction::SetEqualizerParam(instrument_id, band_idx, param, value) => {
            eq::handle_set_eq_param(state, audio, *instrument_id, *band_idx, *param, *value)
        }
        TrackAction::ToggleEqualizer(instrument_id) => {
            eq::handle_toggle_eq(state, *instrument_id)
        }
        TrackAction::LinkLayer(a, b) => layer::handle_link_layer(state, *a, *b),
        TrackAction::UnlinkLayer(id) => layer::handle_unlink_layer(state, *id),
        TrackAction::AdjustLayerOctaveOffset(id, delta) => {
            layer::handle_adjust_layer_octave_offset(state, *id, *delta)
        }
        // Per-track groove settings
        TrackAction::SetTrackSwing(_, _)
        | TrackAction::SetTrackSwingGrid(_, _)
        | TrackAction::AdjustTrackSwing(_, _)
        | TrackAction::SetTrackHumanizeVelocity(_, _)
        | TrackAction::AdjustTrackHumanizeVelocity(_, _)
        | TrackAction::SetTrackHumanizeTiming(_, _)
        | TrackAction::AdjustTrackHumanizeTiming(_, _)
        | TrackAction::SetTrackTimingOffset(_, _)
        | TrackAction::AdjustTrackTimingOffset(_, _)
        | TrackAction::ResetTrackGroove(_)
        // Per-track time signature
        | TrackAction::SetTrackTimeSignature(_, _)
        | TrackAction::NextTrackTimeSignature(_) => groove::dispatch(state, action),
        // LFO actions
        TrackAction::ToggleLfo(id) => lfo::handle_toggle_lfo(state, *id),
        TrackAction::AdjustLfoRate(id, delta) => {
            lfo::handle_adjust_lfo_rate(state, *id, *delta)
        }
        TrackAction::AdjustLfoDepth(id, delta) => {
            lfo::handle_adjust_lfo_depth(state, *id, *delta)
        }
        TrackAction::SetLfoShape(id, shape) => {
            lfo::handle_set_lfo_shape(state, *id, *shape)
        }
        TrackAction::SetLfoTarget(id, target) => {
            lfo::handle_set_lfo_target(state, *id, *target)
        }
        // Envelope actions
        TrackAction::AdjustEnvelopeAttack(id, delta) => {
            envelope::handle_adjust_envelope_attack(state, *id, *delta)
        }
        TrackAction::AdjustEnvelopeDecay(id, delta) => {
            envelope::handle_adjust_envelope_decay(state, *id, *delta)
        }
        TrackAction::AdjustEnvelopeSustain(id, delta) => {
            envelope::handle_adjust_envelope_sustain(state, *id, *delta)
        }
        TrackAction::AdjustEnvelopeRelease(id, delta) => {
            envelope::handle_adjust_envelope_release(state, *id, *delta)
        }
        // Channel config
        TrackAction::ToggleChannelConfig(id) => {
            handle_toggle_channel_config(state, *id)
        }
        // Processing chain reordering
        TrackAction::MoveStage(id, stage_idx, direction) => {
            handle_move_stage(state, *id, *stage_idx, *direction)
        }
    }
}

fn handle_move_stage(
    state: &mut AppState,
    id: crate::state::TrackId,
    stage_idx: usize,
    direction: i8,
) -> DispatchResult {
    imbolc_types::reduce::reduce_action(
        &imbolc_types::DomainAction::Track(TrackAction::MoveStage(id, stage_idx, direction)),
        &mut state.tracks,
        &mut state.session,
    );
    let mut result = DispatchResult::none();
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    result
}

fn handle_toggle_channel_config(state: &mut AppState, id: crate::state::TrackId) -> DispatchResult {
    imbolc_types::reduce::reduce_action(
        &imbolc_types::DomainAction::Track(TrackAction::ToggleChannelConfig(id)),
        &mut state.tracks,
        &mut state.session,
    );
    let mut result = DispatchResult::default();
    result
        .audio_effects
        .push(AudioEffect::RebuildRoutingForInstrument(id));
    result
}
