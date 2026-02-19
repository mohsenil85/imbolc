//! Arrangement and clip types for timeline-based composition.

use std::collections::HashMap;
use std::path::PathBuf;

use super::automation::{AutomationLane, AutomationPoint, AutomationTarget};
use super::piano_roll::Note;
use crate::{AudioClipId, AutomationLaneId, ClipId, ClipInstanceId, TrackId};
use serde::{Deserialize, Serialize};

/// An audio clip represents recorded audio from an input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioClip {
    pub id: AudioClipId,
    pub name: String,
    /// Path to the WAV file on disk
    pub path: PathBuf,
    /// Start tick of the recording on the timeline
    pub start_tick: u32,
    /// Duration in ticks
    pub duration_ticks: u32,
    /// Associated instrument (track) ID
    pub instrument_id: TrackId,
    /// Waveform peaks for display (normalized 0-1)
    pub waveform_peaks: Vec<f32>,
    /// Original sample rate of the recording
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlayMode {
    #[default]
    Pattern,
    Song,
}

/// Reusable pattern of notes for a single instrument.
/// Notes use tick positions relative to clip start (0-based).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: ClipId,
    pub name: String,
    pub instrument_id: TrackId,
    pub length_ticks: u32,
    pub notes: Vec<Note>,
    /// Per-clip automation lanes (0-based tick positions, like notes)
    #[serde(default)]
    pub automation_lanes: Vec<AutomationLane>,
}

/// A placement of a clip on the timeline. Multiple placements can share a clip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipInstance {
    pub id: ClipInstanceId,
    pub clip_id: ClipId,
    pub instrument_id: TrackId,
    pub start_tick: u32,              // Absolute position on timeline
    pub length_override: Option<u32>, // Trim shorter than clip, None = use clip.length_ticks
}

impl ClipInstance {
    pub fn effective_length(&self, clip: &Clip) -> u32 {
        self.length_override.unwrap_or(clip.length_ticks)
    }

    pub fn end_tick(&self, clip: &Clip) -> u32 {
        self.start_tick + self.effective_length(clip)
    }
}

/// Saved context when editing a clip in the piano roll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipEditContext {
    pub clip_id: ClipId,
    pub instrument_id: TrackId,
    pub stashed_notes: Vec<Note>, // Original piano roll track notes
    pub stashed_loop_start: u32,
    pub stashed_loop_end: u32,
    pub stashed_looping: bool,
    /// Session automation lanes for this instrument, stashed on clip edit enter
    #[serde(default)]
    pub stashed_automation_lanes: Vec<AutomationLane>,
    /// Session selected_lane, stashed on clip edit enter
    #[serde(default)]
    pub stashed_selected_automation_lane: Option<usize>,
}

/// Top-level arrangement state. Owned by SessionState.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrangementState {
    pub clips: Vec<Clip>,
    pub placements: Vec<ClipInstance>,
    pub play_mode: PlayMode,
    #[serde(skip)]
    pub editing_clip: Option<ClipEditContext>,

    // UI state (persisted)
    pub selected_placement: Option<usize>, // Index into placements vec
    pub selected_lane: usize,
    pub view_start_tick: u32,
    pub ticks_per_col: u32, // Zoom: ticks per terminal column (default 120)
    pub cursor_tick: u32,

    pub(crate) next_clip_id: ClipId,
    pub(crate) next_placement_id: ClipInstanceId,
    pub(crate) next_clip_automation_lane_id: AutomationLaneId,
}

impl Default for ArrangementState {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrangementState {
    pub fn new() -> Self {
        Self {
            clips: Vec::new(),
            placements: Vec::new(),
            play_mode: PlayMode::default(),
            editing_clip: None,
            selected_placement: None,
            selected_lane: 0,
            view_start_tick: 0,
            ticks_per_col: 120,
            cursor_tick: 0,
            next_clip_id: ClipId::new(1),
            next_placement_id: ClipInstanceId::new(1),
            next_clip_automation_lane_id: AutomationLaneId::new(0),
        }
    }

    pub fn add_clip(&mut self, name: String, instrument_id: TrackId, length_ticks: u32) -> ClipId {
        let id = self.next_clip_id;
        self.next_clip_id = id.next();
        self.clips.push(Clip {
            id,
            name,
            instrument_id,
            length_ticks,
            notes: Vec::new(),
            automation_lanes: Vec::new(),
        });
        id
    }

    pub fn clip(&self, id: ClipId) -> Option<&Clip> {
        self.clips.iter().find(|c| c.id == id)
    }

    pub fn clip_mut(&mut self, id: ClipId) -> Option<&mut Clip> {
        self.clips.iter_mut().find(|c| c.id == id)
    }

    pub fn remove_clip(&mut self, id: ClipId) {
        if let Some(pos) = self.clips.iter().position(|c| c.id == id) {
            self.clips.remove(pos);
            // Cascade delete placements
            self.placements.retain(|p| p.clip_id != id);
            // Clear selection if it was a placement of this clip (simplified: just clear selection)
            self.selected_placement = None;
        }
    }

    pub fn clips_for_instrument(&self, instrument_id: TrackId) -> Vec<&Clip> {
        self.clips
            .iter()
            .filter(|c| c.instrument_id == instrument_id)
            .collect()
    }

    pub fn add_placement(
        &mut self,
        clip_id: ClipId,
        instrument_id: TrackId,
        start_tick: u32,
    ) -> ClipInstanceId {
        let id = self.next_placement_id;
        self.next_placement_id = id.next();
        self.placements.push(ClipInstance {
            id,
            clip_id,
            instrument_id,
            start_tick,
            length_override: None,
        });
        id
    }

    pub fn remove_placement(&mut self, id: ClipInstanceId) {
        if let Some(pos) = self.placements.iter().position(|p| p.id == id) {
            self.placements.remove(pos);
            self.selected_placement = None;
        }
    }

    pub fn move_placement(&mut self, id: ClipInstanceId, new_start_tick: u32) {
        if let Some(p) = self.placements.iter_mut().find(|p| p.id == id) {
            p.start_tick = new_start_tick;
        }
    }

    pub fn resize_placement(&mut self, id: ClipInstanceId, new_length: Option<u32>) {
        if let Some(p) = self.placements.iter_mut().find(|p| p.id == id) {
            p.length_override = new_length;
        }
    }

    pub fn placements_for_instrument(&self, instrument_id: TrackId) -> Vec<&ClipInstance> {
        let mut placements: Vec<&ClipInstance> = self
            .placements
            .iter()
            .filter(|p| p.instrument_id == instrument_id)
            .collect();
        placements.sort_by_key(|p| p.start_tick);
        placements
    }

    pub fn placement_at(&self, instrument_id: TrackId, tick: u32) -> Option<&ClipInstance> {
        for placement in self.placements_for_instrument(instrument_id) {
            if let Some(clip) = self.clip(placement.clip_id) {
                if tick >= placement.start_tick && tick < placement.end_tick(clip) {
                    return Some(placement);
                }
            }
        }
        None
    }

    pub fn flatten_to_notes(&self) -> HashMap<TrackId, Vec<Note>> {
        let mut result: HashMap<TrackId, Vec<Note>> = HashMap::new();

        // Sort placements by start_tick to ensure somewhat ordered output, though we sort at the end anyway
        let mut sorted_placements: Vec<&ClipInstance> = self.placements.iter().collect();
        sorted_placements.sort_by_key(|p| p.start_tick);

        for placement in sorted_placements {
            if let Some(clip) = self.clip(placement.clip_id) {
                let effective_len = placement.effective_length(clip);
                let notes_entry = result.entry(placement.instrument_id).or_default();

                for note in &clip.notes {
                    if note.tick < effective_len {
                        let new_tick = placement.start_tick + note.tick;
                        let mut new_duration = note.duration;

                        // Clamp duration if it extends past effective length
                        if note.tick + new_duration > effective_len {
                            new_duration = effective_len - note.tick;
                        }

                        if new_duration > 0 {
                            let mut new_note = note.clone();
                            new_note.tick = new_tick;
                            new_note.duration = new_duration;
                            notes_entry.push(new_note);
                        }
                    }
                }
            }
        }

        // Sort notes by tick for each instrument
        for notes in result.values_mut() {
            notes.sort_by_key(|n| n.tick);
        }

        result
    }

    pub fn arrangement_length(&self) -> u32 {
        let mut max_end = 0;
        for placement in &self.placements {
            if let Some(clip) = self.clip(placement.clip_id) {
                max_end = max_end.max(placement.end_tick(clip));
            }
        }
        max_end
    }

    /// Flatten per-clip automation into absolute-tick lanes for Song mode playback.
    /// Merges lanes from all placements by AutomationTarget; same-tick conflicts
    /// resolve in placement order (later placement wins via dedup).
    pub fn flatten_automation(&self) -> Vec<AutomationLane> {
        let mut merged: HashMap<AutomationTarget, Vec<AutomationPoint>> = HashMap::new();
        // Track min/max per target so merged lanes keep correct ranges
        let mut ranges: HashMap<AutomationTarget, (f32, f32)> = HashMap::new();

        let mut sorted_placements: Vec<&ClipInstance> = self.placements.iter().collect();
        sorted_placements.sort_by_key(|p| p.start_tick);

        for placement in sorted_placements {
            if let Some(clip) = self.clip(placement.clip_id) {
                let effective_len = placement.effective_length(clip);

                for lane in &clip.automation_lanes {
                    let points = merged.entry(lane.target.clone()).or_default();
                    ranges
                        .entry(lane.target.clone())
                        .or_insert((lane.min_value, lane.max_value));

                    for point in &lane.points {
                        if point.tick < effective_len {
                            let mut new_point = point.clone();
                            new_point.tick = placement.start_tick + point.tick;
                            points.push(new_point);
                        }
                    }
                }
            }
        }

        let mut result = Vec::new();
        for (target, mut points) in merged {
            points.sort_by_key(|p| p.tick);
            // Dedup same-tick: last wins (later placement overwrites)
            points.dedup_by_key(|p| p.tick);

            let (min_value, max_value) = ranges
                .get(&target)
                .copied()
                .unwrap_or_else(|| target.default_range());
            result.push(AutomationLane {
                id: AutomationLaneId::new(0), // IDs don't matter for flattened playback snapshot
                target,
                points,
                enabled: true,
                record_armed: false,
                min_value,
                max_value,
            });
        }

        result
    }

    pub fn remove_instrument_data(&mut self, instrument_id: TrackId) {
        let clip_ids_to_remove: Vec<ClipId> = self
            .clips
            .iter()
            .filter(|c| c.instrument_id == instrument_id)
            .map(|c| c.id)
            .collect();

        self.clips.retain(|c| c.instrument_id != instrument_id);

        self.placements.retain(|p| {
            p.instrument_id != instrument_id && !clip_ids_to_remove.contains(&p.clip_id)
        });

        self.selected_placement = None;
    }

    /// Get the next clip ID counter (for persistence)
    pub fn next_clip_id(&self) -> ClipId {
        self.next_clip_id
    }

    /// Get the next placement ID counter (for persistence)
    pub fn next_placement_id(&self) -> ClipInstanceId {
        self.next_placement_id
    }

    /// Get the next clip automation lane ID counter (for persistence)
    pub fn next_clip_automation_lane_id(&self) -> AutomationLaneId {
        self.next_clip_automation_lane_id
    }

    pub fn recalculate_next_ids(&mut self) {
        self.next_clip_id = self
            .clips
            .iter()
            .map(|c| c.id)
            .max()
            .map_or(ClipId::new(0), |m| m.next());
        self.next_placement_id = self
            .placements
            .iter()
            .map(|p| p.id)
            .max()
            .map_or(ClipInstanceId::new(0), |m| m.next());
        self.next_clip_automation_lane_id = self
            .clips
            .iter()
            .flat_map(|c| c.automation_lanes.iter().map(|l| l.id))
            .max()
            .map_or(AutomationLaneId::new(0), |m| m.next());
    }

    /// Allocate a fresh automation lane ID for use in clips
    pub fn next_clip_lane_id(&mut self) -> AutomationLaneId {
        let id = self.next_clip_automation_lane_id;
        self.next_clip_automation_lane_id = id.next();
        id
    }

    /// Find the clip whose placement contains `tick`, or the nearest placement by distance.
    /// Returns the clip ID (not placement ID) for use with clip edit.
    pub fn nearest_clip_for_instrument(&self, instrument_id: TrackId, tick: u32) -> Option<ClipId> {
        let placements = self.placements_for_instrument(instrument_id);
        if placements.is_empty() {
            return None;
        }

        // First: check if tick falls inside any placement
        for p in &placements {
            if let Some(clip) = self.clip(p.clip_id) {
                if tick >= p.start_tick && tick < p.end_tick(clip) {
                    return Some(p.clip_id);
                }
            }
        }

        // Otherwise: find nearest by distance to placement start
        placements
            .iter()
            .min_by_key(|p| {
                let dist_start = (p.start_tick as i64 - tick as i64).unsigned_abs();
                let dist_end = self
                    .clip(p.clip_id)
                    .map(|c| (p.end_tick(c) as i64 - tick as i64).unsigned_abs())
                    .unwrap_or(u64::MAX);
                dist_start.min(dist_end)
            })
            .map(|p| p.clip_id)
    }

    /// Next distinct clip by timeline order for the given instrument.
    /// Finds the first placement after the current clip's placements.
    pub fn next_clip_for_instrument(
        &self,
        instrument_id: TrackId,
        current_clip_id: ClipId,
    ) -> Option<ClipId> {
        let placements = self.placements_for_instrument(instrument_id);
        // Find the last placement of the current clip
        let current_end = placements
            .iter()
            .filter(|p| p.clip_id == current_clip_id)
            .filter_map(|p| self.clip(p.clip_id).map(|c| p.end_tick(c)))
            .max()
            .unwrap_or(0);

        // Find the first placement that starts at or after current_end with a different clip_id
        placements
            .iter()
            .find(|p| p.clip_id != current_clip_id && p.start_tick >= current_end)
            .map(|p| p.clip_id)
            .or_else(|| {
                // Also check for any clip after the current one in timeline order
                placements
                    .iter()
                    .filter(|p| p.clip_id != current_clip_id)
                    .find(|p| {
                        p.start_tick
                            > placements
                                .iter()
                                .filter(|pp| pp.clip_id == current_clip_id)
                                .map(|pp| pp.start_tick)
                                .min()
                                .unwrap_or(0)
                    })
                    .map(|p| p.clip_id)
            })
    }

    /// Previous distinct clip by timeline order for the given instrument.
    pub fn prev_clip_for_instrument(
        &self,
        instrument_id: TrackId,
        current_clip_id: ClipId,
    ) -> Option<ClipId> {
        let placements = self.placements_for_instrument(instrument_id);
        // Find the first placement of the current clip
        let current_start = placements
            .iter()
            .filter(|p| p.clip_id == current_clip_id)
            .map(|p| p.start_tick)
            .min()
            .unwrap_or(0);

        // Find the last placement before current_start with a different clip_id
        placements
            .iter()
            .rev()
            .find(|p| p.clip_id != current_clip_id && p.start_tick < current_start)
            .map(|p| p.clip_id)
    }

    /// Create a clip and place it atomically. Returns the new clip ID.
    pub fn create_and_place_clip(
        &mut self,
        instrument_id: TrackId,
        length: u32,
        start_tick: u32,
    ) -> ClipId {
        let clip_count = self.clips_for_instrument(instrument_id).len();
        let clip_id = self.add_clip(format!("Clip {}", clip_count + 1), instrument_id, length);
        self.add_placement(clip_id, instrument_id, start_tick);
        clip_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TrackId;

    #[test]
    fn test_add_remove_clip() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 384);
        assert!(arr.clip(cid).is_some());
        assert_eq!(arr.clip(cid).unwrap().name, "Test");

        arr.remove_clip(cid);
        assert!(arr.clip(cid).is_none());
    }

    #[test]
    fn test_add_remove_placement() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 384);
        let pid = arr.add_placement(cid, TrackId::new(1), 0);

        assert_eq!(arr.placements.len(), 1);
        arr.remove_placement(pid);
        assert_eq!(arr.placements.len(), 0);
    }

    #[test]
    fn test_cascade_delete() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 384);
        arr.add_placement(cid, TrackId::new(1), 0);
        arr.add_placement(cid, TrackId::new(1), 384);

        assert_eq!(arr.placements.len(), 2);
        arr.remove_clip(cid);
        assert_eq!(arr.placements.len(), 0);
    }

    #[test]
    fn test_remove_instrument_data() {
        let mut arr = ArrangementState::new();
        let cid1 = arr.add_clip("Inst1".to_string(), TrackId::new(1), 384);
        let cid2 = arr.add_clip("Inst2".to_string(), TrackId::new(2), 384);

        arr.add_placement(cid1, TrackId::new(1), 0);
        arr.add_placement(cid2, TrackId::new(2), 0);

        arr.remove_instrument_data(TrackId::new(1));

        assert!(arr.clip(cid1).is_none());
        assert!(arr.clip(cid2).is_some());
        assert_eq!(arr.placements.len(), 1);
        assert_eq!(arr.placements[0].instrument_id, TrackId::new(2));
    }

    #[test]
    fn test_flatten_to_notes() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 384);

        if let Some(clip) = arr.clip_mut(cid) {
            clip.notes.push(Note {
                tick: 0,
                pitch: 60,
                velocity: 100,
                duration: 48,
                probability: 1.0,
            });
            clip.notes.push(Note {
                tick: 96,
                pitch: 62,
                velocity: 100,
                duration: 48,
                probability: 1.0,
            });
        }

        // Placement 1: Start at 0
        arr.add_placement(cid, TrackId::new(1), 0);
        // Placement 2: Start at 384
        arr.add_placement(cid, TrackId::new(1), 384);

        let flat = arr.flatten_to_notes();
        let notes = flat.get(&TrackId::new(1)).unwrap();

        assert_eq!(notes.len(), 4);
        assert_eq!(notes[0].tick, 0);
        assert_eq!(notes[0].pitch, 60);
        assert_eq!(notes[1].tick, 96);
        assert_eq!(notes[2].tick, 384);
        assert_eq!(notes[3].tick, 384 + 96);
    }

    #[test]
    fn test_flatten_with_override_and_clamp() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 100);

        if let Some(clip) = arr.clip_mut(cid) {
            // Note at 0, duration 50
            clip.notes.push(Note {
                tick: 0,
                pitch: 60,
                velocity: 100,
                duration: 50,
                probability: 1.0,
            });
            // Note at 60, duration 50 (extends past 100)
            clip.notes.push(Note {
                tick: 60,
                pitch: 62,
                velocity: 100,
                duration: 50,
                probability: 1.0,
            });
        }

        let pid = arr.add_placement(cid, TrackId::new(1), 0);
        arr.resize_placement(pid, Some(80)); // Trim to 80

        let flat = arr.flatten_to_notes();
        let notes = flat.get(&TrackId::new(1)).unwrap();

        // Should have note at 0 (full duration 50 < 80)
        // Should have note at 60 (clamped duration: 80 - 60 = 20)
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].duration, 50);
        assert_eq!(notes[1].tick, 60);
        assert_eq!(notes[1].duration, 20);
    }

    #[test]
    fn test_arrangement_length() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 120);

        arr.add_placement(cid, TrackId::new(1), 0);
        arr.add_placement(cid, TrackId::new(1), 240);

        assert_eq!(arr.arrangement_length(), 360);
    }

    #[test]
    fn test_placement_at() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 100);
        arr.add_placement(cid, TrackId::new(1), 20);

        assert!(arr.placement_at(TrackId::new(1), 10).is_none());
        assert!(arr.placement_at(TrackId::new(1), 20).is_some());
        assert!(arr.placement_at(TrackId::new(1), 50).is_some());
        assert!(arr.placement_at(TrackId::new(1), 120).is_none());
    }

    #[test]
    fn test_flatten_automation_single_clip() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 384);

        if let Some(clip) = arr.clip_mut(cid) {
            let mut lane = AutomationLane::new(
                AutomationLaneId::new(0),
                AutomationTarget::level(TrackId::new(1)),
            );
            lane.points.push(AutomationPoint::new(0, 0.0));
            lane.points.push(AutomationPoint::new(192, 1.0));
            clip.automation_lanes.push(lane);
        }

        arr.add_placement(cid, TrackId::new(1), 100);

        let flat = arr.flatten_automation();
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].target, AutomationTarget::level(TrackId::new(1)));
        assert_eq!(flat[0].points.len(), 2);
        assert_eq!(flat[0].points[0].tick, 100);
        assert_eq!(flat[0].points[1].tick, 292);
    }

    #[test]
    fn test_flatten_automation_multiple_placements() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 200);

        if let Some(clip) = arr.clip_mut(cid) {
            let mut lane = AutomationLane::new(
                AutomationLaneId::new(0),
                AutomationTarget::filter_cutoff(TrackId::new(1)),
            );
            lane.points.push(AutomationPoint::new(0, 0.5));
            lane.points.push(AutomationPoint::new(100, 1.0));
            clip.automation_lanes.push(lane);
        }

        arr.add_placement(cid, TrackId::new(1), 0);
        arr.add_placement(cid, TrackId::new(1), 500);

        let flat = arr.flatten_automation();
        assert_eq!(flat.len(), 1);
        let lane = &flat[0];
        assert_eq!(lane.points.len(), 4);
        assert_eq!(lane.points[0].tick, 0);
        assert_eq!(lane.points[1].tick, 100);
        assert_eq!(lane.points[2].tick, 500);
        assert_eq!(lane.points[3].tick, 600);
    }

    #[test]
    fn test_flatten_automation_empty_clip() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 384);
        arr.add_placement(cid, TrackId::new(1), 0);

        let flat = arr.flatten_automation();
        assert!(flat.is_empty());
    }

    #[test]
    fn test_flatten_automation_overlapping_dedup() {
        let mut arr = ArrangementState::new();
        // Two clips with same target, placed so they produce points at the same tick
        let cid1 = arr.add_clip("A".to_string(), TrackId::new(1), 100);
        let cid2 = arr.add_clip("B".to_string(), TrackId::new(1), 100);

        if let Some(clip) = arr.clip_mut(cid1) {
            let mut lane = AutomationLane::new(
                AutomationLaneId::new(0),
                AutomationTarget::level(TrackId::new(1)),
            );
            lane.points.push(AutomationPoint::new(50, 0.2));
            clip.automation_lanes.push(lane);
        }
        if let Some(clip) = arr.clip_mut(cid2) {
            let mut lane = AutomationLane::new(
                AutomationLaneId::new(1),
                AutomationTarget::level(TrackId::new(1)),
            );
            lane.points.push(AutomationPoint::new(0, 0.8));
            clip.automation_lanes.push(lane);
        }

        // Place clip1 at tick 0 → point at tick 50
        // Place clip2 at tick 50 → point at tick 50 (conflict!)
        arr.add_placement(cid1, TrackId::new(1), 0);
        arr.add_placement(cid2, TrackId::new(1), 50);

        let flat = arr.flatten_automation();
        assert_eq!(flat.len(), 1);
        // dedup_by_key keeps the first occurrence — clip1's point at tick 50 (value 0.2)
        // comes first in sorted order, clip2's point at tick 50 (value 0.8) comes second
        // dedup_by_key keeps the first, removes duplicates
        let points_at_50: Vec<_> = flat[0].points.iter().filter(|p| p.tick == 50).collect();
        assert_eq!(points_at_50.len(), 1);
    }

    // ── nearest/next/prev/create clip helpers ──────────────────────

    #[test]
    fn test_nearest_clip_inside_placement() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("A".to_string(), TrackId::new(1), 200);
        arr.add_placement(cid, TrackId::new(1), 100);

        assert_eq!(
            arr.nearest_clip_for_instrument(TrackId::new(1), 150),
            Some(cid)
        );
    }

    #[test]
    fn test_nearest_clip_before_placement() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("A".to_string(), TrackId::new(1), 200);
        arr.add_placement(cid, TrackId::new(1), 100);

        // tick 50 is before the placement at 100; nearest should still find it
        assert_eq!(
            arr.nearest_clip_for_instrument(TrackId::new(1), 50),
            Some(cid)
        );
    }

    #[test]
    fn test_nearest_clip_no_clips() {
        let arr = ArrangementState::new();
        assert_eq!(arr.nearest_clip_for_instrument(TrackId::new(1), 0), None);
    }

    #[test]
    fn test_nearest_clip_picks_closer() {
        let mut arr = ArrangementState::new();
        let cid1 = arr.add_clip("A".to_string(), TrackId::new(1), 100);
        let cid2 = arr.add_clip("B".to_string(), TrackId::new(1), 100);
        arr.add_placement(cid1, TrackId::new(1), 0);
        arr.add_placement(cid2, TrackId::new(1), 500);

        // tick 400 is closer to clip2 at 500 than clip1 ending at 100
        assert_eq!(
            arr.nearest_clip_for_instrument(TrackId::new(1), 400),
            Some(cid2)
        );
        // tick 50 is inside clip1
        assert_eq!(
            arr.nearest_clip_for_instrument(TrackId::new(1), 50),
            Some(cid1)
        );
    }

    #[test]
    fn test_next_clip_for_instrument() {
        let mut arr = ArrangementState::new();
        let cid1 = arr.add_clip("A".to_string(), TrackId::new(1), 100);
        let cid2 = arr.add_clip("B".to_string(), TrackId::new(1), 100);
        arr.add_placement(cid1, TrackId::new(1), 0);
        arr.add_placement(cid2, TrackId::new(1), 200);

        assert_eq!(
            arr.next_clip_for_instrument(TrackId::new(1), cid1),
            Some(cid2)
        );
    }

    #[test]
    fn test_next_clip_at_last_returns_none() {
        let mut arr = ArrangementState::new();
        let cid1 = arr.add_clip("A".to_string(), TrackId::new(1), 100);
        arr.add_placement(cid1, TrackId::new(1), 0);

        assert_eq!(arr.next_clip_for_instrument(TrackId::new(1), cid1), None);
    }

    #[test]
    fn test_prev_clip_for_instrument() {
        let mut arr = ArrangementState::new();
        let cid1 = arr.add_clip("A".to_string(), TrackId::new(1), 100);
        let cid2 = arr.add_clip("B".to_string(), TrackId::new(1), 100);
        arr.add_placement(cid1, TrackId::new(1), 0);
        arr.add_placement(cid2, TrackId::new(1), 200);

        assert_eq!(
            arr.prev_clip_for_instrument(TrackId::new(1), cid2),
            Some(cid1)
        );
    }

    #[test]
    fn test_prev_clip_at_first_returns_none() {
        let mut arr = ArrangementState::new();
        let cid1 = arr.add_clip("A".to_string(), TrackId::new(1), 100);
        arr.add_placement(cid1, TrackId::new(1), 0);

        assert_eq!(arr.prev_clip_for_instrument(TrackId::new(1), cid1), None);
    }

    #[test]
    fn test_create_and_place_clip() {
        let mut arr = ArrangementState::new();
        let cid = arr.create_and_place_clip(TrackId::new(1), 384, 0);

        assert_eq!(arr.clips.len(), 1);
        assert_eq!(arr.placements.len(), 1);
        assert_eq!(arr.clip(cid).unwrap().name, "Clip 1");
        assert_eq!(arr.clip(cid).unwrap().length_ticks, 384);
        assert_eq!(arr.placements[0].start_tick, 0);
        assert_eq!(arr.placements[0].clip_id, cid);
    }

    #[test]
    fn test_create_and_place_clip_increments_name() {
        let mut arr = ArrangementState::new();
        let cid1 = arr.create_and_place_clip(TrackId::new(1), 384, 0);
        let cid2 = arr.create_and_place_clip(TrackId::new(1), 384, 384);

        assert_eq!(arr.clip(cid1).unwrap().name, "Clip 1");
        assert_eq!(arr.clip(cid2).unwrap().name, "Clip 2");
    }

    #[test]
    fn test_flatten_automation_respects_effective_length() {
        let mut arr = ArrangementState::new();
        let cid = arr.add_clip("Test".to_string(), TrackId::new(1), 200);

        if let Some(clip) = arr.clip_mut(cid) {
            let mut lane = AutomationLane::new(
                AutomationLaneId::new(0),
                AutomationTarget::pan(TrackId::new(1)),
            );
            lane.points.push(AutomationPoint::new(0, 0.0));
            lane.points.push(AutomationPoint::new(50, 0.5));
            lane.points.push(AutomationPoint::new(150, 1.0)); // Past trim point
            clip.automation_lanes.push(lane);
        }

        let pid = arr.add_placement(cid, TrackId::new(1), 0);
        arr.resize_placement(pid, Some(100)); // Trim to 100

        let flat = arr.flatten_automation();
        assert_eq!(flat.len(), 1);
        // Point at tick 150 should be excluded (past effective_length of 100)
        assert_eq!(flat[0].points.len(), 2);
        assert_eq!(flat[0].points[0].tick, 0);
        assert_eq!(flat[0].points[1].tick, 50);
    }
}
