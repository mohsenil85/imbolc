//! Track collection state.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::drum_sequencer::DrumSequencerState;
use super::track::{SourceType, Track};
use crate::{BufferId, GroupId, SampleId, TrackId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackState {
    pub tracks: Vec<Track>,
    pub selected: Option<usize>,
    pub next_id: TrackId,
    #[serde(default = "default_sampler_buffer_id")]
    pub next_sampler_buffer_id: BufferId,
    /// Counter for allocating sample blob IDs
    #[serde(default)]
    pub next_sample_id: SampleId,
    /// Set by dispatch when editing a track; read by TrackEditPane on_enter
    #[serde(skip)]
    pub editing_track_id: Option<TrackId>,
    /// Counter for allocating layer group IDs
    pub next_layer_group_id: GroupId,
    /// Index from TrackId → Vec position for O(1) lookups.
    #[serde(skip)]
    id_index: HashMap<TrackId, usize>,
}

impl TrackState {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            selected: None,
            next_id: TrackId::new(0),
            next_sampler_buffer_id: BufferId::new(20000),
            next_sample_id: SampleId::default(),
            editing_track_id: None,
            next_layer_group_id: GroupId::new(0),
            id_index: HashMap::new(),
        }
    }

    /// Rebuild the id → index lookup table from the Vec.
    ///
    /// Call after any operation that replaces `tracks` wholesale
    /// (e.g. undo/redo, persistence load, network state replacement).
    pub fn rebuild_index(&mut self) {
        self.id_index.clear();
        for (i, inst) in self.tracks.iter().enumerate() {
            self.id_index.insert(inst.id, i);
        }
    }

    pub fn add_track(&mut self, source: SourceType) -> TrackId {
        let id = self.next_id;
        self.next_id = TrackId::new(self.next_id.get() + 1);
        let track = Track::new(id, source);
        self.tracks.push(track);
        self.selected = Some(self.tracks.len() - 1);
        self.id_index.insert(id, self.tracks.len() - 1);

        id
    }

    pub fn remove_track(&mut self, id: TrackId) {
        // Capture layer group before removal for singleton cleanup
        let old_group = self.track(id).and_then(|i| i.layer.group);

        if let Some(pos) = self.tracks.iter().position(|s| s.id == id) {
            self.tracks.remove(pos);

            if let Some(sel) = self.selected {
                if sel >= self.tracks.len() {
                    self.selected = if self.tracks.is_empty() {
                        None
                    } else {
                        Some(self.tracks.len() - 1)
                    };
                }
            }

            // Rebuild index since positions shifted
            self.rebuild_index();
        }

        // If old group now has only 1 member, clear it (group of 1 is meaningless)
        if let Some(g) = old_group {
            let remaining: Vec<TrackId> = self
                .tracks
                .iter()
                .filter(|i| i.layer.group == Some(g))
                .map(|i| i.id)
                .collect();
            if remaining.len() == 1 {
                if let Some(inst) = self.track_mut(remaining[0]) {
                    inst.layer.group = None;
                }
            }
        }
    }

    /// Returns the Vec index of a track by its ID.
    pub fn track_index(&self, id: TrackId) -> Option<usize> {
        if let Some(&idx) = self.id_index.get(&id) {
            if self.tracks.get(idx).map(|t| t.id) == Some(id) {
                return Some(idx);
            }
        }
        self.tracks.iter().position(|t| t.id == id)
    }

    /// Returns the TrackId at a given Vec position.
    pub fn track_id_at(&self, index: usize) -> Option<TrackId> {
        self.tracks.get(index).map(|t| t.id)
    }

    pub fn track(&self, id: TrackId) -> Option<&Track> {
        // Use index for O(1) lookup, fall back to linear scan if index is stale
        if let Some(&idx) = self.id_index.get(&id) {
            if let Some(inst) = self.tracks.get(idx) {
                if inst.id == id {
                    return Some(inst);
                }
            }
        }
        self.tracks.iter().find(|s| s.id == id)
    }

    pub fn track_mut(&mut self, id: TrackId) -> Option<&mut Track> {
        // Use index for O(1) lookup, fall back to linear scan if index is stale
        if let Some(&idx) = self.id_index.get(&id) {
            if let Some(inst) = self.tracks.get(idx) {
                if inst.id == id {
                    return self.tracks.get_mut(idx);
                }
            }
        }
        self.tracks.iter_mut().find(|s| s.id == id)
    }

    pub fn selected_track(&self) -> Option<&Track> {
        self.selected.and_then(|idx| self.tracks.get(idx))
    }

    #[allow(dead_code)]
    pub fn selected_track_mut(&mut self) -> Option<&mut Track> {
        self.selected.and_then(|idx| self.tracks.get_mut(idx))
    }

    pub fn select_next(&mut self) {
        if self.tracks.is_empty() {
            self.selected = None;
            return;
        }
        self.selected = match self.selected {
            None => Some(0),
            Some(idx) if idx < self.tracks.len() - 1 => Some(idx + 1),
            Some(idx) => Some(idx),
        };
    }

    pub fn select_prev(&mut self) {
        if self.tracks.is_empty() {
            self.selected = None;
            return;
        }
        self.selected = match self.selected {
            None => Some(0),
            Some(0) => Some(0),
            Some(idx) => Some(idx - 1),
        };
    }

    /// Check if any track is soloed
    pub fn any_track_solo(&self) -> bool {
        self.tracks.iter().any(|s| s.channel_strip.solo)
    }

    pub fn selected_drum_sequencer(&self) -> Option<&DrumSequencerState> {
        self.selected_track().and_then(|s| s.drum_sequencer())
    }

    pub fn selected_drum_sequencer_mut(&mut self) -> Option<&mut DrumSequencerState> {
        self.selected
            .and_then(|idx| self.tracks.get_mut(idx))
            .and_then(|s| s.drum_sequencer_mut())
    }

    /// Allocate a new unique layer group ID
    pub fn next_layer_group(&mut self) -> GroupId {
        let id = self.next_layer_group_id;
        self.next_layer_group_id = GroupId::new(id.get() + 1);
        id
    }

    /// Returns sorted unique group IDs from tracks that have a layer_group set.
    pub fn active_layer_groups(&self) -> Vec<GroupId> {
        let mut groups: Vec<GroupId> = self.tracks.iter().filter_map(|i| i.layer.group).collect();
        groups.sort_unstable();
        groups.dedup();
        groups
    }

    /// Returns all track IDs in the same layer group as `id` (including `id` itself).
    /// If the track has no layer group, returns just `vec![id]`.
    pub fn layer_group_members(&self, id: TrackId) -> Vec<TrackId> {
        let group = self.track(id).and_then(|inst| inst.layer.group);
        match group {
            Some(g) => self
                .tracks
                .iter()
                .filter(|inst| inst.layer.group == Some(g))
                .map(|inst| inst.id)
                .collect(),
            None => vec![id],
        }
    }
}

impl Default for TrackState {
    fn default() -> Self {
        Self::new()
    }
}

fn default_sampler_buffer_id() -> BufferId {
    BufferId::new(20000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_state_creation() {
        let state = TrackState::new();
        assert_eq!(state.tracks.len(), 0);
        assert_eq!(state.selected, None);
    }

    #[test]
    fn test_add_track() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);
        let id2 = state.add_track(SourceType::Sin);

        assert_eq!(state.tracks.len(), 2);
        assert_eq!(state.tracks[0].id, id1);
        assert_eq!(state.tracks[1].id, id2);
        assert_eq!(state.selected, Some(1)); // selects newly added
    }

    #[test]
    fn test_remove_track() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);
        let id2 = state.add_track(SourceType::Sin);
        let _id3 = state.add_track(SourceType::Sqr);

        state.remove_track(id2);

        assert_eq!(state.tracks.len(), 2);
        assert_eq!(state.tracks[0].id, id1);
    }

    #[test]
    fn test_remove_last_track() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);
        let id2 = state.add_track(SourceType::Sin);

        state.selected = Some(1);
        state.remove_track(id2);

        assert_eq!(state.selected, Some(0));
        assert_eq!(state.tracks[0].id, id1);
    }

    #[test]
    fn test_remove_all_tracks() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);

        state.remove_track(id1);
        assert_eq!(state.selected, None);
        assert!(state.tracks.is_empty());
    }

    #[test]
    fn test_select_navigation() {
        let mut state = TrackState::new();
        state.add_track(SourceType::Saw);
        state.add_track(SourceType::Sin);
        state.add_track(SourceType::Sqr);

        assert_eq!(state.selected, Some(2)); // selects last added
        state.select_prev();
        assert_eq!(state.selected, Some(1));
        state.select_prev();
        assert_eq!(state.selected, Some(0));
        state.select_prev();
        assert_eq!(state.selected, Some(0)); // stay at start
        state.select_next();
        assert_eq!(state.selected, Some(1));
        state.select_next();
        assert_eq!(state.selected, Some(2));
        state.select_next();
        assert_eq!(state.selected, Some(2)); // stay at end
    }

    #[test]
    fn layer_group_members_returns_all_in_group() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);
        let id2 = state.add_track(SourceType::Sin);
        let _id3 = state.add_track(SourceType::Sqr);

        let group = state.next_layer_group();
        state.track_mut(id1).unwrap().layer.group = Some(group);
        state.track_mut(id2).unwrap().layer.group = Some(group);

        let members = state.layer_group_members(id1);
        assert_eq!(members.len(), 2);
        assert!(members.contains(&id1));
        assert!(members.contains(&id2));
    }

    #[test]
    fn layer_group_members_solo_no_group() {
        let mut state = TrackState::new();
        let id = state.add_track(SourceType::Saw);
        let members = state.layer_group_members(id);
        assert_eq!(members, vec![id]);
    }

    #[test]
    fn remove_track_clears_singleton_group() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);
        let id2 = state.add_track(SourceType::Sin);

        let group = state.next_layer_group();
        state.track_mut(id1).unwrap().layer.group = Some(group);
        state.track_mut(id2).unwrap().layer.group = Some(group);

        state.remove_track(id1);

        // id2 should have layer_group cleared (group of 1 is meaningless)
        assert_eq!(state.track(id2).unwrap().layer.group, None);
    }

    #[test]
    fn select_next_wraps_at_boundary() {
        let mut state = TrackState::new();
        state.add_track(SourceType::Saw);
        state.add_track(SourceType::Sin);
        state.selected = Some(1);
        state.select_next();
        assert_eq!(state.selected, Some(1)); // stays at end, does not wrap
    }

    #[test]
    fn select_prev_wraps_at_boundary() {
        let mut state = TrackState::new();
        state.add_track(SourceType::Saw);
        state.add_track(SourceType::Sin);
        state.selected = Some(0);
        state.select_prev();
        assert_eq!(state.selected, Some(0)); // stays at start, does not wrap
    }

    #[test]
    fn index_correct_after_add() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);
        let id2 = state.add_track(SourceType::Sin);
        let id3 = state.add_track(SourceType::Sqr);

        assert!(state.track(id1).is_some());
        assert!(state.track(id2).is_some());
        assert!(state.track(id3).is_some());
        assert_eq!(state.track(id1).unwrap().id, id1);
        assert_eq!(state.track(id2).unwrap().id, id2);
        assert_eq!(state.track(id3).unwrap().id, id3);
    }

    #[test]
    fn index_correct_after_remove() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);
        let id2 = state.add_track(SourceType::Sin);
        let id3 = state.add_track(SourceType::Sqr);

        state.remove_track(id2);

        assert!(state.track(id1).is_some());
        assert!(state.track(id2).is_none());
        assert!(state.track(id3).is_some());
        assert_eq!(state.track(id1).unwrap().id, id1);
        assert_eq!(state.track(id3).unwrap().id, id3);
    }

    #[test]
    fn fallback_works_when_index_empty() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);
        let id2 = state.add_track(SourceType::Sin);

        // Clear the index to simulate a deserialized state without rebuild_index
        state.id_index.clear();

        // Should still find tracks via linear fallback
        assert_eq!(state.track(id1).unwrap().id, id1);
        assert_eq!(state.track(id2).unwrap().id, id2);
        assert_eq!(state.track_mut(id1).unwrap().id, id1);
    }

    #[test]
    fn rebuild_index_restores_lookups() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);
        let id2 = state.add_track(SourceType::Sin);

        // Simulate whole-struct replacement by clearing and rebuilding
        state.id_index.clear();
        state.rebuild_index();

        assert_eq!(state.track(id1).unwrap().id, id1);
        assert_eq!(state.track(id2).unwrap().id, id2);
    }

    #[test]
    fn clone_preserves_index() {
        let mut state = TrackState::new();
        let id1 = state.add_track(SourceType::Saw);
        let id2 = state.add_track(SourceType::Sin);

        let cloned = state.clone();
        assert_eq!(cloned.track(id1).unwrap().id, id1);
        assert_eq!(cloned.track(id2).unwrap().id, id2);
    }
}
