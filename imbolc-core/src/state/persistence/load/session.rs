use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};

use super::decoders::*;
use crate::state::session::SessionState;
use crate::state::track_state::TrackState;

pub(super) fn load_session(
    conn: &Connection,
    session: &mut SessionState,
    tracks: &mut TrackState,
) -> SqlResult<()> {
    let row = conn.query_row(
        "SELECT bpm, time_sig_num, time_sig_denom, key, scale, tuning_a4, snap,
                next_track_id, next_sampler_buffer_id, selected_track, next_layer_group_id,
                humanize_velocity, humanize_timing,
                click_enabled, click_volume, click_muted,
                tuning, ji_flavor, next_sample_id
         FROM session WHERE id = 1",
        [],
        |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, f32>(5)?,
                row.get::<_, i32>(6)?,
                row.get::<_, u32>(7)?,
                row.get::<_, u32>(8)?,
                row.get::<_, Option<i64>>(9)?,
                row.get::<_, u32>(10)?,
                row.get::<_, f32>(11)?,
                row.get::<_, f32>(12)?,
                row.get::<_, i32>(13)?,
                row.get::<_, f32>(14)?,
                row.get::<_, i32>(15)?,
                row.get::<_, String>(16)?,
                row.get::<_, String>(17)?,
                row.get::<_, u32>(18)?,
            ))
        },
    )?;

    session.bpm = row.0 as u16;
    session.time_signature = (row.1 as u8, row.2 as u8);
    session.key = decode_key(&row.3);
    session.scale = decode_scale(&row.4);
    session.tuning_a4 = row.5;
    session.snap = row.6 != 0;
    tracks.next_id = imbolc_types::TrackId::new(row.7);
    tracks.next_sampler_buffer_id = imbolc_types::BufferId::new(row.8);
    tracks.selected = row.9.map(|v| v as usize);
    tracks.next_layer_group_id = imbolc_types::GroupId::new(row.10);
    session.humanize.velocity = row.11;
    session.humanize.timing = row.12;
    session.click_track.enabled = row.13 != 0;
    session.click_track.volume = row.14;
    session.click_track.muted = row.15 != 0;
    session.tuning = decode_tuning(&row.16);
    session.ji_flavor = decode_ji_flavor(&row.17);
    tracks.next_sample_id = imbolc_types::SampleId::new(row.18);

    Ok(())
}

pub(super) fn load_musical_settings(
    conn: &Connection,
    session: &mut SessionState,
) -> SqlResult<()> {
    let result = conn.query_row(
        "SELECT bpm, time_sig_num, time_sig_denom, ticks_per_beat, loop_start, loop_end, looping, swing_amount
         FROM musical_settings WHERE id = 1",
        [],
        |row| {
            Ok((
                row.get::<_, f32>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, u32>(3)?,
                row.get::<_, u32>(4)?,
                row.get::<_, u32>(5)?,
                row.get::<_, i32>(6)?,
                row.get::<_, f32>(7)?,
            ))
        },
    ).optional()?;

    if let Some((bpm, tsn, tsd, tpb, ls, le, looping, swing)) = result {
        session.piano_roll.bpm = bpm;
        session.piano_roll.time_signature = (tsn as u8, tsd as u8);
        session.piano_roll.ticks_per_beat = tpb;
        session.piano_roll.loop_start = ls;
        session.piano_roll.loop_end = le;
        session.piano_roll.looping = looping != 0;
        session.piano_roll.swing_amount = swing;
    }

    Ok(())
}

pub(super) fn load_piano_roll(conn: &Connection, session: &mut SessionState) -> SqlResult<()> {
    use crate::state::piano_roll::Note;

    // Clear existing sequences
    session.piano_roll.sequences.clear();
    session.piano_roll.sequence_order.clear();

    // Load tracks ordered by position
    let mut track_stmt =
        conn.prepare("SELECT track_id, polyphonic FROM piano_roll_tracks ORDER BY position")?;
    let tracks: Vec<(imbolc_types::TrackId, bool)> = track_stmt
        .query_map([], |row| {
            Ok((
                imbolc_types::TrackId::new(row.get::<_, u32>(0)?),
                row.get::<_, i32>(1)? != 0,
            ))
        })?
        .collect::<SqlResult<_>>()?;

    for (inst_id, polyphonic) in &tracks {
        session.piano_roll.add_sequence(*inst_id);
        if let Some(seq) = session.piano_roll.sequences.get_mut(inst_id) {
            seq.polyphonic = *polyphonic;
        }
    }

    // Load notes
    let mut note_stmt = conn.prepare(
        "SELECT track_track_id, tick, duration, pitch, velocity, probability
         FROM piano_roll_notes ORDER BY track_track_id, tick",
    )?;
    let notes: Vec<(imbolc_types::TrackId, Note)> = note_stmt
        .query_map([], |row| {
            Ok((
                imbolc_types::TrackId::new(row.get::<_, u32>(0)?),
                Note {
                    tick: row.get::<_, u32>(1)?,
                    duration: row.get::<_, u32>(2)?,
                    pitch: row.get::<_, i32>(3)? as u8,
                    velocity: row.get::<_, i32>(4)? as u8,
                    probability: row.get::<_, f32>(5)?,
                },
            ))
        })?
        .collect::<SqlResult<_>>()?;

    for (inst_id, note) in notes {
        if let Some(seq) = session.piano_roll.sequences.get_mut(&inst_id) {
            seq.notes.push(note);
        }
    }

    Ok(())
}

pub(super) fn load_custom_synthdefs(
    conn: &Connection,
    session: &mut SessionState,
) -> SqlResult<()> {
    use crate::state::custom_synthdef::{CustomSynthDef, CustomSynthDefRegistry, ParamSpec};

    let mut registry = CustomSynthDefRegistry::new();

    let mut stmt = conn
        .prepare("SELECT id, name, synthdef_name, source_path FROM custom_synthdefs ORDER BY id")?;
    let synthdefs: Vec<(u32, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<SqlResult<_>>()?;

    for (id, name, synthdef_name, source_path) in synthdefs {
        let mut param_stmt = conn.prepare(
            "SELECT name, default_val, min_val, max_val FROM custom_synthdef_params WHERE synthdef_id = ?1 ORDER BY position"
        )?;
        let params: Vec<ParamSpec> = param_stmt
            .query_map(params![id], |row| {
                Ok(ParamSpec {
                    name: row.get(0)?,
                    default: row.get(1)?,
                    min: row.get(2)?,
                    max: row.get(3)?,
                })
            })?
            .collect::<SqlResult<_>>()?;

        registry.add(CustomSynthDef {
            id: imbolc_types::CustomSynthDefId::new(id),
            name,
            synthdef_name,
            source_path: PathBuf::from(source_path),
            params,
        });
    }

    session.custom_synthdefs = registry;
    Ok(())
}

pub(super) fn load_vst_plugins(conn: &Connection, session: &mut SessionState) -> SqlResult<()> {
    use crate::state::vst_plugin::{VstParamSpec, VstPlugin, VstPluginKind, VstPluginRegistry};

    let mut registry = VstPluginRegistry::new();

    let mut stmt =
        conn.prepare("SELECT id, name, plugin_path, kind FROM vst_plugins ORDER BY id")?;
    let plugins: Vec<(u32, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<SqlResult<_>>()?;

    for (id, name, plugin_path, kind_str) in plugins {
        let kind = match kind_str.as_str() {
            "Effect" => VstPluginKind::Effect,
            _ => VstPluginKind::Track,
        };

        let mut param_stmt = conn.prepare(
            "SELECT param_index, name, default_val, label FROM vst_plugin_params WHERE plugin_id = ?1 ORDER BY position"
        )?;
        let params: Vec<VstParamSpec> = param_stmt
            .query_map(params![id], |row| {
                Ok(VstParamSpec {
                    index: row.get(0)?,
                    name: row.get(1)?,
                    default: row.get(2)?,
                    label: row.get(3)?,
                })
            })?
            .collect::<SqlResult<_>>()?;

        registry.add(VstPlugin {
            id: imbolc_types::VstPluginId::new(id),
            name,
            plugin_path: PathBuf::from(plugin_path),
            kind,
            params,
        });
    }

    session.vst_plugins = registry;
    Ok(())
}
