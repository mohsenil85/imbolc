use std::time::Duration;

use super::commands::AudioFeedback;
use super::engine::AudioEngine;
use super::snapshot::{SessionSnapshot, TrackSnapshot};
use imbolc_types::{SourceExtra, TrackId};

/// Tick the drum sequencer. Returns delayed feedbacks as `(delay_secs, feedback)` pairs,
/// where `delay_secs` matches the audio scheduling offset so the UI cursor stays in sync.
pub fn tick_drum_sequencer(
    instruments: &mut TrackSnapshot,
    session: &SessionSnapshot,
    bpm: f32,
    engine: &mut AudioEngine,
    rng_state: &mut u64,
    elapsed: Duration,
) -> Vec<(f64, AudioFeedback)> {
    // Collect instrument triggers to execute after the main loop
    // (target_track_id, freq, velocity, offset_secs)
    let mut instrument_triggers: Vec<(TrackId, f32, f32, f64)> = Vec::new();
    let mut delayed_feedbacks: Vec<(f64, AudioFeedback)> = Vec::new();

    for instrument in &mut instruments.tracks {
        let seq = match &mut instrument.source_extra {
            SourceExtra::Kit(s) => s,
            _ => continue,
        };
        if !seq.playing {
            seq.last_played_step = None;
            seq.reverse_fired.clear();
            continue;
        }

        let pattern_length = seq.pattern().length;
        let steps_per_beat = seq.step_resolution.steps_per_beat();
        let steps_per_second = (bpm as f64 / 60.0) * steps_per_beat;
        if steps_per_second <= 0.0 {
            continue;
        }
        let secs_per_step_unit = 1.0 / steps_per_second;

        let old_accum = seq.step_accumulator;
        seq.step_accumulator += elapsed.as_secs_f64() * steps_per_second;

        // Collect all steps that should fire in this tick with their precise offsets.
        // Each entry: (step_index, pattern_index, offset_secs)
        let mut steps_to_play: Vec<(usize, usize, f64)> = Vec::new();
        let mut threshold_consumed: f64 = 0.0;

        loop {
            // Swing threshold depends on which step boundary we're crossing
            let next_step = (seq.current_step + 1) % pattern_length;
            let swing_threshold: f64 = if seq.swing_amount > 0.0 && next_step % 2 == 1 {
                1.0 + seq.swing_amount as f64 * 0.5
            } else if seq.swing_amount > 0.0 && seq.current_step % 2 == 1 {
                1.0 - seq.swing_amount as f64 * 0.5
            } else {
                1.0
            };

            if seq.step_accumulator < swing_threshold {
                break;
            }

            seq.step_accumulator -= swing_threshold;
            threshold_consumed += swing_threshold;

            // Advance step
            let next = seq.current_step + 1;
            if next >= pattern_length {
                // Pattern wrapped — advance chain if enabled
                if seq.chain_enabled && !seq.chain.is_empty() {
                    seq.chain_position = (seq.chain_position + 1) % seq.chain.len();
                    let next_pattern = seq.chain[seq.chain_position];
                    if next_pattern < seq.patterns.len() {
                        if next_pattern != seq.current_pattern {
                            seq.reverse_fired.clear();
                        }
                        seq.current_pattern = next_pattern;
                    }
                }
                seq.current_step = 0;
            } else {
                seq.current_step = next;
            }

            // Precise offset: time from tick start to this step crossing
            let offset_secs = ((threshold_consumed - old_accum) * secs_per_step_unit).max(0.0)
                + engine.schedule_lookahead_secs;

            steps_to_play.push((seq.current_step, seq.current_pattern, offset_secs));
        }

        // Handle initial step when sequencer first starts (no threshold crossed yet)
        if steps_to_play.is_empty() && seq.last_played_step != Some(seq.current_step) {
            steps_to_play.push((
                seq.current_step,
                seq.current_pattern,
                engine.schedule_lookahead_secs,
            ));
        }

        // Play each step with its precise offset
        for &(step, pattern_idx, offset_secs) in &steps_to_play {
            if engine.is_running() && !instrument.channel_strip.mute {
                let pattern = &seq.patterns[pattern_idx];
                for (pad_idx, pad) in seq.pads.iter().enumerate() {
                    if let Some(step_data) = pattern.steps.get(pad_idx).and_then(|s| s.get(step)) {
                        if !step_data.active {
                            continue;
                        }

                        // Skip reversed samples that were already pre-triggered
                        if pad.reverse
                            && pad.buffer_id.is_some()
                            && pad.duration_secs > 0.0
                            && seq.reverse_fired.remove(&(pad_idx, step))
                        {
                            continue;
                        }

                        // Probability check: skip hit if random exceeds probability
                        if step_data.probability < 1.0 {
                            *rng_state = rng_state
                                .wrapping_mul(6364136223846793005)
                                .wrapping_add(1442695040888963407);
                            let r = ((*rng_state >> 33) as f32) / (u32::MAX as f32);
                            if r > step_data.probability {
                                continue;
                            }
                        }

                        // Per-pad groove with fallback: pad → track → global
                        let track_humanize_vel = instrument
                            .groove
                            .humanize_velocity
                            .unwrap_or(session.humanize.velocity);
                        let track_humanize_time = instrument
                            .groove
                            .humanize_timing
                            .unwrap_or(session.humanize.timing);
                        let effective_humanize_vel =
                            pad.groove.humanize_velocity.unwrap_or(track_humanize_vel);
                        let effective_humanize_time =
                            pad.groove.humanize_timing.unwrap_or(track_humanize_time);
                        let timing_offset_ms = if pad.groove.timing_offset_ms != 0.0 {
                            pad.groove.timing_offset_ms
                        } else {
                            instrument.groove.timing_offset_ms
                        };

                        // Calculate final offset with timing offset (rush/drag)
                        let mut final_offset = offset_secs + (timing_offset_ms / 1000.0) as f64;

                        // Per-pad swing: apply additional timing offset on odd steps
                        if let Some(pad_swing) = pad.groove.swing_amount {
                            if pad_swing > 0.0 && step % 2 == 1 {
                                let half_step_secs = secs_per_step_unit * 0.5;
                                final_offset += pad_swing as f64 * half_step_secs;
                            }
                        }

                        // Timing humanization: jitter offset by up to +/- 20ms
                        if effective_humanize_time > 0.0 {
                            *rng_state = rng_state
                                .wrapping_mul(6364136223846793005)
                                .wrapping_add(1442695040888963407);
                            let r = ((*rng_state >> 33) as f32) / (u32::MAX as f32);
                            let jitter = (r - 0.5) * 2.0 * effective_humanize_time * 0.02;
                            final_offset = (final_offset + jitter as f64).max(0.0);
                        }

                        let mut amp = (step_data.velocity as f32 / 127.0) * pad.level;
                        // Velocity humanization using per-track setting
                        if effective_humanize_vel > 0.0 {
                            *rng_state = rng_state
                                .wrapping_mul(6364136223846793005)
                                .wrapping_add(1442695040888963407);
                            let r = ((*rng_state >> 33) as f32) / (u32::MAX as f32);
                            let jitter = (r - 0.5) * 2.0 * effective_humanize_vel * (30.0 / 127.0);
                            amp = (amp + jitter).clamp(0.01, 1.0);
                        }

                        // Calculate pitch offset (used for both samples and instruments)
                        let total_pitch = pad.pitch as i16 + step_data.pitch_offset as i16;

                        // Check if this pad triggers an instrument (one-shot synth)
                        if let Some(target_track_id) = pad.instrument_id {
                            // Track trigger mode: collect for execution after loop
                            let freq = pad.trigger_freq * 2.0_f32.powf(total_pitch as f32 / 12.0);
                            instrument_triggers.push((target_track_id, freq, amp, final_offset));
                        } else if let Some(buffer_id) = pad.buffer_id {
                            // Sample mode: play one-shot sample
                            let pitch_rate = 2.0_f32.powf(total_pitch as f32 / 12.0);
                            let rate = if pad.reverse { -pitch_rate } else { pitch_rate };
                            let _ = engine.play_drum_hit_to_instrument(
                                buffer_id,
                                amp,
                                instrument.id,
                                pad.slice_start,
                                pad.slice_end,
                                rate,
                                final_offset,
                            );
                        }
                    }
                }
            }
            delayed_feedbacks.push((
                offset_secs,
                AudioFeedback::DrumSequencerStep {
                    instrument_id: instrument.id,
                    step,
                },
            ));
            seq.last_played_step = Some(step);
        }

        // --- Reverse pre-trigger look-ahead pass ---
        // For reversed sample pads, scan ahead in the pattern and fire early so the
        // sample finishes playing at the target step time (builds into the beat).
        if engine.is_running() && !instrument.channel_strip.mute {
            let elapsed_secs = elapsed.as_secs_f64();
            let current_pattern_idx = seq.current_pattern;
            let current_step = seq.current_step;
            let step_accum = seq.step_accumulator;
            let pattern = &seq.patterns[current_pattern_idx];

            for (pad_idx, pad) in seq.pads.iter().enumerate() {
                // Only reversed sample pads with known duration
                if !pad.reverse
                    || pad.buffer_id.is_none()
                    || pad.duration_secs <= 0.0
                    || pad.instrument_id.is_some()
                {
                    continue;
                }

                let buffer_id = pad.buffer_id.unwrap();
                let slice_length = (pad.slice_end - pad.slice_start).abs();
                if slice_length <= 0.0 {
                    continue;
                }

                // Base playback duration at pad pitch (used to determine look-ahead range)
                let base_pitch_rate = 2.0_f32.powf(pad.pitch as f32 / 12.0).max(0.01);
                let max_playback_secs =
                    pad.duration_secs as f64 * slice_length as f64 / base_pitch_rate as f64;

                // How many steps ahead to scan
                let look_ahead_steps = ((max_playback_secs * steps_per_second).ceil() as usize + 1)
                    .min(pattern_length);

                for ahead in 1..=look_ahead_steps {
                    let target_step = (current_step + ahead) % pattern_length;

                    if seq.reverse_fired.contains(&(pad_idx, target_step)) {
                        continue;
                    }

                    let step_data =
                        match pattern.steps.get(pad_idx).and_then(|s| s.get(target_step)) {
                            Some(sd) if sd.active => sd,
                            _ => continue,
                        };

                    // Per-step pitch affects actual playback duration
                    let total_pitch = pad.pitch as i16 + step_data.pitch_offset as i16;
                    let step_pitch_rate = 2.0_f32.powf(total_pitch as f32 / 12.0).max(0.01);
                    let playback_secs =
                        pad.duration_secs as f64 * slice_length as f64 / step_pitch_rate as f64;

                    // Time until the target step (seconds from tick start)
                    let time_until_target = (ahead as f64 - step_accum) * secs_per_step_unit;

                    // Pre-trigger time: start playing so it finishes at the target step
                    let trigger_time = time_until_target - playback_secs;

                    // Fire if trigger falls within this tick window.
                    // Use osc_offset check to allow slightly negative trigger_time
                    // (when trigger point falls just before tick start due to granularity).
                    let osc_offset = trigger_time + engine.schedule_lookahead_secs;
                    if osc_offset >= 0.0 && trigger_time < elapsed_secs {
                        // Probability check
                        if step_data.probability < 1.0 {
                            *rng_state = rng_state
                                .wrapping_mul(6364136223846793005)
                                .wrapping_add(1442695040888963407);
                            let r = ((*rng_state >> 33) as f32) / (u32::MAX as f32);
                            if r > step_data.probability {
                                // Mark as fired so we don't re-check
                                seq.reverse_fired.insert((pad_idx, target_step));
                                continue;
                            }
                        }

                        let amp = (step_data.velocity as f32 / 127.0) * pad.level;
                        let rate = -step_pitch_rate; // negative for reverse
                        let _ = engine.play_drum_hit_to_instrument(
                            buffer_id,
                            amp,
                            instrument.id,
                            pad.slice_start,
                            pad.slice_end,
                            rate,
                            osc_offset,
                        );

                        seq.reverse_fired.insert((pad_idx, target_step));
                    }
                }
            }
        }
    }

    // Execute collected instrument triggers (needs immutable borrow of instruments)
    for (target_id, freq, amp, offset) in instrument_triggers {
        let _ =
            engine.trigger_instrument_oneshot(target_id, freq, amp, offset, instruments, session);
    }

    delayed_feedbacks
}

#[cfg(test)]
mod tests {
    use super::*;
    use imbolc_types::state::drum_sequencer::DrumSequencerState;
    use imbolc_types::{BufferId, SourceType, Track};

    /// Create a minimal test setup with one Kit instrument.
    fn setup_kit() -> (TrackSnapshot, SessionSnapshot, AudioEngine) {
        let mut tracks = TrackSnapshot::new();
        let mut inst = Track::new(TrackId::new(1), SourceType::Kit);
        if let SourceExtra::Kit(ref mut seq) = inst.source_extra {
            seq.playing = true;
        }
        tracks.tracks.push(inst);
        let session = SessionSnapshot::new();
        let mut engine = AudioEngine::new();
        // Mark engine as running so audio firing paths are exercised
        engine.is_running = true;
        (tracks, session, engine)
    }

    fn get_seq(tracks: &mut TrackSnapshot) -> &mut DrumSequencerState {
        match &mut tracks.tracks[0].source_extra {
            SourceExtra::Kit(s) => s,
            _ => unreachable!(),
        }
    }

    #[test]
    fn reverse_pad_without_duration_fires_normally() {
        // A reversed pad with duration_secs=0 should fire at step time (no pre-trigger)
        let (mut tracks, session, mut engine) = setup_kit();
        let seq = get_seq(&mut tracks);
        seq.pads[0].buffer_id = Some(BufferId::new(1));
        seq.pads[0].reverse = true;
        seq.pads[0].duration_secs = 0.0; // unknown duration
        seq.pattern_mut().steps[0][0].active = true;

        let mut rng = 12345u64;
        let feedbacks = tick_drum_sequencer(
            &mut tracks,
            &session,
            120.0,
            &mut engine,
            &mut rng,
            Duration::from_millis(1),
        );

        // Should produce a step feedback for step 0 (fired normally)
        assert_eq!(feedbacks.len(), 1);
        assert!(matches!(
            feedbacks[0].1,
            AudioFeedback::DrumSequencerStep { step: 0, .. }
        ));
    }

    #[test]
    fn reverse_pretrigger_fires_ahead_and_skips_at_step() {
        // A reversed pad with known duration should be pre-triggered early,
        // then skipped when the actual step is reached.
        let (mut tracks, session, mut engine) = setup_kit();
        let seq = get_seq(&mut tracks);
        seq.pads[0].buffer_id = Some(BufferId::new(1));
        seq.pads[0].reverse = true;
        seq.pads[0].duration_secs = 0.25; // 250ms sample

        // At 120 BPM, 1/16 steps: steps_per_second = 8, secs_per_step = 0.125
        // Sample duration = 0.25s = 2 steps of look-ahead
        // Activate step 4 — pre-trigger should fire when we're at step 2
        // (time_until_step4 = 2 * 0.125 = 0.25s, trigger_time = 0.25 - 0.25 = 0.0)
        seq.pattern_mut().steps[0][4].active = true;

        // Advance to step 2 by ticking enough to cross 2 steps
        // Each step = 0.125s, so we need ~0.25s + epsilon to land at step 2
        let mut rng = 12345u64;

        // First: advance past step 0 (initial step) and to step 2
        // Tick 1: initial step 0
        tick_drum_sequencer(
            &mut tracks,
            &session,
            120.0,
            &mut engine,
            &mut rng,
            Duration::from_millis(1),
        );

        // Tick 2: advance ~250ms to cross step 1 and step 2
        tick_drum_sequencer(
            &mut tracks,
            &session,
            120.0,
            &mut engine,
            &mut rng,
            Duration::from_millis(250),
        );

        // Steps 1 and 2 should have been crossed (no active pads though)
        // The pre-trigger for step 4 should have fired (trigger_time ≈ 0)
        let seq = get_seq(&mut tracks);
        assert!(
            seq.reverse_fired.contains(&(0, 4)),
            "Step 4 should be in reverse_fired after pre-trigger"
        );

        // Now advance to step 4
        let feedbacks = tick_drum_sequencer(
            &mut tracks,
            &session,
            120.0,
            &mut engine,
            &mut rng,
            Duration::from_millis(250),
        );

        // Step 4 should have been crossed — check that the pre-triggered entry was consumed
        let seq = get_seq(&mut tracks);
        assert!(
            !seq.reverse_fired.contains(&(0, 4)),
            "Step 4 should be removed from reverse_fired after crossing"
        );

        // The step 4 feedback should still appear (UI cursor update)
        let step4_feedback = feedbacks
            .iter()
            .any(|(_, fb)| matches!(fb, AudioFeedback::DrumSequencerStep { step: 4, .. }));
        assert!(step4_feedback, "Step 4 feedback should still be emitted");
    }

    #[test]
    fn reverse_fired_cleared_on_stop() {
        let (mut tracks, session, mut engine) = setup_kit();
        let seq = get_seq(&mut tracks);
        seq.pads[0].buffer_id = Some(BufferId::new(1));
        seq.pads[0].reverse = true;
        seq.pads[0].duration_secs = 0.5;
        seq.pattern_mut().steps[0][8].active = true;
        seq.reverse_fired.insert((0, 8));

        // Stop the sequencer
        seq.playing = false;

        let mut rng = 12345u64;
        tick_drum_sequencer(
            &mut tracks,
            &session,
            120.0,
            &mut engine,
            &mut rng,
            Duration::from_millis(1),
        );

        let seq = get_seq(&mut tracks);
        assert!(
            seq.reverse_fired.is_empty(),
            "reverse_fired should be cleared on stop"
        );
    }

    #[test]
    fn reverse_fired_cleared_on_chain_pattern_change() {
        let (mut tracks, session, mut engine) = setup_kit();
        let seq = get_seq(&mut tracks);
        seq.pads[0].buffer_id = Some(BufferId::new(1));
        seq.pads[0].reverse = true;
        seq.pads[0].duration_secs = 0.5;
        seq.chain_enabled = true;
        seq.chain = vec![0, 1]; // Chain patterns 0 → 1
        seq.chain_position = 0;

        // Pre-seed a reverse_fired entry
        seq.reverse_fired.insert((0, 5));

        // Advance to the end of the pattern to trigger chain advance
        seq.current_step = 15; // Last step (pattern length = 16)
        seq.step_accumulator = 0.0;

        let mut rng = 12345u64;
        // Tick enough to cross the pattern boundary
        tick_drum_sequencer(
            &mut tracks,
            &session,
            120.0,
            &mut engine,
            &mut rng,
            Duration::from_millis(200),
        );

        let seq = get_seq(&mut tracks);
        // Pattern should have changed and reverse_fired cleared
        assert_eq!(seq.current_pattern, 1);
        assert!(
            seq.reverse_fired.is_empty(),
            "reverse_fired should be cleared on pattern change"
        );
    }

    #[test]
    fn non_reversed_pad_unaffected_by_pretrigger() {
        // A non-reversed pad should fire at step time as usual, regardless of duration_secs
        let (mut tracks, session, mut engine) = setup_kit();
        let seq = get_seq(&mut tracks);
        seq.pads[0].buffer_id = Some(BufferId::new(1));
        seq.pads[0].reverse = false;
        seq.pads[0].duration_secs = 1.0;
        seq.pattern_mut().steps[0][0].active = true;

        let mut rng = 12345u64;
        let feedbacks = tick_drum_sequencer(
            &mut tracks,
            &session,
            120.0,
            &mut engine,
            &mut rng,
            Duration::from_millis(1),
        );

        // Should fire normally on step 0
        assert_eq!(feedbacks.len(), 1);
        let seq = get_seq(&mut tracks);
        assert!(
            seq.reverse_fired.is_empty(),
            "Non-reversed pad should not use reverse_fired"
        );
    }

    #[test]
    fn instrument_trigger_pad_unaffected_by_pretrigger() {
        // Instrument trigger pads should not be pre-triggered even if reversed
        let (mut tracks, session, mut engine) = setup_kit();
        let seq = get_seq(&mut tracks);
        seq.pads[0].instrument_id = Some(TrackId::new(2));
        seq.pads[0].reverse = true;
        seq.pads[0].duration_secs = 1.0;
        seq.pattern_mut().steps[0][0].active = true;

        let mut rng = 12345u64;
        let feedbacks = tick_drum_sequencer(
            &mut tracks,
            &session,
            120.0,
            &mut engine,
            &mut rng,
            Duration::from_millis(1),
        );

        assert_eq!(feedbacks.len(), 1);
        let seq = get_seq(&mut tracks);
        assert!(seq.reverse_fired.is_empty());
    }
}
