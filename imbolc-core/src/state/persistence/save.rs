use rusqlite::{params, Connection, Result as SqlResult};

use crate::state::session::SessionState;
use crate::state::track_state::TrackState;

use super::schema;

/// Save project state to relational tables. Performs DELETE-all + INSERT-current atomically.
pub fn save_relational(
    conn: &Connection,
    session: &SessionState,
    tracks: &TrackState,
) -> SqlResult<()> {
    schema::delete_all_data(conn)?;

    save_session(conn, session, tracks)?;
    save_theme(conn, session)?;
    save_instruments(conn, tracks)?;
    save_mixer(conn, session)?;
    save_layer_group_mixers(conn, session)?;
    save_musical_settings(conn, session)?;
    save_piano_roll(conn, session)?;
    save_automation(conn, session)?;
    save_custom_synthdefs(conn, session)?;
    save_vst_plugins(conn, session)?;
    save_midi_recording(conn, session)?;
    save_param_tags(conn, session)?;
    save_arrangement(conn, session)?;

    Ok(())
}

// ============================================================
// Session
// ============================================================

fn save_session(conn: &Connection, session: &SessionState, tracks: &TrackState) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO session (id, bpm, time_sig_num, time_sig_denom, key, scale, tuning_a4, snap,
            next_track_id, next_sampler_buffer_id, selected_track, next_layer_group_id,
            humanize_velocity, humanize_timing,
            click_enabled, click_volume, click_muted,
            tuning, ji_flavor, next_sample_id)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
        params![
            session.bpm,
            session.time_signature.0,
            session.time_signature.1,
            format!("{:?}", session.key),
            format!("{:?}", session.scale),
            session.tuning_a4,
            session.snap as i32,
            tracks.next_id.get(),
            tracks.next_sampler_buffer_id.get(),
            tracks.selected.map(|s| s as i64),
            tracks.next_layer_group_id.get(),
            session.humanize.velocity,
            session.humanize.timing,
            session.click_track.enabled as i32,
            session.click_track.volume,
            session.click_track.muted as i32,
            format!("{:?}", session.tuning),
            format!("{:?}", session.ji_flavor),
            tracks.next_sample_id.get(),
        ],
    )?;
    Ok(())
}

fn save_theme(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    let t = &session.theme;
    conn.execute(
        "INSERT INTO theme (id, name,
            background_r, background_g, background_b,
            foreground_r, foreground_g, foreground_b,
            border_r, border_g, border_b,
            selection_bg_r, selection_bg_g, selection_bg_b,
            selection_fg_r, selection_fg_g, selection_fg_b,
            muted_r, muted_g, muted_b,
            error_r, error_g, error_b,
            warning_r, warning_g, warning_b,
            success_r, success_g, success_b,
            osc_color_r, osc_color_g, osc_color_b,
            filter_color_r, filter_color_g, filter_color_b,
            env_color_r, env_color_g, env_color_b,
            lfo_color_r, lfo_color_g, lfo_color_b,
            fx_color_r, fx_color_g, fx_color_b,
            sample_color_r, sample_color_g, sample_color_b,
            midi_color_r, midi_color_g, midi_color_b,
            audio_in_color_r, audio_in_color_g, audio_in_color_b,
            meter_low_r, meter_low_g, meter_low_b,
            meter_mid_r, meter_mid_g, meter_mid_b,
            meter_high_r, meter_high_g, meter_high_b,
            waveform_grad_0_r, waveform_grad_0_g, waveform_grad_0_b,
            waveform_grad_1_r, waveform_grad_1_g, waveform_grad_1_b,
            waveform_grad_2_r, waveform_grad_2_g, waveform_grad_2_b,
            waveform_grad_3_r, waveform_grad_3_g, waveform_grad_3_b,
            playing_r, playing_g, playing_b,
            recording_r, recording_g, recording_b,
            armed_r, armed_g, armed_b)
         VALUES (1, ?1,
            ?2,?3,?4, ?5,?6,?7, ?8,?9,?10, ?11,?12,?13, ?14,?15,?16,
            ?17,?18,?19, ?20,?21,?22, ?23,?24,?25, ?26,?27,?28,
            ?29,?30,?31, ?32,?33,?34, ?35,?36,?37, ?38,?39,?40,
            ?41,?42,?43, ?44,?45,?46, ?47,?48,?49, ?50,?51,?52,
            ?53,?54,?55, ?56,?57,?58, ?59,?60,?61, ?62,?63,?64,
            ?65,?66,?67, ?68,?69,?70, ?71,?72,?73, ?74,?75,?76,
            ?77,?78,?79, ?80,?81,?82)",
        params![
            t.name,
            t.background.r,
            t.background.g,
            t.background.b,
            t.foreground.r,
            t.foreground.g,
            t.foreground.b,
            t.border.r,
            t.border.g,
            t.border.b,
            t.selection_bg.r,
            t.selection_bg.g,
            t.selection_bg.b,
            t.selection_fg.r,
            t.selection_fg.g,
            t.selection_fg.b,
            t.muted.r,
            t.muted.g,
            t.muted.b,
            t.error.r,
            t.error.g,
            t.error.b,
            t.warning.r,
            t.warning.g,
            t.warning.b,
            t.success.r,
            t.success.g,
            t.success.b,
            t.osc_color.r,
            t.osc_color.g,
            t.osc_color.b,
            t.filter_color.r,
            t.filter_color.g,
            t.filter_color.b,
            t.env_color.r,
            t.env_color.g,
            t.env_color.b,
            t.lfo_color.r,
            t.lfo_color.g,
            t.lfo_color.b,
            t.fx_color.r,
            t.fx_color.g,
            t.fx_color.b,
            t.sample_color.r,
            t.sample_color.g,
            t.sample_color.b,
            t.midi_color.r,
            t.midi_color.g,
            t.midi_color.b,
            t.audio_in_color.r,
            t.audio_in_color.g,
            t.audio_in_color.b,
            t.meter_low.r,
            t.meter_low.g,
            t.meter_low.b,
            t.meter_mid.r,
            t.meter_mid.g,
            t.meter_mid.b,
            t.meter_high.r,
            t.meter_high.g,
            t.meter_high.b,
            t.waveform_gradient[0].r,
            t.waveform_gradient[0].g,
            t.waveform_gradient[0].b,
            t.waveform_gradient[1].r,
            t.waveform_gradient[1].g,
            t.waveform_gradient[1].b,
            t.waveform_gradient[2].r,
            t.waveform_gradient[2].g,
            t.waveform_gradient[2].b,
            t.waveform_gradient[3].r,
            t.waveform_gradient[3].g,
            t.waveform_gradient[3].b,
            t.playing.r,
            t.playing.g,
            t.playing.b,
            t.recording.r,
            t.recording.g,
            t.recording.b,
            t.armed.r,
            t.armed.g,
            t.armed.b,
        ],
    )?;
    Ok(())
}

// ============================================================
// Instruments
// ============================================================

fn save_instruments(conn: &Connection, tracks: &TrackState) -> SqlResult<()> {
    let mut inst_stmt = conn.prepare(
        "INSERT INTO tracks (
            id, name, position, source_type,
            filter_type, filter_cutoff, filter_cutoff_min, filter_cutoff_max,
            filter_resonance, filter_resonance_min, filter_resonance_max,
            filter_enabled,
            lfo_enabled, lfo_rate, lfo_depth, lfo_shape, lfo_target,
            amp_attack, amp_decay, amp_sustain, amp_release,
            polyphonic, level, pan, mute, solo, active,
            output_target, channel_config, convolution_ir_sample_id, layer_group,
            next_effect_id, eq_enabled,
            arp_enabled, arp_direction, arp_rate, arp_octaves, arp_gate,
            legato_enabled, glide_rate,
            chord_shape, vst_state_path,
            groove_swing_amount, groove_swing_grid,
            groove_humanize_velocity, groove_humanize_timing,
            groove_timing_offset_ms, groove_time_sig_num, groove_time_sig_denom,
            layer_octave_offset)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33,?34,?35,?36,?37,?38,?39,?40,?41,?42,?43,?44,?45,?46,?47,?48,?49,?50)",
    )?;

    for (pos, inst) in tracks.tracks.iter().enumerate() {
        let source_type = encode_source_type(&inst.source);
        let output_target = encode_output_target(&inst.channel_strip.output_target);
        let channel_config = format!("{:?}", inst.channel_strip.channel_config);

        let (
            filter_type,
            filter_cutoff,
            filter_cutoff_min,
            filter_cutoff_max,
            filter_resonance,
            filter_resonance_min,
            filter_resonance_max,
            filter_enabled,
        ) = if let Some(f) = inst.filter() {
            (
                Some(format!("{:?}", f.filter_type)),
                Some(f.cutoff.value),
                Some(f.cutoff.min),
                Some(f.cutoff.max),
                Some(f.resonance.value),
                Some(f.resonance.min),
                Some(f.resonance.max),
                f.enabled as i32,
            )
        } else {
            (None, None, None, None, None, None, None, 1)
        };

        let eq_enabled = inst.eq().map(|eq| eq.enabled as i32);

        let chord_shape = inst
            .note_input
            .chord_shape
            .as_ref()
            .map(|cs| format!("{:?}", cs));

        let vst_state = inst
            .vst_source_state_path()
            .map(|p| p.to_string_lossy().to_string());

        let groove = &inst.groove;
        let groove_time_sig_num = groove.time_signature.map(|(n, _)| n as i32);
        let groove_time_sig_denom = groove.time_signature.map(|(_, d)| d as i32);
        let groove_swing_grid = groove.swing_grid.as_ref().map(|g| format!("{:?}", g));

        inst_stmt.execute(params![
            inst.id.get(),
            inst.name,
            pos as i32,
            source_type,
            filter_type,
            filter_cutoff,
            filter_cutoff_min,
            filter_cutoff_max,
            filter_resonance,
            filter_resonance_min,
            filter_resonance_max,
            filter_enabled,
            inst.modulation.lfo.enabled as i32,
            inst.modulation.lfo.rate,
            inst.modulation.lfo.depth,
            format!("{:?}", inst.modulation.lfo.shape),
            encode_parameter_target(&inst.modulation.lfo.target),
            inst.modulation.amp_envelope.attack,
            inst.modulation.amp_envelope.decay,
            inst.modulation.amp_envelope.sustain,
            inst.modulation.amp_envelope.release,
            inst.polyphonic as i32,
            inst.channel_strip.level,
            inst.channel_strip.pan,
            inst.channel_strip.mute as i32,
            inst.channel_strip.solo as i32,
            inst.channel_strip.active as i32,
            output_target,
            channel_config,
            inst.convolution_ir_sample
                .as_ref()
                .map(|sr| sr.id.get() as i64),
            inst.layer.group.map(|g| g.get()),
            inst.channel_strip.next_effect_id.get(),
            eq_enabled,
            inst.note_input.arpeggiator.enabled as i32,
            format!("{:?}", inst.note_input.arpeggiator.direction),
            format!("{:?}", inst.note_input.arpeggiator.rate),
            inst.note_input.arpeggiator.octaves,
            inst.note_input.arpeggiator.gate,
            0_i32,       // legato_enabled (legacy, now in processing chain)
            "Sixteenth", // glide_rate (legacy, now in processing chain)
            chord_shape,
            vst_state,
            groove.swing_amount,
            groove_swing_grid,
            groove.humanize_velocity,
            groove.humanize_timing,
            groove.timing_offset_ms,
            groove_time_sig_num,
            groove_time_sig_denom,
            inst.layer.octave_offset as i32,
        ])?;

        // Source params
        save_params(
            conn,
            "track_source_params",
            "track_id",
            inst.id.get(),
            &inst.source_params,
        )?;

        // Effects
        let effects: Vec<_> = inst.effects().cloned().collect();
        save_effects(conn, inst.id.get(), &effects)?;

        // Note effects
        let note_effects: Vec<_> = inst.note_effects().cloned().collect();
        save_note_effects(conn, inst.id.get(), &note_effects)?;

        // Sends
        for send in inst.channel_strip.sends.values() {
            conn.execute(
                "INSERT INTO track_sends (track_id, bus_id, level, enabled, tap_point)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    inst.id.get(),
                    send.bus_id.get() as i32,
                    send.level,
                    send.enabled as i32,
                    encode_tap_point(send.tap_point)
                ],
            )?;
        }

        // Filter modulations
        if let Some(f) = inst.filter() {
            save_modulation(conn, inst.id.get(), "cutoff", &f.cutoff.mod_source)?;
            save_modulation(conn, inst.id.get(), "resonance", &f.resonance.mod_source)?;

            // Filter extra params
            save_params(
                conn,
                "track_filter_extra_params",
                "track_id",
                inst.id.get(),
                &f.extra_params,
            )?;
        }

        // EQ bands
        if let Some(eq) = inst.eq() {
            for (i, band) in eq.bands.iter().enumerate() {
                conn.execute(
                    "INSERT INTO track_eq_bands (track_id, band_index, band_type, freq, gain, q, enabled)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        inst.id.get(), i as i32, format!("{:?}", band.band_type),
                        band.freq, band.gain, band.q, band.enabled as i32,
                    ],
                )?;
            }
        }

        // Processing chain order
        save_processing_chain(conn, inst.id.get(), &inst.channel_strip.processing_chain)?;

        // VST param values
        for (param_idx, value) in inst.vst_source_params() {
            conn.execute(
                "INSERT INTO track_vst_params (track_id, param_index, value)
                 VALUES (?1, ?2, ?3)",
                params![inst.id.get(), param_idx, value],
            )?;
        }

        // Sampler config
        if let Some(config) = inst.sampler_config() {
            save_sampler_config(conn, inst.id.get(), config)?;
        }

        // Drum sequencer
        if let Some(seq) = inst.drum_sequencer() {
            save_drum_sequencer(conn, inst.id.get(), seq)?;
        }
    }

    Ok(())
}

fn save_params(
    conn: &Connection,
    table: &str,
    id_col: &str,
    id: u32,
    params: &[crate::state::param::Param],
) -> SqlResult<()> {
    let sql = format!(
        "INSERT INTO {} ({}, position, param_name, param_value_type, param_value_float, param_value_int, param_value_bool, param_min, param_max)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        table, id_col
    );
    let mut stmt = conn.prepare(&sql)?;

    for (pos, p) in params.iter().enumerate() {
        let (vtype, vf, vi, vb) = encode_param_value(&p.value);
        stmt.execute(params![
            id, pos as i32, p.name, vtype, vf, vi, vb, p.min, p.max
        ])?;
    }
    Ok(())
}

fn save_effects(
    conn: &Connection,
    track_id: u32,
    effects: &[crate::state::track::EffectSlot],
) -> SqlResult<()> {
    save_effects_to(
        conn,
        "track_effects",
        "track_effect_params",
        "effect_vst_params",
        "track_id",
        track_id,
        effects,
    )
}

fn save_effects_to(
    conn: &Connection,
    effects_table: &str,
    params_table: &str,
    vst_table: &str,
    owner_col: &str,
    owner_id: u32,
    effects: &[crate::state::track::EffectSlot],
) -> SqlResult<()> {
    for (pos, effect) in effects.iter().enumerate() {
        let effect_type = encode_effect_type(&effect.effect_type);
        let vst_state = effect
            .vst_state_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        let sql = format!(
            "INSERT INTO {} ({}, effect_id, position, effect_type, enabled, vst_state_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            effects_table, owner_col
        );
        conn.execute(
            &sql,
            params![
                owner_id,
                effect.id.get(),
                pos as i32,
                effect_type,
                effect.enabled as i32,
                vst_state
            ],
        )?;

        // Effect params
        let param_sql = format!(
            "INSERT INTO {} ({}, effect_id, position, param_name, param_value_type, param_value_float, param_value_int, param_value_bool, param_min, param_max)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params_table, owner_col
        );
        let mut stmt = conn.prepare(&param_sql)?;
        for (ppos, p) in effect.params.iter().enumerate() {
            let (vtype, vf, vi, vb) = encode_param_value(&p.value);
            stmt.execute(params![
                owner_id,
                effect.id.get(),
                ppos as i32,
                p.name,
                vtype,
                vf,
                vi,
                vb,
                p.min,
                p.max
            ])?;
        }

        // Effect VST param values
        let vst_sql = format!(
            "INSERT INTO {} ({}, effect_id, param_index, value)
             VALUES (?1, ?2, ?3, ?4)",
            vst_table, owner_col
        );
        for (param_idx, value) in &effect.vst_param_values {
            conn.execute(
                &vst_sql,
                params![owner_id, effect.id.get(), param_idx, value],
            )?;
        }
    }
    Ok(())
}

fn save_note_effects(
    conn: &Connection,
    track_id: u32,
    note_effects: &[imbolc_types::NoteEffectSlot],
) -> SqlResult<()> {
    for (pos, ne) in note_effects.iter().enumerate() {
        conn.execute(
            "INSERT INTO track_note_effects (track_id, effect_id, position, effect_type, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                track_id,
                ne.id.get(),
                pos as i32,
                format!("{:?}", ne.effect_type),
                ne.enabled as i32,
            ],
        )?;

        // Note effect params
        save_params(
            conn,
            "track_note_effect_params",
            "effect_id",
            ne.id.get(),
            &ne.params,
        )?;
    }
    Ok(())
}

fn save_processing_chain(
    conn: &Connection,
    track_id: u32,
    chain: &[imbolc_types::ProcessingStage],
) -> SqlResult<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO track_processing_chain \
         (track_id, position, stage_type, effect_id) \
         VALUES (?1, ?2, ?3, ?4)",
    )?;
    for (pos, stage) in chain.iter().enumerate() {
        let (stage_type, effect_id): (&str, Option<u32>) = match stage {
            imbolc_types::ProcessingStage::Filter(_) => ("filter", None),
            imbolc_types::ProcessingStage::Eq(id, _) => ("eq", Some(id.get())),
            imbolc_types::ProcessingStage::Effect(e) => ("effect", Some(e.id.get())),
            imbolc_types::ProcessingStage::NoteEffect(ne) => ("note_effect", Some(ne.id.get())),
        };
        stmt.execute(params![track_id, pos as i32, stage_type, effect_id])?;
    }
    Ok(())
}

fn save_modulation(
    conn: &Connection,
    track_id: u32,
    target_param: &str,
    mod_source: &Option<crate::state::track::ModSource>,
) -> SqlResult<()> {
    use crate::state::track::ModSource;

    if let Some(ms) = mod_source {
        match ms {
            ModSource::Lfo(lfo) => {
                conn.execute(
                    "INSERT INTO track_modulations (track_id, target_param, mod_type,
                        lfo_enabled, lfo_rate, lfo_depth, lfo_shape, lfo_target)
                     VALUES (?1, ?2, 'Lfo', ?3, ?4, ?5, ?6, ?7)",
                    params![
                        track_id,
                        target_param,
                        lfo.enabled as i32,
                        lfo.rate,
                        lfo.depth,
                        format!("{:?}", lfo.shape),
                        encode_parameter_target(&lfo.target),
                    ],
                )?;
            }
            ModSource::Envelope(env) => {
                conn.execute(
                    "INSERT INTO track_modulations (track_id, target_param, mod_type,
                        env_attack, env_decay, env_sustain, env_release)
                     VALUES (?1, ?2, 'Envelope', ?3, ?4, ?5, ?6)",
                    params![
                        track_id,
                        target_param,
                        env.attack,
                        env.decay,
                        env.sustain,
                        env.release,
                    ],
                )?;
            }
            ModSource::TrackParam(src_id, param_name) => {
                conn.execute(
                    "INSERT INTO track_modulations (track_id, target_param, mod_type,
                        source_track_id, source_param_name)
                     VALUES (?1, ?2, 'TrackParam', ?3, ?4)",
                    params![track_id, target_param, src_id.get(), param_name],
                )?;
            }
        }
    }
    Ok(())
}

fn save_sampler_config(
    conn: &Connection,
    track_id: u32,
    config: &crate::state::sampler::SamplerConfig,
) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO sampler_configs (track_id, buffer_id, sample_id, loop_mode, pitch_tracking, next_slice_id, selected_slice)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            track_id,
            config.buffer_id.map(|id| id.get() as i64),
            config.sample_ref.as_ref().map(|sr| sr.id.get() as i64),
            config.loop_mode as i32,
            config.pitch_tracking as i32,
            config.next_slice_id().get(),
            config.selected_slice as i32,
        ],
    )?;

    for (pos, slice) in config.slices.iter().enumerate() {
        conn.execute(
            "INSERT INTO sampler_slices (track_id, slice_id, position, start_pos, end_pos, name, root_note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                track_id, slice.id.get(), pos as i32,
                slice.start, slice.end, slice.name, slice.root_note as i32,
            ],
        )?;
    }
    Ok(())
}

fn save_drum_sequencer(
    conn: &Connection,
    track_id: u32,
    seq: &crate::state::drum_sequencer::DrumSequencerState,
) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO drum_sequencer_state (track_id, current_pattern, next_buffer_id, swing_amount, chain_enabled, step_resolution)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            track_id,
            seq.current_pattern as i32,
            seq.next_buffer_id.get(),
            seq.swing_amount,
            seq.chain_enabled as i32,
            format!("{:?}", seq.step_resolution),
        ],
    )?;

    // Chain
    for (pos, &pattern_idx) in seq.chain.iter().enumerate() {
        conn.execute(
            "INSERT INTO drum_sequencer_chain (track_id, position, pattern_index)
             VALUES (?1, ?2, ?3)",
            params![track_id, pos as i32, pattern_idx as i32],
        )?;
    }

    // Pads
    for (pad_idx, pad) in seq.pads.iter().enumerate() {
        conn.execute(
            "INSERT INTO drum_pads (track_id, pad_index, buffer_id, sample_id, name, level, slice_start, slice_end, reverse, pitch, trigger_track_id, trigger_freq, duration_secs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                track_id, pad_idx as i32,
                pad.buffer_id.map(|id| id.get() as i64),
                pad.sample_ref.as_ref().map(|sr| sr.id.get() as i64),
                pad.name,
                pad.level,
                pad.slice_start,
                pad.slice_end,
                pad.reverse as i32,
                pad.pitch as i32,
                pad.instrument_id.map(|id| id.get() as i64),
                pad.trigger_freq,
                pad.duration_secs,
            ],
        )?;
    }

    // Patterns and steps
    for (pat_idx, pattern) in seq.patterns.iter().enumerate() {
        conn.execute(
            "INSERT INTO drum_patterns (track_id, pattern_index, length)
             VALUES (?1, ?2, ?3)",
            params![track_id, pat_idx as i32, pattern.length as i32],
        )?;

        // Only save active steps (sparse)
        for (pad_idx, pad_steps) in pattern.steps.iter().enumerate() {
            for (step_idx, step) in pad_steps.iter().enumerate() {
                if step.active {
                    conn.execute(
                        "INSERT INTO drum_steps (track_id, pattern_index, pad_index, step_index, velocity, probability, pitch_offset)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            track_id, pat_idx as i32, pad_idx as i32, step_idx as i32,
                            step.velocity as i32, step.probability, step.pitch_offset as i32,
                        ],
                    )?;
                }
            }
        }
    }

    // Chopper
    if let Some(ref chopper) = seq.chopper {
        let peaks_blob: Option<Vec<u8>> = if chopper.waveform_peaks.is_empty() {
            None
        } else {
            // Store as raw f32 bytes
            let mut bytes = Vec::with_capacity(chopper.waveform_peaks.len() * 4);
            for &peak in &chopper.waveform_peaks {
                bytes.extend_from_slice(&peak.to_le_bytes());
            }
            Some(bytes)
        };

        conn.execute(
            "INSERT INTO chopper_states (track_id, buffer_id, sample_id, selected_slice, next_slice_id, duration_secs, waveform_peaks)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                track_id,
                chopper.buffer_id.map(|id| id.get() as i64),
                chopper.sample_ref.as_ref().map(|sr| sr.id.get() as i64),
                chopper.selected_slice as i32,
                chopper.next_slice_id.get(),
                chopper.duration_secs,
                peaks_blob,
            ],
        )?;

        for (pos, slice) in chopper.slices.iter().enumerate() {
            conn.execute(
                "INSERT INTO chopper_slices (track_id, slice_id, position, start_pos, end_pos, name, root_note)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    track_id, slice.id.get(), pos as i32,
                    slice.start, slice.end, slice.name, slice.root_note as i32,
                ],
            )?;
        }
    }

    Ok(())
}

// ============================================================
// Mixer
// ============================================================

fn save_mixer(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    for bus in &session.mixer.buses {
        conn.execute(
            "INSERT INTO mixer_buses (id, name, level, pan, mute, solo)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                bus.id.get() as i32,
                bus.name,
                bus.channel_strip.level,
                bus.channel_strip.pan,
                bus.channel_strip.mute as i32,
                bus.channel_strip.solo as i32
            ],
        )?;

        let bus_effects: Vec<_> = bus.channel_strip.effects().cloned().collect();
        save_effects_to(
            conn,
            "bus_effects",
            "bus_effect_params",
            "bus_effect_vst_params",
            "bus_id",
            bus.id.get() as u32,
            &bus_effects,
        )?;
    }

    let master_eq_enabled = if session.mixer.master_eq().is_some() {
        1i32
    } else {
        0i32
    };
    conn.execute(
        "INSERT INTO mixer_master (id, level, mute, eq_enabled) VALUES (1, ?1, ?2, ?3)",
        params![
            session.mixer.master_level,
            session.mixer.master_mute as i32,
            master_eq_enabled,
        ],
    )?;

    // Save master EQ bands
    if let Some(eq) = session.mixer.master_eq() {
        for (i, band) in eq.bands.iter().enumerate() {
            conn.execute(
                "INSERT INTO master_eq_bands (band_index, freq, gain, q, enabled)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![i as i32, band.freq, band.gain, band.q, band.enabled as i32],
            )?;
        }
    }

    // Save bus EQ bands
    for bus in &session.mixer.buses {
        if let Some(eq) = bus.channel_strip.eq() {
            for (i, band) in eq.bands.iter().enumerate() {
                conn.execute(
                    "INSERT INTO bus_eq_bands (bus_id, band_index, freq, gain, q, enabled)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        bus.id.get() as i32,
                        i as i32,
                        band.freq,
                        band.gain,
                        band.q,
                        band.enabled as i32,
                    ],
                )?;
            }
        }
    }

    Ok(())
}

fn save_layer_group_mixers(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    for gm in &session.mixer.layer_group_mixers {
        let output_target = encode_output_target(&gm.channel_strip.output_target);
        let eq_enabled = if gm.eq().is_some() { 1i32 } else { 0i32 };
        conn.execute(
            "INSERT INTO layer_group_mixers (group_id, name, level, pan, mute, solo, output_target, eq_enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                gm.group_id.get() as i32, gm.name, gm.channel_strip.level, gm.channel_strip.pan,
                gm.channel_strip.mute as i32, gm.channel_strip.solo as i32, output_target, eq_enabled,
            ],
        )?;

        for send in gm.channel_strip.sends.values() {
            conn.execute(
                "INSERT INTO layer_group_sends (group_id, bus_id, level, enabled, tap_point)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    gm.group_id.get() as i32,
                    send.bus_id.get() as i32,
                    send.level,
                    send.enabled as i32,
                    encode_tap_point(send.tap_point)
                ],
            )?;
        }

        // Save EQ bands
        if let Some(eq) = gm.eq() {
            for (i, band) in eq.bands.iter().enumerate() {
                conn.execute(
                    "INSERT INTO layer_group_eq_bands (group_id, band_index, freq, gain, q, enabled)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        gm.group_id.get() as i32, i as i32,
                        band.freq, band.gain, band.q, band.enabled as i32,
                    ],
                )?;
            }
        }

        let gm_effects: Vec<_> = gm.channel_strip.effects().cloned().collect();
        save_effects_to(
            conn,
            "layer_group_effects",
            "layer_group_effect_params",
            "layer_group_effect_vst_params",
            "group_id",
            gm.group_id.get(),
            &gm_effects,
        )?;
    }
    Ok(())
}

// ============================================================
// Musical Settings / Piano Roll
// ============================================================

fn save_musical_settings(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    let pr = &session.piano_roll;
    conn.execute(
        "INSERT INTO musical_settings (id, bpm, time_sig_num, time_sig_denom, ticks_per_beat, loop_start, loop_end, looping, swing_amount)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            pr.bpm,
            pr.time_signature.0,
            pr.time_signature.1,
            pr.ticks_per_beat,
            pr.loop_start,
            pr.loop_end,
            pr.looping as i32,
            pr.swing_amount,
        ],
    )?;
    Ok(())
}

fn save_piano_roll(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    let pr = &session.piano_roll;
    let mut note_id: i64 = 0;

    for (pos, &inst_id) in pr.sequence_order.iter().enumerate() {
        if let Some(seq) = pr.sequences.get(&inst_id) {
            conn.execute(
                "INSERT INTO piano_roll_tracks (track_id, position, polyphonic)
                 VALUES (?1, ?2, ?3)",
                params![inst_id.get(), pos as i32, seq.polyphonic as i32],
            )?;

            for note in &seq.notes {
                conn.execute(
                    "INSERT INTO piano_roll_notes (id, track_track_id, tick, duration, pitch, velocity, probability)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        note_id, inst_id.get(),
                        note.tick as i64, note.duration as i64,
                        note.pitch as i32, note.velocity as i32,
                        note.probability,
                    ],
                )?;
                note_id += 1;
            }
        }
    }
    Ok(())
}

// ============================================================
// Automation
// ============================================================

fn save_automation(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    for lane in &session.automation.lanes {
        let (
            target_type,
            target_inst_id,
            target_bus_id,
            target_effect_id,
            target_param_idx,
            target_extra,
        ) = encode_automation_target(&lane.target);

        conn.execute(
            "INSERT INTO automation_lanes (id, target_type, target_track_id, target_bus_id, target_effect_id, target_param_idx, target_extra, enabled, record_armed, min_value, max_value)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                lane.id.get(), target_type, target_inst_id, target_bus_id,
                target_effect_id, target_param_idx, target_extra,
                lane.enabled as i32, lane.record_armed as i32,
                lane.min_value, lane.max_value,
            ],
        )?;

        for point in &lane.points {
            conn.execute(
                "INSERT INTO automation_points (lane_id, tick, value, curve_type)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    lane.id.get(),
                    point.tick as i64,
                    point.value,
                    format!("{:?}", point.curve)
                ],
            )?;
        }
    }
    Ok(())
}

// ============================================================
// Custom SynthDefs
// ============================================================

fn save_custom_synthdefs(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    for synth in &session.custom_synthdefs.synthdefs {
        conn.execute(
            "INSERT INTO custom_synthdefs (id, name, synthdef_name, source_path)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                synth.id.get(),
                synth.name,
                synth.synthdef_name,
                synth.source_path.to_string_lossy().to_string(),
            ],
        )?;

        for (pos, param) in synth.params.iter().enumerate() {
            conn.execute(
                "INSERT INTO custom_synthdef_params (synthdef_id, position, name, default_val, min_val, max_val)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![synth.id.get(), pos as i32, param.name, param.default, param.min, param.max],
            )?;
        }
    }
    Ok(())
}

// ============================================================
// VST Plugins
// ============================================================

fn save_vst_plugins(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    for plugin in &session.vst_plugins.plugins {
        conn.execute(
            "INSERT INTO vst_plugins (id, name, plugin_path, kind)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                plugin.id.get(),
                plugin.name,
                plugin.plugin_path.to_string_lossy().to_string(),
                format!("{:?}", plugin.kind),
            ],
        )?;

        for (pos, param) in plugin.params.iter().enumerate() {
            conn.execute(
                "INSERT INTO vst_plugin_params (plugin_id, position, param_index, name, default_val, label)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    plugin.id.get(), pos as i32, param.index,
                    param.name, param.default, param.label.as_deref(),
                ],
            )?;
        }
    }
    Ok(())
}

// ============================================================
// MIDI Recording
// ============================================================

fn save_midi_recording(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    let midi = &session.midi_recording;
    conn.execute(
        "INSERT INTO midi_recording_settings (id, live_input_track, note_passthrough, channel_filter)
         VALUES (1, ?1, ?2, ?3)",
        params![
            midi.live_input_instrument.map(|id| id.get() as i64),
            midi.note_passthrough as i32,
            midi.channel_filter.map(|ch| ch as i32),
        ],
    )?;

    for (idx, cc) in midi.cc_mappings.iter().enumerate() {
        let (
            target_type,
            target_inst_id,
            target_bus_id,
            target_effect_id,
            target_param_idx,
            target_extra,
        ) = encode_automation_target(&cc.target);
        let source_str = match cc.source {
            imbolc_types::CcMappingSource::Manual => "Manual",
            imbolc_types::CcMappingSource::Tag => "Tag",
        };
        conn.execute(
            "INSERT INTO midi_cc_mappings (id, cc_number, channel, target_type, target_track_id, target_bus_id, target_effect_id, target_param_idx, target_extra, min_value, max_value, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                idx as i64, cc.cc_number as i32,
                cc.channel.map(|ch| ch as i32),
                target_type, target_inst_id, target_bus_id,
                target_effect_id, target_param_idx, target_extra,
                cc.min_value, cc.max_value,
                source_str,
            ],
        )?;
    }

    for (idx, pb) in midi.pitch_bend_configs.iter().enumerate() {
        let (
            target_type,
            target_inst_id,
            target_bus_id,
            target_effect_id,
            target_param_idx,
            target_extra,
        ) = encode_automation_target(&pb.target);
        conn.execute(
            "INSERT INTO midi_pitch_bend_configs (id, target_type, target_track_id, target_bus_id, target_effect_id, target_param_idx, target_extra, center_value, range, sensitivity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                idx as i64,
                target_type, target_inst_id, target_bus_id,
                target_effect_id, target_param_idx, target_extra,
                pb.center_value, pb.range, pb.sensitivity,
            ],
        )?;
    }
    Ok(())
}

// ============================================================
// Parameter Tags
// ============================================================

fn save_param_tags(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    for (pos, tag) in session.param_tags.tags.iter().enumerate() {
        conn.execute(
            "INSERT INTO param_tags (id, name, position) VALUES (?1, ?2, ?3)",
            params![pos as i64, tag.name, pos as i32],
        )?;

        for (tpos, target) in tag.targets.iter().enumerate() {
            let (
                target_type,
                target_inst_id,
                target_bus_id,
                target_effect_id,
                target_param_idx,
                target_extra,
            ) = encode_automation_target(target);
            conn.execute(
                "INSERT INTO param_tag_targets (tag_id, position, target_type, target_track_id, target_bus_id, target_effect_id, target_param_idx, target_extra)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    pos as i64, tpos as i32,
                    target_type, target_inst_id, target_bus_id,
                    target_effect_id, target_param_idx, target_extra,
                ],
            )?;
        }
    }
    Ok(())
}

// ============================================================
// Arrangement
// ============================================================

fn save_arrangement(conn: &Connection, session: &SessionState) -> SqlResult<()> {
    let arr = &session.arrangement;

    conn.execute(
        "INSERT INTO arrangement_state (id, play_mode, selected_placement, selected_lane, view_start_tick, ticks_per_col, cursor_tick, next_clip_id, next_placement_id, next_clip_automation_lane_id)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            format!("{:?}", arr.play_mode),
            arr.selected_placement.map(|s| s as i64),
            arr.selected_lane as i64,
            arr.view_start_tick as i64,
            arr.ticks_per_col as i64,
            arr.cursor_tick as i64,
            arr.next_clip_id().get(),
            arr.next_placement_id().get(),
            arr.next_clip_automation_lane_id().get(),
        ],
    )?;

    // Clips
    for clip in &arr.clips {
        conn.execute(
            "INSERT INTO arrangement_clips (id, name, track_id, length_ticks)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                clip.id.get(),
                clip.name,
                clip.instrument_id.get(),
                clip.length_ticks as i64
            ],
        )?;

        // Clip notes
        for (pos, note) in clip.notes.iter().enumerate() {
            conn.execute(
                "INSERT INTO arrangement_clip_notes (clip_id, position, tick, duration, pitch, velocity, probability)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    clip.id.get(), pos as i32,
                    note.tick as i64, note.duration as i64,
                    note.pitch as i32, note.velocity as i32,
                    note.probability,
                ],
            )?;
        }

        // Clip automation lanes
        for lane in &clip.automation_lanes {
            let (
                target_type,
                target_inst_id,
                target_bus_id,
                target_effect_id,
                target_param_idx,
                target_extra,
            ) = encode_automation_target(&lane.target);
            conn.execute(
                "INSERT INTO arrangement_clip_automation_lanes (id, clip_id, target_type, target_track_id, target_bus_id, target_effect_id, target_param_idx, target_extra, enabled, record_armed, min_value, max_value)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    lane.id.get(), clip.id.get(), target_type, target_inst_id, target_bus_id,
                    target_effect_id, target_param_idx, target_extra,
                    lane.enabled as i32, lane.record_armed as i32,
                    lane.min_value, lane.max_value,
                ],
            )?;

            for point in &lane.points {
                conn.execute(
                    "INSERT INTO arrangement_clip_automation_points (lane_id, tick, value, curve_type)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![lane.id.get(), point.tick as i64, point.value, format!("{:?}", point.curve)],
                )?;
            }
        }
    }

    // Placements
    for placement in &arr.placements {
        conn.execute(
            "INSERT INTO arrangement_placements (id, clip_id, track_id, start_tick, length_override)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                placement.id.get(), placement.clip_id.get(), placement.instrument_id.get(),
                placement.start_tick as i64,
                placement.length_override.map(|l| l as i64),
            ],
        )?;
    }

    Ok(())
}

// ============================================================
// Encoding helpers
// ============================================================

fn encode_source_type(source: &crate::state::track::SourceType) -> String {
    use crate::state::track::SourceType;
    match source {
        SourceType::Custom(id) => format!("Custom:{}", id),
        SourceType::Vst(id) => format!("Vst:{}", id),
        other => format!("{:?}", other),
    }
}

fn encode_effect_type(effect: &crate::state::track::EffectType) -> String {
    use crate::state::track::EffectType;
    match effect {
        EffectType::Vst(id) => format!("Vst:{}", id),
        other => format!("{:?}", other),
    }
}

fn encode_tap_point(tap_point: crate::state::track::SendTapPoint) -> &'static str {
    use crate::state::track::SendTapPoint;
    match tap_point {
        SendTapPoint::PreInsert => "PreInsert",
        SendTapPoint::PostInsert => "PostInsert",
    }
}

fn encode_output_target(target: &crate::state::track::OutputTarget) -> String {
    use crate::state::track::OutputTarget;
    match target {
        OutputTarget::Master => "Master".to_string(),
        OutputTarget::Bus(id) => format!("Bus:{}", id),
    }
}

pub fn encode_parameter_target(target: &crate::state::track::ParameterTarget) -> String {
    use crate::state::track::ParameterTarget;
    match target {
        ParameterTarget::SendLevel(bus_id) => format!("SendLevel:bus:{}", bus_id.get()),
        ParameterTarget::EffectParam(eid, pidx) => format!("EffectParam:{}:{}", eid, pidx),
        ParameterTarget::EffectBypass(eid) => format!("EffectBypass:{}", eid),
        ParameterTarget::EqBandFreq(idx) => format!("EqBandFreq:{}", idx),
        ParameterTarget::EqBandGain(idx) => format!("EqBandGain:{}", idx),
        ParameterTarget::EqBandQ(idx) => format!("EqBandQ:{}", idx),
        ParameterTarget::EqBandSlope(idx) => format!("EqBandSlope:{}", idx),
        ParameterTarget::VstParam(idx) => format!("VstParam:{}", idx),
        ParameterTarget::SourceParam(idx) => format!("SourceParam:{}", idx),
        other => format!("{:?}", other),
    }
}

#[allow(clippy::type_complexity)]
pub fn encode_automation_target(
    target: &crate::state::AutomationTarget,
) -> (
    String,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<String>,
) {
    use crate::state::track::ParameterTarget;
    use imbolc_types::{AutomationTarget, BusParameter, GlobalParameter, TrackParameter};

    match target {
        AutomationTarget::Track(inst_id, TrackParameter::Standard(param_target)) => {
            let target_extra = match param_target {
                ParameterTarget::EffectParam(eid, pidx) => Some(format!("{}:{}", eid, pidx)),
                ParameterTarget::EffectBypass(eid) => Some(format!("{}", eid)),
                ParameterTarget::SendLevel(bus_id) => Some(format!("bus:{}", bus_id.get())),
                ParameterTarget::EqBandFreq(idx) => Some(format!("{}", idx)),
                ParameterTarget::EqBandGain(idx) => Some(format!("{}", idx)),
                ParameterTarget::EqBandQ(idx) => Some(format!("{}", idx)),
                ParameterTarget::EqBandSlope(idx) => Some(format!("{}", idx)),
                ParameterTarget::VstParam(idx) => Some(format!("{}", idx)),
                ParameterTarget::SourceParam(idx) => Some(format!("{}", idx)),
                _ => None,
            };
            let param_name = encode_parameter_target(param_target);
            (
                param_name,
                Some(inst_id.get() as i64),
                None,
                None,
                None,
                target_extra,
            )
        }
        AutomationTarget::Bus(bus_id, BusParameter::Level) => (
            "BusLevel".to_string(),
            None,
            Some(bus_id.get() as i64),
            None,
            None,
            None,
        ),
        AutomationTarget::Global(GlobalParameter::Bpm) => {
            ("GlobalBpm".to_string(), None, None, None, None, None)
        }
        AutomationTarget::Global(GlobalParameter::TimeSignature) => (
            "GlobalTimeSignature".to_string(),
            None,
            None,
            None,
            None,
            None,
        ),
        AutomationTarget::Generative(param) => {
            use imbolc_types::GenerativeParameter;
            let name = match param {
                GenerativeParameter::Density => "GenDensity",
                GenerativeParameter::Chaos => "GenChaos",
                GenerativeParameter::Energy => "GenEnergy",
                GenerativeParameter::Motion => "GenMotion",
            };
            (name.to_string(), None, None, None, None, None)
        }
    }
}

fn encode_param_value(
    value: &crate::state::param::ParamValue,
) -> (&str, Option<f64>, Option<i64>, Option<i32>) {
    use crate::state::param::ParamValue;
    match value {
        ParamValue::Float(v) => ("Float", Some(*v as f64), None, None),
        ParamValue::Int(v) => ("Int", None, Some(*v as i64), None),
        ParamValue::Bool(v) => ("Bool", None, None, Some(*v as i32)),
    }
}
