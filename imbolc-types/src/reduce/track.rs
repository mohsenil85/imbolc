use crate::{
    BusId, EqualizerParamKind, FilterType, Param, ParamValue, SessionState, SourceType,
    TrackAction, TrackId, TrackState,
};

pub(super) fn reduce(
    action: &TrackAction,
    instruments: &mut TrackState,
    session: &mut SessionState,
) -> bool {
    match action {
        TrackAction::Add(source_type) => {
            let id = instruments.add_track(*source_type);
            initialize_track_from_registries(id, *source_type, instruments, session);
            session.piano_roll.add_sequence(id);
            true
        }
        TrackAction::Delete(id) => {
            instruments.remove_track(*id);
            session.piano_roll.remove_sequence(*id);
            session.automation.remove_lanes_for_instrument(*id);
            session.arrangement.remove_instrument_data(*id);
            true
        }
        TrackAction::Edit(id) => {
            instruments.editing_track_id = Some(*id);
            true
        }
        TrackAction::SetState(update) => {
            if let Some(instrument) = instruments.track_mut(update.id) {
                instrument.source = update.source;
                instrument.source_params = update.source_params.clone();
                instrument.channel_strip.processing_chain = update.processing_chain.clone();
                instrument.modulation.lfo = update.lfo.clone();
                instrument.modulation.amp_envelope = update.amp_envelope.clone();
                instrument.polyphonic = update.polyphonic;
                instrument.channel_strip.active = update.active;
            }
            true
        }
        TrackAction::AddEffect(id, effect_type) => {
            if let Some(instrument) = instruments.track_mut(*id) {
                instrument.add_effect(*effect_type);
            }
            true
        }
        TrackAction::RemoveEffect(id, effect_id) => {
            if let Some(instrument) = instruments.track_mut(*id) {
                instrument.remove_effect(*effect_id);
            }
            true
        }
        TrackAction::MoveStage(id, chain_idx, direction) => {
            if let Some(instrument) = instruments.track_mut(*id) {
                instrument.move_stage(*chain_idx, *direction);
            }
            true
        }
        TrackAction::SetFilter(id, filter_type) => {
            if let Some(instrument) = instruments.track_mut(*id) {
                instrument.set_filter(*filter_type);
            }
            true
        }
        TrackAction::ToggleEffectBypass(id, effect_id) => {
            if let Some(instrument) = instruments.track_mut(*id) {
                if let Some(effect) = instrument.effects_mut().find(|e| e.id == *effect_id) {
                    effect.enabled = !effect.enabled;
                }
            }
            true
        }
        TrackAction::ToggleFilter(id) => {
            if let Some(instrument) = instruments.track_mut(*id) {
                instrument.toggle_filter();
            }
            true
        }
        TrackAction::NextFilterType(id) => {
            if let Some(instrument) = instruments.track_mut(*id) {
                if let Some(filter) = instrument.filter_mut() {
                    filter.filter_type = match filter.filter_type {
                        FilterType::Lpf => FilterType::Hpf,
                        FilterType::Hpf => FilterType::Bpf,
                        FilterType::Bpf => FilterType::Notch,
                        FilterType::Notch => FilterType::Comb,
                        FilterType::Comb => FilterType::Allpass,
                        FilterType::Allpass => FilterType::Vowel,
                        FilterType::Vowel => FilterType::ResDrive,
                        FilterType::ResDrive => FilterType::ParametricEq,
                        FilterType::ParametricEq => FilterType::Lpf,
                    };
                    filter.extra_params = filter.filter_type.default_extra_params();
                }
            }
            true
        }
        TrackAction::AdjustFilterCutoff(id, delta) => {
            if let Some(instrument) = instruments.track_mut(*id) {
                if let Some(filter) = instrument.filter_mut() {
                    filter.cutoff.value = (filter.cutoff.value + delta * filter.cutoff.max * 0.02)
                        .clamp(filter.cutoff.min, filter.cutoff.max);
                }
            }
            true
        }
        TrackAction::AdjustFilterResonance(id, delta) => {
            if let Some(instrument) = instruments.track_mut(*id) {
                if let Some(filter) = instrument.filter_mut() {
                    filter.resonance.value = (filter.resonance.value + delta * 0.05)
                        .clamp(filter.resonance.min, filter.resonance.max);
                }
            }
            true
        }
        TrackAction::AdjustEffectParam(id, effect_id, param_idx, delta) => {
            if let Some(instrument) = instruments.track_mut(*id) {
                if let Some(effect) = instrument.effects_mut().find(|e| e.id == *effect_id) {
                    if let Some(param) = effect.params.get_mut(param_idx.get()) {
                        param.adjust_delta(*delta);
                    }
                }
            }
            true
        }
        // PlayNote/PlayNotes/PlayDrumPad: audio side effects only, no state mutation
        TrackAction::PlayNote(_, _)
        | TrackAction::PlayNotes(_, _)
        | TrackAction::PlayDrumPad(_, _) => true,

        TrackAction::Select(idx) => {
            if *idx < instruments.tracks.len() {
                instruments.selected = Some(*idx);
            }
            true
        }
        TrackAction::SelectNext => {
            instruments.select_next();
            true
        }
        TrackAction::SelectPrev => {
            instruments.select_prev();
            true
        }
        TrackAction::SelectFirst => {
            if !instruments.tracks.is_empty() {
                instruments.selected = Some(0);
            }
            true
        }
        TrackAction::SelectLast => {
            if !instruments.tracks.is_empty() {
                instruments.selected = Some(instruments.tracks.len() - 1);
            }
            true
        }
        TrackAction::LoadSampleResult(instrument_id, path) => {
            let buffer_id = instruments.next_sampler_buffer_id;
            instruments.next_sampler_buffer_id += 1;
            let sample_name = path.file_stem().map(|s| s.to_string_lossy().to_string());
            if let Some(instrument) = instruments.track_mut(*instrument_id) {
                if let Some(config) = instrument.sampler_config_mut() {
                    config.buffer_id = Some(buffer_id);
                    config.sample_name = sample_name;
                }
            }
            true
        }
        TrackAction::ToggleArpeggiator(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.note_input.arpeggiator.enabled = !inst.note_input.arpeggiator.enabled;
            }
            true
        }
        TrackAction::NextArpeggiatorDirection(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.note_input.arpeggiator.direction =
                    inst.note_input.arpeggiator.direction.next();
            }
            true
        }
        TrackAction::PrevArpeggiatorDirection(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.note_input.arpeggiator.direction =
                    inst.note_input.arpeggiator.direction.prev();
            }
            true
        }
        TrackAction::NextArpeggiatorRate(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.note_input.arpeggiator.rate = inst.note_input.arpeggiator.rate.next();
            }
            true
        }
        TrackAction::PrevArpeggiatorRate(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.note_input.arpeggiator.rate = inst.note_input.arpeggiator.rate.prev();
            }
            true
        }
        TrackAction::AdjustArpeggiatorOctaves(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.note_input.arpeggiator.octaves =
                    (inst.note_input.arpeggiator.octaves as i8 + delta).clamp(1, 4) as u8;
            }
            true
        }
        TrackAction::AdjustArpeggiatorGate(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.note_input.arpeggiator.gate =
                    (inst.note_input.arpeggiator.gate + delta).clamp(0.1, 1.0);
            }
            true
        }
        TrackAction::NextChordShape(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.note_input.chord_shape = match inst.note_input.chord_shape {
                    None => Some(crate::ChordShape::Major),
                    Some(shape) => Some(shape.next()),
                };
            }
            true
        }
        TrackAction::PrevChordShape(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.note_input.chord_shape = match inst.note_input.chord_shape {
                    None => Some(crate::ChordShape::Octave),
                    Some(crate::ChordShape::Major) => None,
                    Some(shape) => Some(shape.prev()),
                };
            }
            true
        }
        TrackAction::ClearChordShape(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.note_input.chord_shape = None;
            }
            true
        }
        TrackAction::LoadIRResult(instrument_id, effect_id, path) => {
            let buffer_id = instruments.next_sampler_buffer_id;
            instruments.next_sampler_buffer_id += 1;
            if let Some(instrument) = instruments.track_mut(*instrument_id) {
                if let Some(effect) = instrument.effects_mut().find(|e| e.id == *effect_id) {
                    if let Some(param) = effect.params.iter_mut().find(|p| p.name == "ir_buffer") {
                        param.value = crate::ParamValue::Int(buffer_id as i32);
                    }
                }
                instrument.convolution_ir_path = Some(path.to_string_lossy().to_string());
            }
            true
        }
        // ShowVstEffectParams: navigation only, no state mutation
        TrackAction::ShowVstEffectParams(_, _) => true,

        TrackAction::SetEqualizerParam(instrument_id, band_idx, param, value) => {
            if let Some(instrument) = instruments.track_mut(*instrument_id) {
                if let Some(eq) = instrument.eq_mut() {
                    if let Some(band) = eq.bands.get_mut(*band_idx) {
                        match param {
                            EqualizerParamKind::Freq => band.freq = value.clamp(20.0, 20000.0),
                            EqualizerParamKind::Gain => band.gain = value.clamp(-24.0, 24.0),
                            EqualizerParamKind::Q => band.q = value.clamp(0.1, 10.0),
                            EqualizerParamKind::Enabled => band.enabled = *value > 0.5,
                            EqualizerParamKind::Slope => {
                                band.slope = crate::FilterSlope::from_normalized(*value)
                            }
                            EqualizerParamKind::BandType => {
                                let allowed = crate::EqBandType::allowed_types(*band_idx);
                                let idx = (*value as usize).min(allowed.len().saturating_sub(1));
                                let new_type = allowed[idx];
                                if new_type != band.band_type {
                                    band.band_type = new_type;
                                    if !new_type.has_gain() {
                                        band.gain = 0.0;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            true
        }
        TrackAction::ToggleEqualizer(instrument_id) => {
            if let Some(instrument) = instruments.track_mut(*instrument_id) {
                instrument.toggle_eq();
            }
            true
        }
        TrackAction::LinkLayer(a, b) => {
            reduce_link_layer(instruments, session, *a, *b);
            true
        }
        TrackAction::UnlinkLayer(id) => {
            reduce_unlink_layer(instruments, session, *id);
            true
        }
        TrackAction::AdjustLayerOctaveOffset(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.layer.octave_offset = (inst.layer.octave_offset + delta).clamp(-4, 4);
            }
            true
        }
        // Groove settings
        TrackAction::SetTrackSwing(id, value) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.groove.swing_amount = value.map(|v| v.clamp(0.0, 1.0));
            }
            true
        }
        TrackAction::SetTrackSwingGrid(id, grid) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.groove.swing_grid = *grid;
            }
            true
        }
        TrackAction::AdjustTrackSwing(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                let current = inst
                    .groove
                    .swing_amount
                    .unwrap_or(session.piano_roll.swing_amount);
                inst.groove.swing_amount = Some((current + delta).clamp(0.0, 1.0));
            }
            true
        }
        TrackAction::SetTrackHumanizeVelocity(id, value) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.groove.humanize_velocity = value.map(|v| v.clamp(0.0, 1.0));
            }
            true
        }
        TrackAction::AdjustTrackHumanizeVelocity(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                let current = inst
                    .groove
                    .humanize_velocity
                    .unwrap_or(session.humanize.velocity);
                inst.groove.humanize_velocity = Some((current + delta).clamp(0.0, 1.0));
            }
            true
        }
        TrackAction::SetTrackHumanizeTiming(id, value) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.groove.humanize_timing = value.map(|v| v.clamp(0.0, 1.0));
            }
            true
        }
        TrackAction::AdjustTrackHumanizeTiming(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                let current = inst
                    .groove
                    .humanize_timing
                    .unwrap_or(session.humanize.timing);
                inst.groove.humanize_timing = Some((current + delta).clamp(0.0, 1.0));
            }
            true
        }
        TrackAction::SetTrackTimingOffset(id, value) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.groove.timing_offset_ms = value.clamp(-50.0, 50.0);
            }
            true
        }
        TrackAction::AdjustTrackTimingOffset(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.groove.timing_offset_ms =
                    (inst.groove.timing_offset_ms + delta).clamp(-50.0, 50.0);
            }
            true
        }
        TrackAction::ResetTrackGroove(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.groove.reset();
            }
            true
        }
        TrackAction::SetTrackTimeSignature(id, ts) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.groove.time_signature = *ts;
            }
            true
        }
        TrackAction::NextTrackTimeSignature(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.groove.time_signature = match inst.groove.time_signature {
                    None => Some((4, 4)),
                    Some((4, 4)) => Some((3, 4)),
                    Some((3, 4)) => Some((5, 4)),
                    Some((5, 4)) => Some((6, 8)),
                    Some((6, 8)) => Some((7, 8)),
                    Some((7, 8)) => Some((12, 8)),
                    Some((12, 8)) => None,
                    Some(_) => Some((4, 4)),
                };
            }
            true
        }
        // LFO actions
        TrackAction::ToggleLfo(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.modulation.lfo.enabled = !inst.modulation.lfo.enabled;
            }
            true
        }
        TrackAction::AdjustLfoRate(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.modulation.lfo.rate =
                    (inst.modulation.lfo.rate + delta * 0.5).clamp(0.1, 20.0);
            }
            true
        }
        TrackAction::AdjustLfoDepth(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.modulation.lfo.depth =
                    (inst.modulation.lfo.depth + delta * 0.05).clamp(0.0, 1.0);
            }
            true
        }
        TrackAction::SetLfoShape(id, shape) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.modulation.lfo.shape = *shape;
            }
            true
        }
        TrackAction::SetLfoTarget(id, target) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.modulation.lfo.target = *target;
            }
            true
        }
        // Envelope actions
        TrackAction::AdjustEnvelopeAttack(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.modulation.amp_envelope.attack =
                    (inst.modulation.amp_envelope.attack + delta * 0.1).clamp(0.001, 2.0);
            }
            true
        }
        TrackAction::AdjustEnvelopeDecay(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.modulation.amp_envelope.decay =
                    (inst.modulation.amp_envelope.decay + delta * 0.1).clamp(0.001, 2.0);
            }
            true
        }
        TrackAction::AdjustEnvelopeSustain(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.modulation.amp_envelope.sustain =
                    (inst.modulation.amp_envelope.sustain + delta * 0.05).clamp(0.0, 1.0);
            }
            true
        }
        TrackAction::AdjustEnvelopeRelease(id, delta) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.modulation.amp_envelope.release =
                    (inst.modulation.amp_envelope.release + delta * 0.2).clamp(0.001, 5.0);
            }
            true
        }
        // Channel config
        TrackAction::ToggleChannelConfig(id) => {
            if let Some(inst) = instruments.track_mut(*id) {
                inst.channel_strip.channel_config = inst.channel_strip.channel_config.toggle();
            }
            true
        }
    }
}

/// Initialize instrument name and source_params from Custom/VST registries.
/// Called by the reducer and by `AppState::add_instrument()` to keep them aligned.
pub fn initialize_track_from_registries(
    id: TrackId,
    source_type: SourceType,
    instruments: &mut TrackState,
    session: &SessionState,
) {
    if let SourceType::Custom(custom_id) = source_type {
        if let Some(synthdef) = session.custom_synthdefs.get(custom_id) {
            if let Some(inst) = instruments.track_mut(id) {
                inst.name = format!("{}-{}", synthdef.synthdef_name, id);
                inst.source_params = synthdef
                    .params
                    .iter()
                    .map(|p| Param {
                        name: p.name.clone(),
                        value: ParamValue::Float(p.default),
                        min: p.min,
                        max: p.max,
                    })
                    .collect();
            }
        }
    }
    if let SourceType::Vst(vst_id) = source_type {
        if let Some(plugin) = session.vst_plugins.get(vst_id) {
            if let Some(inst) = instruments.track_mut(id) {
                inst.name = format!("{}-{}", plugin.name.to_lowercase(), id);
                inst.source_params = plugin
                    .params
                    .iter()
                    .map(|p| Param {
                        name: p.name.clone(),
                        value: ParamValue::Float(p.default),
                        min: 0.0,
                        max: 1.0,
                    })
                    .collect();
            }
        }
    }
}

fn reduce_link_layer(
    instruments: &mut TrackState,
    session: &mut SessionState,
    a: TrackId,
    b: TrackId,
) {
    if a == b {
        return;
    }
    let group_b = instruments.track(b).and_then(|i| i.layer.group);
    let group_a = instruments.track(a).and_then(|i| i.layer.group);
    let group_id = match (group_a, group_b) {
        (_, Some(g)) => g,
        (Some(g), None) => g,
        (None, None) => instruments.next_layer_group(),
    };
    if let Some(inst) = instruments.track_mut(a) {
        inst.layer.group = Some(group_id);
    }
    if let Some(inst) = instruments.track_mut(b) {
        inst.layer.group = Some(group_id);
    }
    // Auto-create GroupMixer if new group
    let bus_ids: Vec<BusId> = session.mixer.bus_ids().collect();
    if session.mixer.layer_group_mixer(group_id).is_none() {
        session.mixer.add_layer_group_mixer(group_id, &bus_ids);
    }
}

fn reduce_unlink_layer(instruments: &mut TrackState, session: &mut SessionState, id: TrackId) {
    let old_group = instruments.track(id).and_then(|i| i.layer.group);
    if let Some(inst) = instruments.track_mut(id) {
        inst.layer.group = None;
    }
    if let Some(g) = old_group {
        let remaining: Vec<TrackId> = instruments
            .tracks
            .iter()
            .filter(|i| i.layer.group == Some(g))
            .map(|i| i.id)
            .collect();
        if remaining.len() <= 1 {
            if remaining.len() == 1 {
                if let Some(inst) = instruments.track_mut(remaining[0]) {
                    inst.layer.group = None;
                }
            }
            session.mixer.remove_layer_group_mixer(g);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::custom_synthdef::{CustomSynthDef, ParamSpec};
    use crate::state::vst::{VstParamSpec, VstPlugin, VstPluginKind};
    use crate::{CustomSynthDefId, VstPluginId};
    use std::path::PathBuf;

    fn make_session_with_custom() -> SessionState {
        let mut session = SessionState::new();
        let synthdef = CustomSynthDef {
            id: CustomSynthDefId::new(0),
            name: "test".to_string(),
            synthdef_name: "imbolc_test".to_string(),
            source_path: PathBuf::from("/tmp/test.scd"),
            params: vec![
                ParamSpec {
                    name: "freq".to_string(),
                    default: 440.0,
                    min: 20.0,
                    max: 20000.0,
                },
                ParamSpec {
                    name: "amp".to_string(),
                    default: 0.5,
                    min: 0.0,
                    max: 1.0,
                },
            ],
        };
        session.custom_synthdefs.add(synthdef);
        session
    }

    fn make_session_with_vst() -> SessionState {
        let mut session = SessionState::new();
        let plugin = VstPlugin {
            id: VstPluginId::new(0),
            name: "TestSynth".to_string(),
            plugin_path: PathBuf::from("/tmp/test.vst3"),
            kind: VstPluginKind::Track,
            params: vec![
                VstParamSpec {
                    index: 0,
                    name: "cutoff".to_string(),
                    default: 0.7,
                    label: None,
                },
                VstParamSpec {
                    index: 1,
                    name: "resonance".to_string(),
                    default: 0.3,
                    label: None,
                },
            ],
        };
        session.vst_plugins.add(plugin);
        session
    }

    #[test]
    fn add_custom_initializes_params_from_registry() {
        let mut session = make_session_with_custom();
        let mut instruments = TrackState::new();
        let custom_id = CustomSynthDefId::new(0);

        reduce(
            &TrackAction::Add(SourceType::Custom(custom_id)),
            &mut instruments,
            &mut session,
        );

        let inst = &instruments.tracks[0];
        assert!(inst.name.starts_with("imbolc_test-"));
        assert_eq!(inst.source_params.len(), 2);
        assert_eq!(inst.source_params[0].name, "freq");
        assert_eq!(inst.source_params[0].min, 20.0);
        assert_eq!(inst.source_params[0].max, 20000.0);
        assert_eq!(inst.source_params[1].name, "amp");
    }

    #[test]
    fn add_vst_initializes_params_from_registry() {
        let mut session = make_session_with_vst();
        let mut instruments = TrackState::new();
        let vst_id = VstPluginId::new(0);

        reduce(
            &TrackAction::Add(SourceType::Vst(vst_id)),
            &mut instruments,
            &mut session,
        );

        let inst = &instruments.tracks[0];
        assert!(inst.name.starts_with("testsynth-"));
        assert_eq!(inst.source_params.len(), 2);
        assert_eq!(inst.source_params[0].name, "cutoff");
        assert_eq!(inst.source_params[0].min, 0.0);
        assert_eq!(inst.source_params[0].max, 1.0);
        assert_eq!(inst.source_params[1].name, "resonance");
    }

    #[test]
    fn add_basic_source_uses_default_params() {
        let mut session = SessionState::new();
        let mut instruments = TrackState::new();

        reduce(
            &TrackAction::Add(SourceType::Saw),
            &mut instruments,
            &mut session,
        );

        let inst = &instruments.tracks[0];
        // Saw gets default_params() from SourceType, not from registries
        assert_eq!(inst.source_params, SourceType::Saw.default_params());
    }

    #[test]
    fn initialize_helper_matches_reducer_for_custom() {
        let mut session = make_session_with_custom();
        let custom_id = CustomSynthDefId::new(0);

        // Path A: via reducer
        let mut instruments_a = TrackState::new();
        reduce(
            &TrackAction::Add(SourceType::Custom(custom_id)),
            &mut instruments_a,
            &mut session,
        );

        // Path B: via helper directly
        let mut instruments_b = TrackState::new();
        let id = instruments_b.add_track(SourceType::Custom(custom_id));
        initialize_track_from_registries(
            id,
            SourceType::Custom(custom_id),
            &mut instruments_b,
            &session,
        );

        let a = &instruments_a.tracks[0];
        let b = &instruments_b.tracks[0];
        assert_eq!(a.name, b.name);
        assert_eq!(a.source_params.len(), b.source_params.len());
        for (pa, pb) in a.source_params.iter().zip(b.source_params.iter()) {
            assert_eq!(pa.name, pb.name);
            assert_eq!(pa.min, pb.min);
            assert_eq!(pa.max, pb.max);
        }
    }

    #[test]
    fn initialize_helper_matches_reducer_for_vst() {
        let mut session = make_session_with_vst();
        let vst_id = VstPluginId::new(0);

        // Path A: via reducer
        let mut instruments_a = TrackState::new();
        reduce(
            &TrackAction::Add(SourceType::Vst(vst_id)),
            &mut instruments_a,
            &mut session,
        );

        // Path B: via helper directly
        let mut instruments_b = TrackState::new();
        let id = instruments_b.add_track(SourceType::Vst(vst_id));
        initialize_track_from_registries(id, SourceType::Vst(vst_id), &mut instruments_b, &session);

        let a = &instruments_a.tracks[0];
        let b = &instruments_b.tracks[0];
        assert_eq!(a.name, b.name);
        assert_eq!(a.source_params.len(), b.source_params.len());
        for (pa, pb) in a.source_params.iter().zip(b.source_params.iter()) {
            assert_eq!(pa.name, pb.name);
            assert_eq!(pa.min, pb.min);
            assert_eq!(pa.max, pb.max);
        }
    }
}
