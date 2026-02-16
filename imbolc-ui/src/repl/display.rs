use crate::state::AppState;
use imbolc_types::*;

pub fn format_instrument_list(state: &AppState) -> String {
    if state.instruments.instruments.is_empty() {
        return "No instruments. Use 'instrument add <source>' to create one.".to_string();
    }

    let selected = state.instruments.selected;
    let mut lines = Vec::new();
    for (i, inst) in state.instruments.instruments.iter().enumerate() {
        let marker = if Some(i) == selected { '*' } else { ' ' };
        let mute_str = if inst.mixer.mute {
            " muted"
        } else if inst.mixer.solo {
            " solo"
        } else {
            ""
        };
        let pan_str = format_pan(inst.mixer.pan);
        lines.push(format!(
            "{}{:>2}. {:<20} [{:<12}] vol:{:.2}  pan:{}{}",
            marker,
            i + 1,
            inst.name,
            inst.source.short_name(),
            inst.mixer.level,
            pan_str,
            mute_str,
        ));
    }
    lines.join("\n")
}

pub fn format_instrument_detail(state: &AppState, id: InstrumentId) -> String {
    let inst = match state.instruments.instrument(id) {
        Some(inst) => inst,
        None => return format!("Instrument {} not found", id),
    };

    let mut lines = vec![
        format!("Instrument: {} (id: {})", inst.name, id),
        format!("  Source: {} ({})", inst.source.name(), inst.source.short_name()),
        format!("  Level: {:.2}  Pan: {}  Active: {}", inst.mixer.level, format_pan(inst.mixer.pan), inst.mixer.active),
        format!("  Mute: {}  Solo: {}", inst.mixer.mute, inst.mixer.solo),
    ];

    // Filter
    if let Some(filter) = inst.filter() {
        lines.push(format!("  Filter: {} ({})", filter.filter_type.name(), if filter.enabled { "on" } else { "off" }));
        lines.push(format!("    Cutoff: {:.1}  Resonance: {:.2}", filter.cutoff.value, filter.resonance.value));
    } else {
        lines.push("  Filter: none".to_string());
    }

    // Envelope
    let env = &inst.modulation.amp_envelope;
    lines.push(format!(
        "  Envelope: A:{:.3} D:{:.3} S:{:.2} R:{:.3}",
        env.attack, env.decay, env.sustain, env.release,
    ));

    // LFO
    let lfo = &inst.modulation.lfo;
    lines.push(format!(
        "  LFO: {} ({}, rate:{:.1}, depth:{:.2}, target:{})",
        if lfo.enabled { "on" } else { "off" },
        lfo.shape.name(),
        lfo.rate,
        lfo.depth,
        lfo.target.short_name(),
    ));

    // Effects
    let effects: Vec<_> = inst.effects().collect();
    if effects.is_empty() {
        lines.push("  Effects: none".to_string());
    } else {
        lines.push("  Effects:".to_string());
        for slot in effects {
            let bypass = if slot.enabled { "" } else { " [bypassed]" };
            lines.push(format!("    - {} (id:{}){}",
                slot.effect_type.name(), slot.id, bypass));
        }
    }

    // Arpeggiator
    let arp = &inst.note_input.arpeggiator;
    if arp.enabled {
        lines.push(format!("  Arpeggiator: on (dir:{}, rate:{}, oct:{}, gate:{:.2})",
            arp.direction.name(), arp.rate.name(), arp.octaves, arp.gate));
    }

    // Chord shape
    if let Some(shape) = &inst.note_input.chord_shape {
        lines.push(format!("  Chord: {}", shape.name()));
    }

    lines.join("\n")
}

pub fn format_transport(state: &AppState) -> String {
    let playing = state.audio.playing;
    let bpm = state.session.bpm;
    let key = state.session.key.name();
    let scale = state.session.scale.name();
    let (num, den) = state.session.time_signature;
    let tuning = state.session.tuning.short_name();
    let playhead = state.audio.playhead;

    format!(
        "Transport: {}  BPM: {}  Key: {} {}  Time: {}/{}  Tuning: {}  Playhead: {} ticks",
        if playing { "PLAYING" } else { "STOPPED" },
        bpm, key, scale, num, den, tuning, playhead,
    )
}

pub fn format_mixer(state: &AppState) -> String {
    let mut lines = vec!["Mixer:".to_string()];

    // Instruments
    for (i, inst) in state.instruments.instruments.iter().enumerate() {
        let mute = if inst.mixer.mute { " M" } else { "" };
        let solo = if inst.mixer.solo { " S" } else { "" };
        lines.push(format!(
            "  [{:>2}] {:<16} vol:{:.2}  pan:{}{}{}",
            i, inst.name, inst.mixer.level, format_pan(inst.mixer.pan), mute, solo,
        ));
    }

    // Buses
    for bus in &state.session.mixer.buses {
        let mute = if bus.mute { " M" } else { "" };
        let solo = if bus.solo { " S" } else { "" };
        lines.push(format!(
            "  [B{}] {:<16} vol:{:.2}  pan:{}{}{}",
            bus.id, bus.name, bus.level, format_pan(bus.pan), mute, solo,
        ));
    }

    // Master
    let master_mute = if state.session.mixer.master_mute { " M" } else { "" };
    lines.push(format!(
        "  [MST] Master            vol:{:.2}{}",
        state.session.mixer.master_level, master_mute,
    ));

    lines.join("\n")
}

pub fn format_notes(state: &AppState, track_idx: usize) -> String {
    let track_order = &state.session.piano_roll.track_order;
    if track_idx >= track_order.len() {
        return format!("Track index {} out of range (0..{})", track_idx, track_order.len());
    }
    let inst_id = track_order[track_idx];
    let track = match state.session.piano_roll.tracks.get(&inst_id) {
        Some(t) => t,
        None => return format!("No track for instrument {}", inst_id),
    };

    if track.notes.is_empty() {
        return format!("Track {} (instrument {}): no notes", track_idx, inst_id);
    }

    let mut lines = vec![format!("Track {} (instrument {}):", track_idx, inst_id)];
    for note in &track.notes {
        lines.push(format!(
            "  pitch:{:>3}  tick:{:>6}  dur:{:>5}  vel:{:>3}",
            note.pitch, note.tick, note.duration, note.velocity,
        ));
    }
    lines.join("\n")
}

pub fn format_effects(state: &AppState, id: InstrumentId) -> String {
    let inst = match state.instruments.instrument(id) {
        Some(inst) => inst,
        None => return format!("Instrument {} not found", id),
    };

    let effects: Vec<_> = inst.effects().collect();
    if effects.is_empty() {
        return format!("Instrument {} ({}) has no effects", id, inst.name);
    }

    let mut lines = vec![format!("Effects for {} (id: {}):", inst.name, id)];
    for slot in effects {
        let bypass = if slot.enabled { "on" } else { "bypassed" };
        lines.push(format!("  [{}] {} ({})", slot.id, slot.effect_type.name(), bypass));
        for (i, param) in slot.params.iter().enumerate() {
            lines.push(format!("    {}: {} = {:?}", i, param.name, param.value));
        }
    }
    lines.join("\n")
}

pub fn format_buses(state: &AppState) -> String {
    if state.session.mixer.buses.is_empty() {
        return "No buses".to_string();
    }

    let mut lines = vec!["Buses:".to_string()];
    for bus in &state.session.mixer.buses {
        let mute = if bus.mute { " M" } else { "" };
        lines.push(format!(
            "  [{}] {} vol:{:.2} pan:{}{}",
            bus.id, bus.name, bus.level, format_pan(bus.pan), mute,
        ));
        for fx in &bus.effect_chain.effects {
            let bypass = if fx.enabled { "" } else { " [bypassed]" };
            lines.push(format!("    - {} (id:{}){}",
                fx.effect_type.name(), fx.id, bypass));
        }
    }
    lines.join("\n")
}

pub fn format_arrangement(state: &AppState) -> String {
    let arr = &state.session.arrangement;
    let mut lines = vec![format!("Arrangement ({} clips, {} placements):", arr.clips.len(), arr.placements.len())];

    if !arr.clips.is_empty() {
        lines.push("  Clips:".to_string());
        for clip in &arr.clips {
            lines.push(format!("    [{}] {} (inst:{}, len:{})", clip.id, clip.name, clip.instrument_id, clip.length_ticks));
        }
    }

    if !arr.placements.is_empty() {
        lines.push("  Placements:".to_string());
        for p in &arr.placements {
            lines.push(format!("    [{}] clip:{} inst:{} start:{}", p.id, p.clip_id, p.instrument_id, p.start_tick));
        }
    }

    lines.join("\n")
}

pub fn format_automation(state: &AppState) -> String {
    let auto = &state.session.automation;
    if auto.lanes.is_empty() {
        return "No automation lanes".to_string();
    }

    let mut lines = vec![format!("Automation ({} lanes):", auto.lanes.len())];
    for lane in &auto.lanes {
        let armed = if lane.record_armed { " [armed]" } else { "" };
        let enabled = if lane.enabled { "" } else { " [disabled]" };
        lines.push(format!("  [{}] {}{}{} ({} points)",
            lane.id, lane.target.short_name(), armed, enabled, lane.points.len()));
    }
    lines.join("\n")
}

pub fn format_generative(state: &AppState) -> String {
    let gen = &state.session.generative;
    let mut lines = vec![
        format!("Generative Engine: {}", if gen.enabled { "on" } else { "off" }),
        format!("  Capture: {}  Scale Lock: {}", gen.capture_enabled, gen.constraints.scale_lock),
        format!("  Macros: density:{:.2} chaos:{:.2} energy:{:.2} motion:{:.2}",
            gen.macros.density, gen.macros.chaos, gen.macros.energy, gen.macros.motion),
        format!("  Pitch range: {}..{}", gen.constraints.pitch_min, gen.constraints.pitch_max),
    ];

    if gen.voices.is_empty() {
        lines.push("  Voices: none".to_string());
    } else {
        lines.push(format!("  Voices ({}):", gen.voices.len()));
        for voice in &gen.voices {
            let target = voice.target_instrument
                .map(|id| format!("inst:{}", id))
                .unwrap_or_else(|| "unassigned".to_string());
            lines.push(format!("    [{}] {} ({}) -> {}",
                voice.id, voice.algorithm.name(),
                if voice.enabled { "on" } else { "off" },
                target));
        }
    }

    lines.join("\n")
}

pub fn format_session(state: &AppState) -> String {
    let s = &state.session;
    let mut lines = vec![
        format!("Session:"),
        format!("  BPM: {}  Key: {} {}  Time: {}/{}",
            s.bpm, s.key.name(), s.scale.name(), s.time_signature.0, s.time_signature.1),
        format!("  Tuning: {} (A4={:.1}Hz)  JI: {}",
            s.tuning.name(), s.tuning_a4, s.ji_flavor.name()),
        format!("  Snap: {}  Humanize: vel:{:.2} time:{:.2}",
            s.snap, s.humanize.velocity, s.humanize.timing),
    ];

    if let Some(path) = &state.project.path {
        lines.push(format!("  Project: {}", path.display()));
    }
    lines.push(format!("  Dirty: {}", state.project.dirty));

    lines.join("\n")
}

pub fn format_sequencer(state: &AppState) -> String {
    let selected = state.instruments.selected;
    let inst = selected.and_then(|i| state.instruments.instruments.get(i));
    let inst = match inst {
        Some(i) => i,
        None => return "No instrument selected".to_string(),
    };

    let seq = match &inst.source_extra {
        imbolc_types::state::instrument::SourceExtra::Kit(seq) => seq,
        _ => return "No drum sequencer on selected instrument".to_string(),
    };

    let mut lines = vec![
        format!("Drum Sequencer for {} (pattern {}/{}):",
            inst.name, seq.current_pattern + 1, seq.patterns.len()),
    ];
    if let Some(pattern) = seq.patterns.get(seq.current_pattern) {
        for (pad_idx, pad_steps) in pattern.steps.iter().enumerate() {
            let steps_str: String = pad_steps.iter()
                .map(|s| if s.active { 'x' } else { '.' })
                .collect();
            lines.push(format!("  [{:>2}] {}", pad_idx, steps_str));
        }
    }
    lines.join("\n")
}

pub fn format_server(state: &AppState) -> String {
    format!("Server: {:?}", state.audio.server_status)
}

pub fn format_midi(state: &AppState) -> String {
    let midi = &state.midi;
    let mut lines = vec!["MIDI:".to_string()];

    if let Some(port) = &midi.connected_port {
        lines.push(format!("  Connected: {}", port));
    } else {
        lines.push("  Not connected".to_string());
    }

    if !midi.port_names.is_empty() {
        lines.push("  Available ports:".to_string());
        for (i, name) in midi.port_names.iter().enumerate() {
            lines.push(format!("    [{}] {}", i, name));
        }
    }

    lines.join("\n")
}

fn format_pan(pan: f32) -> String {
    if pan.abs() < 0.01 {
        "C".to_string()
    } else if pan < 0.0 {
        format!("L{}", (pan.abs() * 100.0) as i32)
    } else {
        format!("R{}", (pan * 100.0) as i32)
    }
}
