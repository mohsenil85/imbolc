use crate::action::{ArrangementAction, AudioEffect, DispatchResult, NavIntent, PaneId};
use crate::state::arrangement::{ClipEditContext, PlayMode};
use crate::state::AppState;
use imbolc_audio::AudioHandle;

pub(super) fn dispatch_arrangement(
    action: &ArrangementAction,
    state: &mut AppState,
    audio: &mut AudioHandle,
) -> DispatchResult {
    match action {
        ArrangementAction::TogglePlayMode => {
            let arr = &mut state.session.arrangement;
            arr.play_mode = match arr.play_mode {
                PlayMode::Pattern => PlayMode::Song,
                PlayMode::Song => PlayMode::Pattern,
            };
            let mut result = DispatchResult::none();
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
            result
        }
        ArrangementAction::CreateClip {
            instrument_id,
            length_ticks,
        } => {
            let clip_id = state.session.arrangement.add_clip(
                "Clip".to_string(),
                *instrument_id,
                *length_ticks,
            );
            if let Some(clip) = state.session.arrangement.clip_mut(clip_id) {
                clip.name = format!("Clip {}", clip_id);
            }
            DispatchResult::none()
        }
        ArrangementAction::CaptureClipFromPianoRoll { instrument_id } => {
            let (loop_start, loop_end, seq_notes) = {
                let pr = &state.session.piano_roll;
                let notes = pr
                    .sequences
                    .get(instrument_id)
                    .map(|t| t.notes.clone())
                    .unwrap_or_default();
                (pr.loop_start, pr.loop_end, notes)
            };

            let length_ticks = loop_end.saturating_sub(loop_start);
            let mut notes = Vec::new();
            if length_ticks > 0 {
                for note in seq_notes {
                    if note.tick >= loop_start && note.tick < loop_end {
                        let mut new_note = note.clone();
                        new_note.tick = note.tick - loop_start;
                        if new_note.tick + new_note.duration > length_ticks {
                            new_note.duration = length_ticks.saturating_sub(new_note.tick);
                        }
                        if new_note.duration > 0 {
                            notes.push(new_note);
                        }
                    }
                }
            }

            // Capture automation lanes for this instrument within the loop region
            let mut clip_automation_lanes = Vec::new();
            if length_ticks > 0 {
                for lane in state
                    .session
                    .automation
                    .lanes_for_instrument(*instrument_id)
                {
                    let points: Vec<_> = lane
                        .points
                        .iter()
                        .filter(|p| p.tick >= loop_start && p.tick < loop_end)
                        .map(|p| {
                            let mut new_point = p.clone();
                            new_point.tick = p.tick - loop_start;
                            new_point
                        })
                        .collect();
                    if !points.is_empty() {
                        let mut new_lane = lane.clone();
                        new_lane.points = points;
                        new_lane.id = state.session.arrangement.next_clip_lane_id();
                        clip_automation_lanes.push(new_lane);
                    }
                }
            }

            let clip_id = state.session.arrangement.add_clip(
                "Clip".to_string(),
                *instrument_id,
                length_ticks,
            );
            if let Some(clip) = state.session.arrangement.clip_mut(clip_id) {
                clip.name = format!("Clip {}", clip_id);
                clip.notes = notes;
                clip.automation_lanes = clip_automation_lanes;
            }
            DispatchResult::none()
        }
        ArrangementAction::DeleteClip(clip_id) => {
            state.session.arrangement.remove_clip(*clip_id);
            let mut result = DispatchResult::none();
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
            result.audio_effects.push(AudioEffect::UpdateAutomation);
            result
        }
        ArrangementAction::RenameClip(clip_id, name) => {
            if let Some(clip) = state.session.arrangement.clip_mut(*clip_id) {
                clip.name = name.clone();
            }
            DispatchResult::none()
        }
        ArrangementAction::AddClipInstance {
            clip_id,
            instrument_id,
            start_tick,
        } => {
            state
                .session
                .arrangement
                .add_placement(*clip_id, *instrument_id, *start_tick);
            let mut result = DispatchResult::none();
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
            result.audio_effects.push(AudioEffect::UpdateAutomation);
            result
        }
        ArrangementAction::RemoveClipInstance(clip_instance_id) => {
            state
                .session
                .arrangement
                .remove_placement(*clip_instance_id);
            let mut result = DispatchResult::none();
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
            result.audio_effects.push(AudioEffect::UpdateAutomation);
            result
        }
        ArrangementAction::MoveClipInstance {
            clip_instance_id,
            new_start_tick,
        } => {
            state
                .session
                .arrangement
                .move_placement(*clip_instance_id, *new_start_tick);
            let mut result = DispatchResult::none();
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
            result.audio_effects.push(AudioEffect::UpdateAutomation);
            result
        }
        ArrangementAction::ResizeClipInstance {
            clip_instance_id,
            new_length,
        } => {
            state
                .session
                .arrangement
                .resize_placement(*clip_instance_id, *new_length);
            let mut result = DispatchResult::none();
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
            result.audio_effects.push(AudioEffect::UpdateAutomation);
            result
        }
        ArrangementAction::DuplicateClipInstance(clip_instance_id) => {
            let original = state
                .session
                .arrangement
                .placements
                .iter()
                .find(|p| p.id == *clip_instance_id)
                .cloned();

            if let Some(original) = original {
                if let Some(clip) = state.session.arrangement.clip(original.clip_id) {
                    let new_start = original.end_tick(clip);
                    let new_id = state.session.arrangement.add_placement(
                        original.clip_id,
                        original.instrument_id,
                        new_start,
                    );
                    if let Some(new_placement) = state
                        .session
                        .arrangement
                        .placements
                        .iter_mut()
                        .find(|p| p.id == new_id)
                    {
                        new_placement.length_override = original.length_override;
                    }
                }
            }
            let mut result = DispatchResult::none();
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
            result.audio_effects.push(AudioEffect::UpdateAutomation);
            result
        }
        ArrangementAction::SelectClipInstance(selection) => {
            state.session.arrangement.selected_placement = *selection;
            DispatchResult::none()
        }
        ArrangementAction::SelectLane(lane) => {
            let max_lane = state.tracks.tracks.len().saturating_sub(1);
            state.session.arrangement.selected_lane = (*lane).min(max_lane);
            DispatchResult::none()
        }
        ArrangementAction::MoveCursor(delta_cols) => {
            let arr = &mut state.session.arrangement;
            let delta_ticks = *delta_cols as i64 * arr.ticks_per_col as i64;
            let new_tick = (arr.cursor_tick as i64 + delta_ticks).max(0) as u32;
            arr.cursor_tick = new_tick;
            DispatchResult::none()
        }
        ArrangementAction::ScrollView(delta) => {
            let arr = &mut state.session.arrangement;
            let delta_ticks = *delta as i64 * arr.ticks_per_col as i64;
            let new_start = (arr.view_start_tick as i64 + delta_ticks).max(0) as u32;
            arr.view_start_tick = new_start;
            DispatchResult::none()
        }
        ArrangementAction::ZoomIn => {
            let arr = &mut state.session.arrangement;
            arr.ticks_per_col = (arr.ticks_per_col / 2).max(30);
            DispatchResult::none()
        }
        ArrangementAction::ZoomOut => {
            let arr = &mut state.session.arrangement;
            arr.ticks_per_col = (arr.ticks_per_col * 2).min(1920);
            DispatchResult::none()
        }
        ArrangementAction::EnterClipEdit(clip_id) => {
            let clip = match state.session.arrangement.clip(*clip_id).cloned() {
                Some(clip) => clip,
                None => return DispatchResult::none(),
            };

            let (stashed_notes, stashed_loop_start, stashed_loop_end, stashed_looping) = {
                let pr = &mut state.session.piano_roll;
                let seq = match pr.sequences.get_mut(&clip.instrument_id) {
                    Some(seq) => seq,
                    None => return DispatchResult::none(),
                };
                let stashed_notes = seq.notes.clone();
                let stashed_loop_start = pr.loop_start;
                let stashed_loop_end = pr.loop_end;
                let stashed_looping = pr.looping;

                seq.notes = clip.notes.clone();
                pr.loop_start = 0;
                pr.loop_end = clip.length_ticks;
                pr.looping = true;
                state.audio.playhead = 0;

                (
                    stashed_notes,
                    stashed_loop_start,
                    stashed_loop_end,
                    stashed_looping,
                )
            };

            // Stash session automation lanes for this instrument, then load clip automation
            let stashed_automation_lanes: Vec<_> = state
                .session
                .automation
                .lanes
                .iter()
                .filter(|l| l.target.instrument_id() == Some(clip.instrument_id))
                .cloned()
                .collect();
            let stashed_selected_automation_lane = state.session.automation.selected_lane;

            state
                .session
                .automation
                .remove_lanes_for_instrument(clip.instrument_id);

            // Load clip automation lanes into session
            for clip_lane in &clip.automation_lanes {
                let lane_id = state.session.automation.add_lane(clip_lane.target.clone());
                if let Some(session_lane) = state.session.automation.lane_mut(lane_id) {
                    session_lane.points = clip_lane.points.clone();
                    session_lane.enabled = clip_lane.enabled;
                    session_lane.record_armed = clip_lane.record_armed;
                }
            }

            state.session.arrangement.editing_clip = Some(ClipEditContext {
                clip_id: clip.id,
                instrument_id: clip.instrument_id,
                stashed_notes,
                stashed_loop_start,
                stashed_loop_end,
                stashed_looping,
                stashed_automation_lanes,
                stashed_selected_automation_lane,
            });

            let mut result = DispatchResult::with_nav(NavIntent::PushTo(PaneId::PianoRoll));
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
            result.audio_effects.push(AudioEffect::UpdateAutomation);
            result
        }
        ArrangementAction::ExitClipEdit => {
            let ctx = match state.session.arrangement.editing_clip.take() {
                Some(ctx) => ctx,
                None => return DispatchResult::none(),
            };

            let (edited_notes, loop_end) = {
                let pr = &state.session.piano_roll;
                let notes = pr
                    .sequences
                    .get(&ctx.instrument_id)
                    .map(|t| t.notes.clone())
                    .unwrap_or_default();
                (notes, pr.loop_end)
            };

            // Save session automation lanes for this instrument back into the clip
            let edited_automation_lanes: Vec<_> = state
                .session
                .automation
                .lanes
                .iter()
                .filter(|l| l.target.instrument_id() == Some(ctx.instrument_id))
                .cloned()
                .collect();

            if let Some(clip) = state.session.arrangement.clip_mut(ctx.clip_id) {
                clip.notes = edited_notes;
                clip.length_ticks = loop_end;
                clip.automation_lanes = edited_automation_lanes;
            }

            // Remove clip automation lanes from session and restore stashed lanes
            state
                .session
                .automation
                .remove_lanes_for_instrument(ctx.instrument_id);

            for stashed_lane in &ctx.stashed_automation_lanes {
                let lane_id = state
                    .session
                    .automation
                    .add_lane(stashed_lane.target.clone());
                if let Some(session_lane) = state.session.automation.lane_mut(lane_id) {
                    session_lane.points = stashed_lane.points.clone();
                    session_lane.enabled = stashed_lane.enabled;
                    session_lane.record_armed = stashed_lane.record_armed;
                }
            }
            state.session.automation.selected_lane = ctx.stashed_selected_automation_lane;

            {
                let pr = &mut state.session.piano_roll;
                if let Some(seq) = pr.sequences.get_mut(&ctx.instrument_id) {
                    seq.notes = ctx.stashed_notes;
                }
                pr.loop_start = ctx.stashed_loop_start;
                pr.loop_end = ctx.stashed_loop_end;
                pr.looping = ctx.stashed_looping;
            }

            let mut result = DispatchResult::with_nav(NavIntent::PopOrSwitchTo(PaneId::TrackList));
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
            result.audio_effects.push(AudioEffect::UpdateAutomation);
            result
        }
        ArrangementAction::TogglePlayback => {
            let pr = &mut state.session.piano_roll;
            pr.playing = !pr.playing;
            state.audio.playing = pr.playing;
            audio.set_playing(pr.playing);
            if !pr.playing {
                state.audio.playhead = 0;
                audio.reset_playhead();
                if audio.is_running() {
                    audio.release_all_voices();
                }
                audio.clear_active_notes();
            }
            pr.recording = false;
            DispatchResult::none()
        }
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use crate::state::arrangement::PlayMode;
    use crate::state::SourceType;
    use imbolc_audio::AudioHandle;
    use imbolc_types::state::piano_roll::Note;
    use imbolc_types::TrackId;

    fn setup() -> (AppState, AudioHandle) {
        let mut state = AppState::new();
        state.add_track(SourceType::Saw);
        (state, AudioHandle::new())
    }

    fn first_instrument_id(state: &AppState) -> TrackId {
        state.tracks.tracks[0].id
    }

    // ── 1. TogglePlayMode ─────────────────────────────────────────────

    #[test]
    fn toggle_play_mode_pattern_to_song() {
        let (mut state, mut audio) = setup();
        assert_eq!(state.session.arrangement.play_mode, PlayMode::Pattern);

        let result =
            dispatch_arrangement(&ArrangementAction::TogglePlayMode, &mut state, &mut audio);
        assert_eq!(state.session.arrangement.play_mode, PlayMode::Song);
        assert!(result.audio_effects.contains(&AudioEffect::UpdatePianoRoll));
    }

    #[test]
    fn toggle_play_mode_song_to_pattern() {
        let (mut state, mut audio) = setup();
        state.session.arrangement.play_mode = PlayMode::Song;

        dispatch_arrangement(&ArrangementAction::TogglePlayMode, &mut state, &mut audio);
        assert_eq!(state.session.arrangement.play_mode, PlayMode::Pattern);
    }

    #[test]
    fn toggle_play_mode_roundtrip() {
        let (mut state, mut audio) = setup();
        dispatch_arrangement(&ArrangementAction::TogglePlayMode, &mut state, &mut audio);
        dispatch_arrangement(&ArrangementAction::TogglePlayMode, &mut state, &mut audio);
        assert_eq!(state.session.arrangement.play_mode, PlayMode::Pattern);
    }

    // ── 2. CreateClip ─────────────────────────────────────────────────

    #[test]
    fn create_clip_adds_clip() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);
        assert!(state.session.arrangement.clips.is_empty());

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );

        assert_eq!(state.session.arrangement.clips.len(), 1);
        let clip = &state.session.arrangement.clips[0];
        assert_eq!(clip.instrument_id, inst_id);
        assert_eq!(clip.length_ticks, 384);
        assert!(clip.notes.is_empty());
    }

    #[test]
    fn create_clip_names_with_id() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );

        let clip = &state.session.arrangement.clips[0];
        assert_eq!(clip.name, format!("Clip {}", clip.id));
    }

    #[test]
    fn create_multiple_clips_increments_ids() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 192,
            },
            &mut state,
            &mut audio,
        );

        assert_eq!(state.session.arrangement.clips.len(), 2);
        assert_ne!(
            state.session.arrangement.clips[0].id,
            state.session.arrangement.clips[1].id
        );
    }

    // ── 3. CaptureClipFromPianoRoll ───────────────────────────────────

    #[test]
    fn capture_clip_from_piano_roll_empty_loop() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        // Loop region 0..0 — no notes captured
        state.session.piano_roll.loop_start = 0;
        state.session.piano_roll.loop_end = 0;

        dispatch_arrangement(
            &ArrangementAction::CaptureClipFromPianoRoll {
                instrument_id: inst_id,
            },
            &mut state,
            &mut audio,
        );

        assert_eq!(state.session.arrangement.clips.len(), 1);
        let clip = &state.session.arrangement.clips[0];
        assert_eq!(clip.length_ticks, 0);
        assert!(clip.notes.is_empty());
    }

    #[test]
    fn capture_clip_from_piano_roll_captures_notes_in_range() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        // Add notes to the piano roll sequence
        if let Some(seq) = state.session.piano_roll.sequences.get_mut(&inst_id) {
            seq.notes.push(Note {
                tick: 0,
                pitch: 60,
                velocity: 100,
                duration: 96,
                probability: 1.0,
            });
            seq.notes.push(Note {
                tick: 192,
                pitch: 64,
                velocity: 80,
                duration: 48,
                probability: 1.0,
            });
            // Note outside loop region
            seq.notes.push(Note {
                tick: 500,
                pitch: 67,
                velocity: 90,
                duration: 48,
                probability: 1.0,
            });
        }

        state.session.piano_roll.loop_start = 0;
        state.session.piano_roll.loop_end = 384;

        dispatch_arrangement(
            &ArrangementAction::CaptureClipFromPianoRoll {
                instrument_id: inst_id,
            },
            &mut state,
            &mut audio,
        );

        assert_eq!(state.session.arrangement.clips.len(), 1);
        let clip = &state.session.arrangement.clips[0];
        assert_eq!(clip.length_ticks, 384);
        // Only the two notes in range 0..384
        assert_eq!(clip.notes.len(), 2);
        assert_eq!(clip.notes[0].tick, 0);
        assert_eq!(clip.notes[0].pitch, 60);
        assert_eq!(clip.notes[1].tick, 192);
        assert_eq!(clip.notes[1].pitch, 64);
    }

    #[test]
    fn capture_clip_offsets_notes_relative_to_loop_start() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        if let Some(seq) = state.session.piano_roll.sequences.get_mut(&inst_id) {
            seq.notes.push(Note {
                tick: 100,
                pitch: 60,
                velocity: 100,
                duration: 48,
                probability: 1.0,
            });
        }

        state.session.piano_roll.loop_start = 96;
        state.session.piano_roll.loop_end = 288;

        dispatch_arrangement(
            &ArrangementAction::CaptureClipFromPianoRoll {
                instrument_id: inst_id,
            },
            &mut state,
            &mut audio,
        );

        let clip = &state.session.arrangement.clips[0];
        assert_eq!(clip.length_ticks, 192); // 288 - 96
        assert_eq!(clip.notes.len(), 1);
        assert_eq!(clip.notes[0].tick, 4); // 100 - 96
    }

    #[test]
    fn capture_clip_clamps_note_duration_to_loop_end() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        if let Some(seq) = state.session.piano_roll.sequences.get_mut(&inst_id) {
            // Note that extends past the loop end
            seq.notes.push(Note {
                tick: 80,
                pitch: 60,
                velocity: 100,
                duration: 200,
                probability: 1.0,
            });
        }

        state.session.piano_roll.loop_start = 0;
        state.session.piano_roll.loop_end = 100;

        dispatch_arrangement(
            &ArrangementAction::CaptureClipFromPianoRoll {
                instrument_id: inst_id,
            },
            &mut state,
            &mut audio,
        );

        let clip = &state.session.arrangement.clips[0];
        assert_eq!(clip.notes.len(), 1);
        // Duration clamped: 100 - 80 = 20
        assert_eq!(clip.notes[0].duration, 20);
    }

    // ── 4. DeleteClip ─────────────────────────────────────────────────

    #[test]
    fn delete_clip_removes_clip() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        let result = dispatch_arrangement(
            &ArrangementAction::DeleteClip(clip_id),
            &mut state,
            &mut audio,
        );

        assert!(state.session.arrangement.clips.is_empty());
        assert!(result.audio_effects.contains(&AudioEffect::UpdatePianoRoll));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::UpdateAutomation));
    }

    #[test]
    fn delete_clip_cascade_removes_placements() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        // Place the clip twice
        dispatch_arrangement(
            &ArrangementAction::AddClipInstance {
                clip_id,
                instrument_id: inst_id,
                start_tick: 0,
            },
            &mut state,
            &mut audio,
        );
        dispatch_arrangement(
            &ArrangementAction::AddClipInstance {
                clip_id,
                instrument_id: inst_id,
                start_tick: 384,
            },
            &mut state,
            &mut audio,
        );
        assert_eq!(state.session.arrangement.placements.len(), 2);

        dispatch_arrangement(
            &ArrangementAction::DeleteClip(clip_id),
            &mut state,
            &mut audio,
        );
        assert!(state.session.arrangement.placements.is_empty());
    }

    // ── 5. RenameClip ─────────────────────────────────────────────────

    #[test]
    fn rename_clip_updates_name() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        dispatch_arrangement(
            &ArrangementAction::RenameClip(clip_id, "My Clip".to_string()),
            &mut state,
            &mut audio,
        );

        assert_eq!(
            state.session.arrangement.clip(clip_id).unwrap().name,
            "My Clip"
        );
    }

    #[test]
    fn rename_nonexistent_clip_is_noop() {
        let (mut state, mut audio) = setup();

        // Should not panic
        dispatch_arrangement(
            &ArrangementAction::RenameClip(9999, "Nope".to_string()),
            &mut state,
            &mut audio,
        );
    }

    // ── 6. PlaceClip / RemovePlacement ────────────────────────────────

    #[test]
    fn place_clip_creates_placement() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        let result = dispatch_arrangement(
            &ArrangementAction::AddClipInstance {
                clip_id,
                instrument_id: inst_id,
                start_tick: 100,
            },
            &mut state,
            &mut audio,
        );

        assert_eq!(state.session.arrangement.placements.len(), 1);
        let p = &state.session.arrangement.placements[0];
        assert_eq!(p.clip_id, clip_id);
        assert_eq!(p.instrument_id, inst_id);
        assert_eq!(p.start_tick, 100);
        assert!(result.audio_effects.contains(&AudioEffect::UpdatePianoRoll));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::UpdateAutomation));
    }

    #[test]
    fn remove_placement_deletes_placement() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        dispatch_arrangement(
            &ArrangementAction::AddClipInstance {
                clip_id,
                instrument_id: inst_id,
                start_tick: 0,
            },
            &mut state,
            &mut audio,
        );
        let clip_instance_id = state.session.arrangement.placements[0].id;

        let result = dispatch_arrangement(
            &ArrangementAction::RemoveClipInstance(clip_instance_id),
            &mut state,
            &mut audio,
        );

        assert!(state.session.arrangement.placements.is_empty());
        assert!(result.audio_effects.contains(&AudioEffect::UpdatePianoRoll));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::UpdateAutomation));
    }

    #[test]
    fn remove_placement_clears_selection() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        dispatch_arrangement(
            &ArrangementAction::AddClipInstance {
                clip_id,
                instrument_id: inst_id,
                start_tick: 0,
            },
            &mut state,
            &mut audio,
        );
        let clip_instance_id = state.session.arrangement.placements[0].id;
        state.session.arrangement.selected_placement = Some(0);

        dispatch_arrangement(
            &ArrangementAction::RemoveClipInstance(clip_instance_id),
            &mut state,
            &mut audio,
        );
        assert_eq!(state.session.arrangement.selected_placement, None);
    }

    // ── 7. MovePlacement / ResizePlacement ────────────────────────────

    #[test]
    fn move_placement_updates_start_tick() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        dispatch_arrangement(
            &ArrangementAction::AddClipInstance {
                clip_id,
                instrument_id: inst_id,
                start_tick: 0,
            },
            &mut state,
            &mut audio,
        );
        let clip_instance_id = state.session.arrangement.placements[0].id;

        let result = dispatch_arrangement(
            &ArrangementAction::MoveClipInstance {
                clip_instance_id,
                new_start_tick: 200,
            },
            &mut state,
            &mut audio,
        );

        assert_eq!(state.session.arrangement.placements[0].start_tick, 200);
        assert!(result.audio_effects.contains(&AudioEffect::UpdatePianoRoll));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::UpdateAutomation));
    }

    #[test]
    fn resize_placement_sets_length_override() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        dispatch_arrangement(
            &ArrangementAction::AddClipInstance {
                clip_id,
                instrument_id: inst_id,
                start_tick: 0,
            },
            &mut state,
            &mut audio,
        );
        let clip_instance_id = state.session.arrangement.placements[0].id;

        let result = dispatch_arrangement(
            &ArrangementAction::ResizeClipInstance {
                clip_instance_id,
                new_length: Some(192),
            },
            &mut state,
            &mut audio,
        );

        assert_eq!(
            state.session.arrangement.placements[0].length_override,
            Some(192)
        );
        assert!(result.audio_effects.contains(&AudioEffect::UpdatePianoRoll));
    }

    #[test]
    fn resize_placement_none_clears_override() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        dispatch_arrangement(
            &ArrangementAction::AddClipInstance {
                clip_id,
                instrument_id: inst_id,
                start_tick: 0,
            },
            &mut state,
            &mut audio,
        );
        let clip_instance_id = state.session.arrangement.placements[0].id;

        // First set an override
        dispatch_arrangement(
            &ArrangementAction::ResizeClipInstance {
                clip_instance_id,
                new_length: Some(192),
            },
            &mut state,
            &mut audio,
        );
        // Then clear it
        dispatch_arrangement(
            &ArrangementAction::ResizeClipInstance {
                clip_instance_id,
                new_length: None,
            },
            &mut state,
            &mut audio,
        );

        assert_eq!(
            state.session.arrangement.placements[0].length_override,
            None
        );
    }

    // ── 8. DuplicatePlacement ─────────────────────────────────────────

    #[test]
    fn duplicate_placement_creates_adjacent_copy() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        dispatch_arrangement(
            &ArrangementAction::AddClipInstance {
                clip_id,
                instrument_id: inst_id,
                start_tick: 0,
            },
            &mut state,
            &mut audio,
        );
        let clip_instance_id = state.session.arrangement.placements[0].id;

        let result = dispatch_arrangement(
            &ArrangementAction::DuplicateClipInstance(clip_instance_id),
            &mut state,
            &mut audio,
        );

        assert_eq!(state.session.arrangement.placements.len(), 2);
        let original = &state.session.arrangement.placements[0];
        let duplicate = &state.session.arrangement.placements[1];
        assert_eq!(original.clip_id, duplicate.clip_id);
        assert_eq!(duplicate.start_tick, 384); // placed right after original
        assert!(result.audio_effects.contains(&AudioEffect::UpdatePianoRoll));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::UpdateAutomation));
    }

    #[test]
    fn duplicate_placement_preserves_length_override() {
        let (mut state, mut audio) = setup();
        let inst_id = first_instrument_id(&state);

        dispatch_arrangement(
            &ArrangementAction::CreateClip {
                instrument_id: inst_id,
                length_ticks: 384,
            },
            &mut state,
            &mut audio,
        );
        let clip_id = state.session.arrangement.clips[0].id;

        dispatch_arrangement(
            &ArrangementAction::AddClipInstance {
                clip_id,
                instrument_id: inst_id,
                start_tick: 0,
            },
            &mut state,
            &mut audio,
        );
        let clip_instance_id = state.session.arrangement.placements[0].id;

        // Set a length override, then duplicate
        dispatch_arrangement(
            &ArrangementAction::ResizeClipInstance {
                clip_instance_id,
                new_length: Some(200),
            },
            &mut state,
            &mut audio,
        );
        dispatch_arrangement(
            &ArrangementAction::DuplicateClipInstance(clip_instance_id),
            &mut state,
            &mut audio,
        );

        let duplicate = &state.session.arrangement.placements[1];
        assert_eq!(duplicate.length_override, Some(200));
        // Duplicate starts at end of original: 0 + 200 (effective length with override)
        assert_eq!(duplicate.start_tick, 200);
    }

    #[test]
    fn duplicate_nonexistent_placement_is_noop() {
        let (mut state, mut audio) = setup();

        dispatch_arrangement(
            &ArrangementAction::DuplicateClipInstance(9999),
            &mut state,
            &mut audio,
        );
        assert!(state.session.arrangement.placements.is_empty());
    }

    // ── 9. SelectPlacement / SelectLane ───────────────────────────────

    #[test]
    fn select_placement_sets_selection() {
        let (mut state, mut audio) = setup();

        dispatch_arrangement(
            &ArrangementAction::SelectClipInstance(Some(3)),
            &mut state,
            &mut audio,
        );
        assert_eq!(state.session.arrangement.selected_placement, Some(3));

        dispatch_arrangement(
            &ArrangementAction::SelectClipInstance(None),
            &mut state,
            &mut audio,
        );
        assert_eq!(state.session.arrangement.selected_placement, None);
    }

    #[test]
    fn select_lane_clamps_to_instrument_count() {
        let (mut state, mut audio) = setup();
        // 1 instrument → max lane is 0
        dispatch_arrangement(&ArrangementAction::SelectLane(5), &mut state, &mut audio);
        assert_eq!(state.session.arrangement.selected_lane, 0);
    }

    #[test]
    fn select_lane_allows_valid_index() {
        let (mut state, mut audio) = setup();
        // Add a second instrument
        state.add_track(SourceType::Sin);
        // 2 instruments → lanes 0 and 1 valid
        dispatch_arrangement(&ArrangementAction::SelectLane(1), &mut state, &mut audio);
        assert_eq!(state.session.arrangement.selected_lane, 1);
    }

    // ── 10. MoveCursor / ScrollView / ZoomIn / ZoomOut ────────────────

    #[test]
    fn move_cursor_positive() {
        let (mut state, mut audio) = setup();
        state.session.arrangement.cursor_tick = 0;
        state.session.arrangement.ticks_per_col = 120;

        dispatch_arrangement(&ArrangementAction::MoveCursor(2), &mut state, &mut audio);
        assert_eq!(state.session.arrangement.cursor_tick, 240); // 2 * 120
    }

    #[test]
    fn move_cursor_negative_clamped_at_zero() {
        let (mut state, mut audio) = setup();
        state.session.arrangement.cursor_tick = 100;
        state.session.arrangement.ticks_per_col = 120;

        dispatch_arrangement(&ArrangementAction::MoveCursor(-2), &mut state, &mut audio);
        // (100 - 240) clamped to 0
        assert_eq!(state.session.arrangement.cursor_tick, 0);
    }

    #[test]
    fn scroll_view_positive() {
        let (mut state, mut audio) = setup();
        state.session.arrangement.view_start_tick = 0;
        state.session.arrangement.ticks_per_col = 120;

        dispatch_arrangement(&ArrangementAction::ScrollView(3), &mut state, &mut audio);
        assert_eq!(state.session.arrangement.view_start_tick, 360); // 3 * 120
    }

    #[test]
    fn scroll_view_negative_clamped_at_zero() {
        let (mut state, mut audio) = setup();
        state.session.arrangement.view_start_tick = 50;
        state.session.arrangement.ticks_per_col = 120;

        dispatch_arrangement(&ArrangementAction::ScrollView(-1), &mut state, &mut audio);
        assert_eq!(state.session.arrangement.view_start_tick, 0);
    }

    #[test]
    fn zoom_in_halves_ticks_per_col() {
        let (mut state, mut audio) = setup();
        state.session.arrangement.ticks_per_col = 240;

        dispatch_arrangement(&ArrangementAction::ZoomIn, &mut state, &mut audio);
        assert_eq!(state.session.arrangement.ticks_per_col, 120);
    }

    #[test]
    fn zoom_in_clamped_at_minimum() {
        let (mut state, mut audio) = setup();
        state.session.arrangement.ticks_per_col = 30;

        dispatch_arrangement(&ArrangementAction::ZoomIn, &mut state, &mut audio);
        // 30 / 2 = 15, but min is 30
        assert_eq!(state.session.arrangement.ticks_per_col, 30);
    }

    #[test]
    fn zoom_out_doubles_ticks_per_col() {
        let (mut state, mut audio) = setup();
        state.session.arrangement.ticks_per_col = 120;

        dispatch_arrangement(&ArrangementAction::ZoomOut, &mut state, &mut audio);
        assert_eq!(state.session.arrangement.ticks_per_col, 240);
    }

    #[test]
    fn zoom_out_clamped_at_maximum() {
        let (mut state, mut audio) = setup();
        state.session.arrangement.ticks_per_col = 1920;

        dispatch_arrangement(&ArrangementAction::ZoomOut, &mut state, &mut audio);
        assert_eq!(state.session.arrangement.ticks_per_col, 1920);
    }

    #[test]
    fn zoom_roundtrip() {
        let (mut state, mut audio) = setup();
        let original = state.session.arrangement.ticks_per_col;

        dispatch_arrangement(&ArrangementAction::ZoomIn, &mut state, &mut audio);
        dispatch_arrangement(&ArrangementAction::ZoomOut, &mut state, &mut audio);
        assert_eq!(state.session.arrangement.ticks_per_col, original);
    }
}
