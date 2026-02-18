use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};

use super::decoders::*;
use super::{load_effects_from, table_exists};
use crate::state::session::SessionState;
use imbolc_types::state::channel_strip::ChannelStrip;
use imbolc_types::BusId;

pub(super) fn load_mixer(conn: &Connection, session: &mut SessionState) -> SqlResult<()> {
    use imbolc_types::MixerBus;

    session.mixer.buses.clear();

    let has_bus_effects = table_exists(conn, "bus_effects")?;

    let mut stmt =
        conn.prepare("SELECT id, name, level, pan, mute, solo FROM mixer_buses ORDER BY id")?;
    let buses = stmt.query_map([], |row| {
        Ok(MixerBus {
            id: BusId::new(row.get::<_, i32>(0)? as u8),
            name: row.get(1)?,
            channel_strip: ChannelStrip {
                level: row.get(2)?,
                pan: row.get(3)?,
                mute: row.get::<_, i32>(4)? != 0,
                solo: row.get::<_, i32>(5)? != 0,
                ..ChannelStrip::new_bus()
            },
        })
    })?;

    for bus in buses {
        let mut bus = bus?;
        if has_bus_effects {
            let effects = load_effects_from(
                conn,
                "bus_effects",
                "bus_effect_params",
                "bus_effect_vst_params",
                "bus_id",
                bus.id.get() as u32,
            )?;
            for effect in effects {
                bus.channel_strip
                    .processing_chain
                    .push(imbolc_types::ProcessingStage::Effect(effect));
            }
            bus.channel_strip.recalculate_next_effect_id();
        }
        session.mixer.buses.push(bus);
    }

    // Master
    let result: Option<(f32, i32)> = conn
        .query_row(
            "SELECT level, mute FROM mixer_master WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    if let Some((level, mute)) = result {
        session.mixer.master_level = level;
        session.mixer.master_mute = mute != 0;
    }

    // Load master EQ
    if table_exists(conn, "master_eq_bands")? {
        let eq_enabled: i32 = conn
            .query_row(
                "SELECT eq_enabled FROM mixer_master WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if eq_enabled != 0 {
            let mut eq = crate::state::track::EqConfig::default();
            let mut band_stmt = conn.prepare(
                "SELECT band_index, freq, gain, q, enabled FROM master_eq_bands ORDER BY band_index",
            )?;
            let bands = band_stmt
                .query_map([], |row| {
                    let band_index: usize = row.get::<_, i32>(0)? as usize;
                    let freq: f32 = row.get(1)?;
                    let gain: f32 = row.get(2)?;
                    let q: f32 = row.get(3)?;
                    let enabled: bool = row.get::<_, i32>(4)? != 0;
                    Ok((band_index, freq, gain, q, enabled))
                })?
                .collect::<SqlResult<Vec<_>>>()?;
            for (band_index, freq, gain, q, enabled) in bands {
                if band_index < eq.bands.len() {
                    eq.bands[band_index].freq = freq;
                    eq.bands[band_index].gain = gain;
                    eq.bands[band_index].q = q;
                    eq.bands[band_index].enabled = enabled;
                }
            }
            // Enable EQ on master channel strip if not already present
            if !session.mixer.master_channel_strip.has_eq() {
                session.mixer.master_channel_strip.toggle_eq();
            }
            if let Some(master_eq) = session.mixer.master_channel_strip.eq_mut() {
                *master_eq = eq;
            }
        }
    }

    // Load bus EQ bands
    if table_exists(conn, "bus_eq_bands")? {
        for bus in &mut session.mixer.buses {
            let mut band_stmt = conn.prepare(
                "SELECT band_index, freq, gain, q, enabled FROM bus_eq_bands WHERE bus_id = ?1 ORDER BY band_index",
            )?;
            let bands = band_stmt
                .query_map([bus.id.get() as i32], |row| {
                    let band_index: usize = row.get::<_, i32>(0)? as usize;
                    let freq: f32 = row.get(1)?;
                    let gain: f32 = row.get(2)?;
                    let q: f32 = row.get(3)?;
                    let enabled: bool = row.get::<_, i32>(4)? != 0;
                    Ok((band_index, freq, gain, q, enabled))
                })?
                .collect::<SqlResult<Vec<_>>>()?;
            if !bands.is_empty() {
                let mut eq = crate::state::track::EqConfig::default();
                for (band_index, freq, gain, q, enabled) in bands {
                    if band_index < eq.bands.len() {
                        eq.bands[band_index].freq = freq;
                        eq.bands[band_index].gain = gain;
                        eq.bands[band_index].q = q;
                        eq.bands[band_index].enabled = enabled;
                    }
                }
                bus.channel_strip.processing_chain.insert(
                    0,
                    imbolc_types::ProcessingStage::Eq(imbolc_types::EffectId::new(0), eq),
                );
            }
        }
    }

    Ok(())
}

pub(super) fn load_layer_group_mixers(
    conn: &Connection,
    session: &mut SessionState,
) -> SqlResult<()> {
    use crate::state::track::MixerSend;
    use imbolc_types::GroupMixer;

    session.mixer.layer_group_mixers.clear();

    let has_group_effects = table_exists(conn, "layer_group_effects")?;

    let mut stmt = conn.prepare(
        "SELECT group_id, name, level, pan, mute, solo, output_target FROM layer_group_mixers ORDER BY group_id"
    )?;
    let rows: Vec<(u32, String, f32, f32, i32, i32, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i32>(0)? as u32,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })?
        .collect::<SqlResult<_>>()?;

    for (group_id_raw, name, level, pan, mute, solo, output_target_str) in rows {
        let group_id = imbolc_types::GroupId::new(group_id_raw);
        let output_target = decode_output_target(&output_target_str);

        // Load sends
        let has_group_tap_point = conn
            .prepare("SELECT tap_point FROM layer_group_sends LIMIT 0")
            .is_ok();
        let group_send_query = if has_group_tap_point {
            "SELECT bus_id, level, enabled, tap_point FROM layer_group_sends WHERE group_id = ?1 ORDER BY bus_id"
        } else {
            "SELECT bus_id, level, enabled FROM layer_group_sends WHERE group_id = ?1 ORDER BY bus_id"
        };
        let mut send_stmt = conn.prepare(group_send_query)?;
        let sends: std::collections::BTreeMap<BusId, MixerSend> = send_stmt
            .query_map(params![group_id_raw as i32], |row| {
                let tap_point = if has_group_tap_point {
                    decode_tap_point(&row.get::<_, String>(3)?)
                } else {
                    Default::default()
                };
                let send = MixerSend {
                    bus_id: BusId::new(row.get::<_, i32>(0)? as u8),
                    level: row.get(1)?,
                    enabled: row.get::<_, i32>(2)? != 0,
                    tap_point,
                };
                Ok((send.bus_id, send))
            })?
            .collect::<SqlResult<_>>()?;

        let mut gm = GroupMixer {
            group_id,
            name,
            channel_strip: ChannelStrip {
                level,
                pan,
                mute: mute != 0,
                solo: solo != 0,
                output_target,
                sends,
                processing_chain: Vec::new(),
                ..ChannelStrip::new_layer_group()
            },
        };
        // Clear the default EQ from the processing chain — we'll load it explicitly below
        gm.channel_strip.processing_chain.clear();

        if has_group_effects {
            let effects = load_effects_from(
                conn,
                "layer_group_effects",
                "layer_group_effect_params",
                "layer_group_effect_vst_params",
                "group_id",
                group_id_raw,
            )?;
            for effect in effects {
                gm.channel_strip
                    .processing_chain
                    .push(imbolc_types::ProcessingStage::Effect(effect));
            }
            gm.channel_strip.recalculate_next_effect_id();
        }

        // Load EQ if the table exists
        if table_exists(conn, "layer_group_eq_bands")? {
            let eq_enabled: i32 = conn
                .query_row(
                    "SELECT eq_enabled FROM layer_group_mixers WHERE group_id = ?1",
                    [group_id_raw],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if eq_enabled != 0 {
                let mut eq = crate::state::track::EqConfig::default();
                let mut band_stmt = conn.prepare(
                    "SELECT band_index, freq, gain, q, enabled FROM layer_group_eq_bands WHERE group_id = ?1 ORDER BY band_index"
                )?;
                let bands = band_stmt
                    .query_map([group_id_raw], |row| {
                        let band_index: usize = row.get::<_, i32>(0)? as usize;
                        let freq: f32 = row.get(1)?;
                        let gain: f32 = row.get(2)?;
                        let q: f32 = row.get(3)?;
                        let enabled: bool = row.get::<_, i32>(4)? != 0;
                        Ok((band_index, freq, gain, q, enabled))
                    })?
                    .collect::<SqlResult<Vec<_>>>()?;
                for (band_index, freq, gain, q, enabled) in bands {
                    if band_index < eq.bands.len() {
                        eq.bands[band_index].freq = freq;
                        eq.bands[band_index].gain = gain;
                        eq.bands[band_index].q = q;
                        eq.bands[band_index].enabled = enabled;
                    }
                }
                // Insert EQ at beginning of processing chain (before effects)
                gm.channel_strip.processing_chain.insert(
                    0,
                    imbolc_types::ProcessingStage::Eq(imbolc_types::EffectId::new(0), eq),
                );
            }
        }

        session.mixer.layer_group_mixers.push(gm);
    }

    Ok(())
}
