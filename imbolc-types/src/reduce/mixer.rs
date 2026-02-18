use crate::{
    BusId, MixerAction, MixerSelection, MixerSend, OutputTarget, SessionState, TrackState,
};

pub(super) fn reduce(
    action: &MixerAction,
    instruments: &mut TrackState,
    session: &mut SessionState,
) -> bool {
    match action {
        MixerAction::Move(delta) => {
            session.mixer.selection = match session.mixer.selection {
                MixerSelection::Track(idx) => {
                    let new_idx = (idx as i32 + *delta as i32)
                        .clamp(0, instruments.tracks.len().saturating_sub(1) as i32)
                        as usize;
                    MixerSelection::Track(new_idx)
                }
                MixerSelection::Group(current_id) => {
                    let group_ids: Vec<u32> = session
                        .mixer
                        .layer_group_mixers
                        .iter()
                        .map(|g| g.group_id)
                        .collect();
                    if group_ids.is_empty() {
                        return true;
                    }
                    let current_pos = group_ids
                        .iter()
                        .position(|&id| id == current_id)
                        .unwrap_or(0);
                    let new_pos = (current_pos as i32 + *delta as i32)
                        .clamp(0, group_ids.len().saturating_sub(1) as i32)
                        as usize;
                    MixerSelection::Group(group_ids[new_pos])
                }
                MixerSelection::Bus(current_id) => {
                    let bus_ids: Vec<BusId> = session.bus_ids().collect();
                    if bus_ids.is_empty() {
                        return true;
                    }
                    let current_pos = bus_ids.iter().position(|&id| id == current_id).unwrap_or(0);
                    let new_pos = (current_pos as i32 + *delta as i32)
                        .clamp(0, bus_ids.len().saturating_sub(1) as i32)
                        as usize;
                    MixerSelection::Bus(bus_ids[new_pos])
                }
                MixerSelection::Master => MixerSelection::Master,
            };
            if let MixerSelection::Track(idx) = session.mixer.selection {
                instruments.selected = Some(idx);
            }
            true
        }
        MixerAction::Jump(direction) => {
            session.mixer.selection = match session.mixer.selection {
                MixerSelection::Track(_) => {
                    if *direction > 0 {
                        MixerSelection::Track(0)
                    } else {
                        MixerSelection::Track(instruments.tracks.len().saturating_sub(1))
                    }
                }
                MixerSelection::Group(_) => {
                    let group_ids: Vec<u32> = session
                        .mixer
                        .layer_group_mixers
                        .iter()
                        .map(|g| g.group_id)
                        .collect();
                    if group_ids.is_empty() {
                        return true;
                    }
                    if *direction > 0 {
                        MixerSelection::Group(group_ids[0])
                    } else {
                        MixerSelection::Group(*group_ids.last().unwrap())
                    }
                }
                MixerSelection::Bus(_) => {
                    let bus_ids: Vec<BusId> = session.bus_ids().collect();
                    if bus_ids.is_empty() {
                        return true;
                    }
                    if *direction > 0 {
                        MixerSelection::Bus(bus_ids[0])
                    } else {
                        MixerSelection::Bus(*bus_ids.last().unwrap())
                    }
                }
                MixerSelection::Master => MixerSelection::Master,
            };
            if let MixerSelection::Track(idx) = session.mixer.selection {
                instruments.selected = Some(idx);
            }
            true
        }
        MixerAction::SelectAt(selection) => {
            session.mixer.selection = *selection;
            if let MixerSelection::Track(idx) = *selection {
                instruments.selected = Some(idx);
            }
            true
        }
        MixerAction::AdjustLevel(delta) => {
            match session.mixer.selection {
                MixerSelection::Track(idx) => {
                    if let Some(instrument) = instruments.tracks.get_mut(idx) {
                        instrument.channel_strip.level =
                            (instrument.channel_strip.level + delta).clamp(0.0, 1.0);
                    }
                }
                MixerSelection::Group(group_id) => {
                    if let Some(gm) = session.mixer.layer_group_mixer_mut(group_id) {
                        gm.channel_strip.level = (gm.channel_strip.level + delta).clamp(0.0, 1.0);
                    }
                }
                MixerSelection::Bus(id) => {
                    if let Some(bus) = session.bus_mut(id) {
                        bus.channel_strip.level = (bus.channel_strip.level + delta).clamp(0.0, 1.0);
                    }
                }
                MixerSelection::Master => {
                    session.mixer.master_level =
                        (session.mixer.master_level + delta).clamp(0.0, 1.0);
                }
            }
            true
        }
        MixerAction::ToggleMute => {
            match session.mixer.selection {
                MixerSelection::Track(idx) => {
                    if let Some(instrument) = instruments.tracks.get_mut(idx) {
                        instrument.channel_strip.mute = !instrument.channel_strip.mute;
                    }
                }
                MixerSelection::Group(group_id) => {
                    if let Some(gm) = session.mixer.layer_group_mixer_mut(group_id) {
                        gm.channel_strip.mute = !gm.channel_strip.mute;
                    }
                }
                MixerSelection::Bus(id) => {
                    if let Some(bus) = session.bus_mut(id) {
                        bus.channel_strip.mute = !bus.channel_strip.mute;
                    }
                }
                MixerSelection::Master => {
                    session.mixer.master_mute = !session.mixer.master_mute;
                }
            }
            true
        }
        MixerAction::ToggleSolo => {
            match session.mixer.selection {
                MixerSelection::Track(idx) => {
                    if let Some(instrument) = instruments.tracks.get_mut(idx) {
                        instrument.channel_strip.solo = !instrument.channel_strip.solo;
                    }
                }
                MixerSelection::Group(group_id) => {
                    if let Some(gm) = session.mixer.layer_group_mixer_mut(group_id) {
                        gm.channel_strip.solo = !gm.channel_strip.solo;
                    }
                }
                MixerSelection::Bus(id) => {
                    if let Some(bus) = session.bus_mut(id) {
                        bus.channel_strip.solo = !bus.channel_strip.solo;
                    }
                }
                MixerSelection::Master => {}
            }
            true
        }
        MixerAction::NextSection => {
            session.mixer.cycle_section();
            if let MixerSelection::Track(_) = session.mixer.selection {
                if let Some(idx) = instruments.selected {
                    session.mixer.selection = MixerSelection::Track(idx);
                }
            }
            true
        }
        MixerAction::NextOutput => {
            let bus_ids: Vec<BusId> = session.bus_ids().collect();
            match session.mixer.selection {
                MixerSelection::Track(idx) => {
                    if let Some(inst) = instruments.tracks.get_mut(idx) {
                        inst.channel_strip.output_target = match inst.channel_strip.output_target {
                            OutputTarget::Master => bus_ids
                                .first()
                                .map(|&id| OutputTarget::Bus(id))
                                .unwrap_or(OutputTarget::Master),
                            OutputTarget::Bus(current_id) => {
                                let pos = bus_ids.iter().position(|&id| id == current_id);
                                match pos {
                                    Some(p) if p + 1 < bus_ids.len() => {
                                        OutputTarget::Bus(bus_ids[p + 1])
                                    }
                                    _ => OutputTarget::Master,
                                }
                            }
                        };
                    }
                }
                MixerSelection::Group(group_id) => {
                    if let Some(gm) = session.mixer.layer_group_mixer_mut(group_id) {
                        gm.channel_strip.output_target = match gm.channel_strip.output_target {
                            OutputTarget::Master => bus_ids
                                .first()
                                .map(|&id| OutputTarget::Bus(id))
                                .unwrap_or(OutputTarget::Master),
                            OutputTarget::Bus(current_id) => {
                                let pos = bus_ids.iter().position(|&id| id == current_id);
                                match pos {
                                    Some(p) if p + 1 < bus_ids.len() => {
                                        OutputTarget::Bus(bus_ids[p + 1])
                                    }
                                    _ => OutputTarget::Master,
                                }
                            }
                        };
                    }
                }
                _ => {}
            }
            true
        }
        MixerAction::PrevOutput => {
            let bus_ids: Vec<BusId> = session.bus_ids().collect();
            match session.mixer.selection {
                MixerSelection::Track(idx) => {
                    if let Some(inst) = instruments.tracks.get_mut(idx) {
                        inst.channel_strip.output_target = match inst.channel_strip.output_target {
                            OutputTarget::Master => bus_ids
                                .last()
                                .map(|&id| OutputTarget::Bus(id))
                                .unwrap_or(OutputTarget::Master),
                            OutputTarget::Bus(current_id) => {
                                let pos = bus_ids.iter().position(|&id| id == current_id);
                                match pos {
                                    Some(0) | None => OutputTarget::Master,
                                    Some(p) => OutputTarget::Bus(bus_ids[p - 1]),
                                }
                            }
                        };
                    }
                }
                MixerSelection::Group(group_id) => {
                    if let Some(gm) = session.mixer.layer_group_mixer_mut(group_id) {
                        gm.channel_strip.output_target = match gm.channel_strip.output_target {
                            OutputTarget::Master => bus_ids
                                .last()
                                .map(|&id| OutputTarget::Bus(id))
                                .unwrap_or(OutputTarget::Master),
                            OutputTarget::Bus(current_id) => {
                                let pos = bus_ids.iter().position(|&id| id == current_id);
                                match pos {
                                    Some(0) | None => OutputTarget::Master,
                                    Some(p) => OutputTarget::Bus(bus_ids[p - 1]),
                                }
                            }
                        };
                    }
                }
                _ => {}
            }
            true
        }
        MixerAction::AdjustSend(bus_id, delta) => {
            match session.mixer.selection {
                MixerSelection::Track(idx) => {
                    if let Some(instrument) = instruments.tracks.get_mut(idx) {
                        if let Some(send) = instrument.channel_strip.sends.get_mut(bus_id) {
                            send.level = (send.level + delta).clamp(0.0, 1.0);
                        }
                    }
                }
                MixerSelection::Group(group_id) => {
                    if let Some(gm) = session.mixer.layer_group_mixer_mut(group_id) {
                        if let Some(send) = gm.channel_strip.sends.get_mut(bus_id) {
                            send.level = (send.level + delta).clamp(0.0, 1.0);
                        }
                    }
                }
                _ => {}
            }
            true
        }
        MixerAction::ToggleSend(bus_id) => {
            match session.mixer.selection {
                MixerSelection::Track(idx) => {
                    if let Some(instrument) = instruments.tracks.get_mut(idx) {
                        let send = instrument
                            .channel_strip
                            .sends
                            .entry(*bus_id)
                            .or_insert_with(|| MixerSend::new(*bus_id));
                        send.enabled = !send.enabled;
                        if send.enabled && send.level <= 0.0 {
                            send.level = 0.5;
                        }
                    }
                }
                MixerSelection::Group(group_id) => {
                    if let Some(gm) = session.mixer.layer_group_mixer_mut(group_id) {
                        let send = gm
                            .channel_strip
                            .sends
                            .entry(*bus_id)
                            .or_insert_with(|| MixerSend::new(*bus_id));
                        send.enabled = !send.enabled;
                        if send.enabled && send.level <= 0.0 {
                            send.level = 0.5;
                        }
                    }
                }
                _ => {}
            }
            true
        }
        MixerAction::NextSendTapPoint(bus_id) => {
            match session.mixer.selection {
                MixerSelection::Track(idx) => {
                    if let Some(instrument) = instruments.tracks.get_mut(idx) {
                        if let Some(send) = instrument.channel_strip.sends.get_mut(bus_id) {
                            send.tap_point = send.tap_point.cycle();
                        }
                    }
                }
                MixerSelection::Group(group_id) => {
                    if let Some(gm) = session.mixer.layer_group_mixer_mut(group_id) {
                        if let Some(send) = gm.channel_strip.sends.get_mut(bus_id) {
                            send.tap_point = send.tap_point.cycle();
                        }
                    }
                }
                _ => {}
            }
            true
        }
        MixerAction::AdjustPan(delta) => {
            match session.mixer.selection {
                MixerSelection::Track(idx) => {
                    if let Some(instrument) = instruments.tracks.get_mut(idx) {
                        instrument.channel_strip.pan =
                            (instrument.channel_strip.pan + delta).clamp(-1.0, 1.0);
                    }
                }
                MixerSelection::Group(group_id) => {
                    if let Some(gm) = session.mixer.layer_group_mixer_mut(group_id) {
                        gm.channel_strip.pan = (gm.channel_strip.pan + delta).clamp(-1.0, 1.0);
                    }
                }
                MixerSelection::Bus(id) => {
                    if let Some(bus) = session.bus_mut(id) {
                        bus.channel_strip.pan = (bus.channel_strip.pan + delta).clamp(-1.0, 1.0);
                    }
                }
                MixerSelection::Master => {}
            }
            true
        }
    }
}
