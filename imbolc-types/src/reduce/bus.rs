use crate::{
    BusAction, EqParamKind, LayerGroupAction, MixerSelection, OutputTarget, SessionState,
    TrackState,
};

pub(super) fn reduce_bus(
    action: &BusAction,
    session: &mut SessionState,
    instruments: &mut TrackState,
) -> bool {
    match action {
        BusAction::Add => {
            session.add_bus();
            true
        }
        BusAction::Remove(bus_id) => {
            let bus_id = *bus_id;
            if session.bus(bus_id).is_none() {
                return true;
            }
            for inst in &mut instruments.tracks {
                if inst.channel_strip.output_target == OutputTarget::Bus(bus_id) {
                    inst.channel_strip.output_target = OutputTarget::Master;
                }
                inst.disable_send_for_bus(bus_id);
            }
            for gm in &mut session.mixer.layer_group_mixers {
                if gm.channel_strip.output_target == OutputTarget::Bus(bus_id) {
                    gm.channel_strip.output_target = OutputTarget::Master;
                }
                gm.disable_send_for_bus(bus_id);
            }
            session.automation.remove_lanes_for_bus(bus_id);
            session.remove_bus(bus_id);
            if let MixerSelection::Bus(id) = session.mixer.selection {
                if id == bus_id {
                    let first_bus = session.bus_ids().next();
                    session.mixer.selection = first_bus
                        .map(MixerSelection::Bus)
                        .unwrap_or(MixerSelection::Master);
                }
            }
            true
        }
        BusAction::Rename(bus_id, name) => {
            if let Some(bus) = session.bus_mut(*bus_id) {
                bus.name = name.clone();
            }
            true
        }
        BusAction::AddEffect(bus_id, effect_type) => {
            if let Some(bus) = session.bus_mut(*bus_id) {
                bus.channel_strip.add_effect(*effect_type);
            }
            true
        }
        BusAction::RemoveEffect(bus_id, effect_id) => {
            if let Some(bus) = session.bus_mut(*bus_id) {
                bus.channel_strip.remove_effect(*effect_id);
            }
            true
        }
        BusAction::MoveEffect(bus_id, effect_id, direction) => {
            if let Some(bus) = session.bus_mut(*bus_id) {
                bus.channel_strip.move_effect(*effect_id, *direction);
            }
            true
        }
        BusAction::ToggleEffectBypass(bus_id, effect_id) => {
            if let Some(bus) = session.bus_mut(*bus_id) {
                if let Some(effect) = bus.channel_strip.effect_by_id_mut(*effect_id) {
                    effect.enabled = !effect.enabled;
                }
            }
            true
        }
        BusAction::AdjustEffectParam(bus_id, effect_id, param_idx, delta) => {
            if let Some(bus) = session.bus_mut(*bus_id) {
                if let Some(effect) = bus.channel_strip.effect_by_id_mut(*effect_id) {
                    if let Some(param) = effect.params.get_mut(param_idx.get()) {
                        param.adjust_delta(*delta);
                    }
                }
            }
            true
        }
    }
}

pub(super) fn reduce_layer_group(action: &LayerGroupAction, session: &mut SessionState) -> bool {
    match action {
        LayerGroupAction::AddEffect(group_id, effect_type) => {
            if let Some(gm) = session.mixer.layer_group_mixer_mut(*group_id) {
                gm.channel_strip.add_effect(*effect_type);
            }
            true
        }
        LayerGroupAction::RemoveEffect(group_id, effect_id) => {
            if let Some(gm) = session.mixer.layer_group_mixer_mut(*group_id) {
                gm.channel_strip.remove_effect(*effect_id);
            }
            true
        }
        LayerGroupAction::MoveEffect(group_id, effect_id, direction) => {
            if let Some(gm) = session.mixer.layer_group_mixer_mut(*group_id) {
                gm.channel_strip.move_effect(*effect_id, *direction);
            }
            true
        }
        LayerGroupAction::ToggleEffectBypass(group_id, effect_id) => {
            if let Some(gm) = session.mixer.layer_group_mixer_mut(*group_id) {
                if let Some(effect) = gm.channel_strip.effect_by_id_mut(*effect_id) {
                    effect.enabled = !effect.enabled;
                }
            }
            true
        }
        LayerGroupAction::AdjustEffectParam(group_id, effect_id, param_idx, delta) => {
            if let Some(gm) = session.mixer.layer_group_mixer_mut(*group_id) {
                if let Some(effect) = gm.channel_strip.effect_by_id_mut(*effect_id) {
                    if let Some(param) = effect.params.get_mut(param_idx.get()) {
                        param.adjust_delta(*delta);
                    }
                }
            }
            true
        }
        LayerGroupAction::ToggleEq(group_id) => {
            if let Some(gm) = session.mixer.layer_group_mixer_mut(*group_id) {
                gm.toggle_eq();
            }
            true
        }
        LayerGroupAction::SetEqParam(group_id, band_idx, param, value) => {
            if let Some(gm) = session.mixer.layer_group_mixer_mut(*group_id) {
                if let Some(eq) = gm.eq_mut() {
                    if let Some(band) = eq.bands.get_mut(*band_idx) {
                        match param {
                            EqParamKind::Freq => band.freq = value.clamp(20.0, 20000.0),
                            EqParamKind::Gain => band.gain = value.clamp(-24.0, 24.0),
                            EqParamKind::Q => band.q = value.clamp(0.1, 10.0),
                            EqParamKind::Enabled => band.enabled = *value > 0.5,
                        }
                    }
                }
            }
            true
        }
        LayerGroupAction::Rename(group_id, name) => {
            if let Some(gm) = session.mixer.layer_group_mixer_mut(*group_id) {
                gm.name = name.clone();
            }
            true
        }
    }
}
