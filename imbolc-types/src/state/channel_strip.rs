use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::track::{
    decode_effect_cursor_from_slice, deserialize_sends, effects_max_cursor, ChannelConfig,
    EffectSlot, EffectType, EqConfig, FilterConfig, FilterType, MixerSend, NoteEffectSlot,
    NoteEffectType, OutputTarget, ProcessingStage,
};
use crate::{BusId, EffectId, ParamIndex};

/// Unified channel strip: level, pan, mute, solo, routing, processing chain.
///
/// Shared by `Track`, `MixerBus`, and `GroupMixer` so that mixer
/// operations (level/pan/mute/solo, effect CRUD, EQ, filter) are written once.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStrip {
    pub level: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub active: bool,
    pub output_target: OutputTarget,
    #[serde(default)]
    pub channel_config: ChannelConfig,
    #[serde(deserialize_with = "deserialize_sends")]
    pub sends: BTreeMap<BusId, MixerSend>,
    #[serde(default)]
    pub processing_chain: Vec<ProcessingStage>,
    #[serde(default)]
    pub next_effect_id: EffectId,
}

impl ChannelStrip {
    /// Create an instrument-style channel strip (with active flag, no effects).
    pub fn new_instrument(active: bool) -> Self {
        Self {
            level: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            active,
            output_target: OutputTarget::Master,
            channel_config: ChannelConfig::default(),
            sends: BTreeMap::new(),
            processing_chain: Vec::new(),
            next_effect_id: EffectId::new(0),
        }
    }

    /// Create a bus-style channel strip (always active, no sends/routing).
    pub fn new_bus() -> Self {
        Self {
            level: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            active: true,
            output_target: OutputTarget::Master,
            channel_config: ChannelConfig::default(),
            sends: BTreeMap::new(),
            processing_chain: Vec::new(),
            next_effect_id: EffectId::new(0),
        }
    }

    /// Create a layer-group-style channel strip (always active, with default EQ).
    pub fn new_layer_group() -> Self {
        let mut cs = Self {
            level: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            active: true,
            output_target: OutputTarget::Master,
            channel_config: ChannelConfig::default(),
            sends: BTreeMap::new(),
            processing_chain: Vec::new(),
            next_effect_id: EffectId::new(0),
        };
        cs.add_eq();
        cs
    }

    /// Disable sends for a removed bus (keeps the entry for undo support).
    pub fn disable_send_for_bus(&mut self, bus_id: BusId) {
        if let Some(send) = self.sends.get_mut(&bus_id) {
            send.enabled = false;
        }
    }

    // --- Processing chain read accessors ---

    /// Get the first filter in the processing chain.
    pub fn filter(&self) -> Option<&FilterConfig> {
        self.processing_chain.iter().find_map(|s| match s {
            ProcessingStage::Filter(f) => Some(f),
            _ => None,
        })
    }

    /// Get the first filter mutably.
    pub fn filter_mut(&mut self) -> Option<&mut FilterConfig> {
        self.processing_chain.iter_mut().find_map(|s| match s {
            ProcessingStage::Filter(f) => Some(f),
            _ => None,
        })
    }

    /// Get all filters in the processing chain.
    pub fn filters(&self) -> impl Iterator<Item = &FilterConfig> {
        self.processing_chain.iter().filter_map(|s| match s {
            ProcessingStage::Filter(f) => Some(f),
            _ => None,
        })
    }

    /// Get all filters mutably.
    pub fn filters_mut(&mut self) -> impl Iterator<Item = &mut FilterConfig> {
        self.processing_chain.iter_mut().filter_map(|s| match s {
            ProcessingStage::Filter(f) => Some(f),
            _ => None,
        })
    }

    /// Get the first EQ config.
    pub fn eq(&self) -> Option<&EqConfig> {
        self.processing_chain.iter().find_map(|s| match s {
            ProcessingStage::Eq(_, eq) => Some(eq),
            _ => None,
        })
    }

    /// Get the first EQ config mutably.
    pub fn eq_mut(&mut self) -> Option<&mut EqConfig> {
        self.processing_chain.iter_mut().find_map(|s| match s {
            ProcessingStage::Eq(_, eq) => Some(eq),
            _ => None,
        })
    }

    /// Get the EffectId of the first EQ, if present.
    pub fn first_eq_id(&self) -> Option<EffectId> {
        self.processing_chain.iter().find_map(|s| s.eq_id())
    }

    /// Get an EQ config by its EffectId.
    pub fn eq_by_id(&self, id: EffectId) -> Option<&EqConfig> {
        self.processing_chain.iter().find_map(|s| match s {
            ProcessingStage::Eq(eid, eq) if *eid == id => Some(eq),
            _ => None,
        })
    }

    /// Get a mutable EQ config by its EffectId.
    pub fn eq_by_id_mut(&mut self, id: EffectId) -> Option<&mut EqConfig> {
        self.processing_chain.iter_mut().find_map(|s| match s {
            ProcessingStage::Eq(eid, eq) if *eid == id => Some(eq),
            _ => None,
        })
    }

    /// Iterate over all EQs as (EffectId, &EqConfig).
    pub fn eqs(&self) -> impl Iterator<Item = (EffectId, &EqConfig)> {
        self.processing_chain.iter().filter_map(|s| match s {
            ProcessingStage::Eq(id, eq) => Some((*id, eq)),
            _ => None,
        })
    }

    /// Chain index of a specific EQ by its EffectId.
    pub fn eq_chain_index_by_id(&self, id: EffectId) -> Option<usize> {
        self.processing_chain
            .iter()
            .position(|s| matches!(s, ProcessingStage::Eq(eid, _) if *eid == id))
    }

    /// Check whether any EQ is present.
    pub fn has_eq(&self) -> bool {
        self.processing_chain.iter().any(|s| s.is_eq())
    }

    /// Add a new EQ to the chain (after last filter, before effects). Returns its EffectId.
    pub fn add_eq(&mut self) -> EffectId {
        let id = self.next_effect_id;
        self.next_effect_id = EffectId::new(self.next_effect_id.get() + 1);
        let insert_at = self
            .processing_chain
            .iter()
            .rposition(|s| s.is_filter())
            .map(|i| i + 1)
            .unwrap_or(0);
        self.processing_chain
            .insert(insert_at, ProcessingStage::Eq(id, EqConfig::default()));
        id
    }

    /// Remove an EQ by its EffectId. Returns true if removed.
    pub fn remove_eq(&mut self, id: EffectId) -> bool {
        if let Some(idx) = self.eq_chain_index_by_id(id) {
            self.processing_chain.remove(idx);
            true
        } else {
            false
        }
    }

    /// Toggle first EQ: remove if present, or add new one after last filter.
    pub fn toggle_eq(&mut self) {
        if let Some(idx) = self.eq_chain_index() {
            self.processing_chain.remove(idx);
        } else {
            self.add_eq();
        }
    }

    /// Get all effects in the processing chain.
    pub fn effects(&self) -> impl Iterator<Item = &EffectSlot> {
        self.processing_chain.iter().filter_map(|s| match s {
            ProcessingStage::Effect(e) => Some(e),
            _ => None,
        })
    }

    /// Get all effects mutably.
    pub fn effects_mut(&mut self) -> impl Iterator<Item = &mut EffectSlot> {
        self.processing_chain.iter_mut().filter_map(|s| match s {
            ProcessingStage::Effect(e) => Some(e),
            _ => None,
        })
    }

    /// Collect effects into a Vec (convenience for code that needs a slice).
    pub fn effects_vec(&self) -> Vec<&EffectSlot> {
        self.effects().collect()
    }

    /// Find an effect by its stable EffectId.
    pub fn effect_by_id(&self, id: EffectId) -> Option<&EffectSlot> {
        self.effects().find(|e| e.id == id)
    }

    /// Find a mutable effect by its stable EffectId.
    pub fn effect_by_id_mut(&mut self, id: EffectId) -> Option<&mut EffectSlot> {
        self.effects_mut().find(|e| e.id == id)
    }

    /// Get the position of an effect among effects only (not chain index).
    pub fn effect_position(&self, id: EffectId) -> Option<usize> {
        self.effects().position(|e| e.id == id)
    }

    // --- Index queries into the full chain ---

    /// Chain index of the first filter.
    pub fn filter_chain_index(&self) -> Option<usize> {
        self.processing_chain.iter().position(|s| s.is_filter())
    }

    /// Chain index of the first EQ.
    pub fn eq_chain_index(&self) -> Option<usize> {
        self.processing_chain.iter().position(|s| s.is_eq())
    }

    /// Chain index of an effect by its EffectId.
    pub fn effect_chain_index(&self, id: EffectId) -> Option<usize> {
        self.processing_chain
            .iter()
            .position(|s| matches!(s, ProcessingStage::Effect(e) if e.id == id))
    }

    // --- Mutation helpers ---

    /// Toggle filter: remove first filter if present, or insert Lpf at index 0.
    pub fn toggle_filter(&mut self) {
        if let Some(idx) = self.filter_chain_index() {
            self.processing_chain.remove(idx);
        } else {
            self.processing_chain.insert(
                0,
                ProcessingStage::Filter(FilterConfig::new(FilterType::Lpf)),
            );
        }
    }

    /// Set filter type. None removes; Some replaces or inserts at index 0.
    pub fn set_filter(&mut self, filter_type: Option<FilterType>) {
        match filter_type {
            None => {
                if let Some(idx) = self.filter_chain_index() {
                    self.processing_chain.remove(idx);
                }
            }
            Some(ft) => {
                if let Some(idx) = self.filter_chain_index() {
                    self.processing_chain[idx] = ProcessingStage::Filter(FilterConfig::new(ft));
                } else {
                    self.processing_chain
                        .insert(0, ProcessingStage::Filter(FilterConfig::new(ft)));
                }
            }
        }
    }

    /// Add an effect to the end of the chain. Returns its stable EffectId.
    pub fn add_effect(&mut self, effect_type: EffectType) -> EffectId {
        let id = self.next_effect_id;
        self.next_effect_id = EffectId::new(self.next_effect_id.get() + 1);
        self.processing_chain
            .push(ProcessingStage::Effect(EffectSlot::new(id, effect_type)));
        id
    }

    /// Remove an effect by its EffectId. Returns true if removed.
    pub fn remove_effect(&mut self, id: EffectId) -> bool {
        if let Some(idx) = self.effect_chain_index(id) {
            self.processing_chain.remove(idx);
            true
        } else {
            false
        }
    }

    /// Move an effect up/down by its EffectId (convenience for effect-only chains).
    pub fn move_effect(&mut self, id: EffectId, direction: i8) -> bool {
        if let Some(idx) = self.effect_chain_index(id) {
            self.move_stage(idx, direction)
        } else {
            false
        }
    }

    /// Move any stage within the processing chain by chain index.
    pub fn move_stage(&mut self, idx: usize, direction: i8) -> bool {
        if idx >= self.processing_chain.len() {
            return false;
        }
        let new_idx = (idx as i64 + direction as i64).max(0) as usize;
        if new_idx >= self.processing_chain.len() || new_idx == idx {
            return false;
        }
        self.processing_chain.swap(idx, new_idx);
        true
    }

    /// Decode a flat cursor position over just the effects in the chain.
    pub fn decode_effect_cursor(&self, cursor: usize) -> Option<(EffectId, Option<ParamIndex>)> {
        let effects: Vec<_> = self.effects().cloned().collect();
        decode_effect_cursor_from_slice(&effects, cursor)
    }

    /// Max cursor position for the effects in this channel strip.
    pub fn effects_max_cursor(&self) -> usize {
        let effects: Vec<_> = self.effects().cloned().collect();
        effects_max_cursor(&effects)
    }

    // --- Note effect accessors ---

    /// Get all note effects in the processing chain.
    pub fn note_effects(&self) -> impl Iterator<Item = &NoteEffectSlot> {
        self.processing_chain.iter().filter_map(|s| match s {
            ProcessingStage::NoteEffect(ne) => Some(ne),
            _ => None,
        })
    }

    /// Get all note effects mutably.
    pub fn note_effects_mut(&mut self) -> impl Iterator<Item = &mut NoteEffectSlot> {
        self.processing_chain.iter_mut().filter_map(|s| match s {
            ProcessingStage::NoteEffect(ne) => Some(ne),
            _ => None,
        })
    }

    /// Find a note effect by its EffectId.
    pub fn note_effect_by_id(&self, id: EffectId) -> Option<&NoteEffectSlot> {
        self.note_effects().find(|ne| ne.id == id)
    }

    /// Find a mutable note effect by its EffectId.
    pub fn note_effect_by_id_mut(&mut self, id: EffectId) -> Option<&mut NoteEffectSlot> {
        self.note_effects_mut().find(|ne| ne.id == id)
    }

    /// Chain index of a note effect by its EffectId.
    pub fn note_effect_chain_index(&self, id: EffectId) -> Option<usize> {
        self.processing_chain
            .iter()
            .position(|s| matches!(s, ProcessingStage::NoteEffect(ne) if ne.id == id))
    }

    /// Add a note effect to the start of the chain (before audio effects).
    /// Returns its stable EffectId.
    pub fn add_note_effect(&mut self, effect_type: NoteEffectType) -> EffectId {
        let id = self.next_effect_id;
        self.next_effect_id = EffectId::new(self.next_effect_id.get() + 1);
        // Insert at position 0 (note effects go before audio processing)
        self.processing_chain.insert(
            0,
            ProcessingStage::NoteEffect(NoteEffectSlot::new(id, effect_type)),
        );
        id
    }

    /// Remove a note effect by its EffectId. Returns true if removed.
    pub fn remove_note_effect(&mut self, id: EffectId) -> bool {
        if let Some(idx) = self.note_effect_chain_index(id) {
            self.processing_chain.remove(idx);
            true
        } else {
            false
        }
    }

    /// Recalculate next_effect_id from existing effects, EQs, and note effects in the chain (used after loading).
    pub fn recalculate_next_effect_id(&mut self) {
        let max_effect = self.effects().map(|e| e.id.get()).max();
        let max_eq = self.eqs().map(|(id, _)| id.get()).max();
        let max_note_effect = self.note_effects().map(|ne| ne.id.get()).max();
        let max_id = max_effect
            .into_iter()
            .chain(max_eq)
            .chain(max_note_effect)
            .max();
        self.next_effect_id = max_id.map_or(EffectId::new(0), |m| EffectId::new(m + 1));
    }
}
