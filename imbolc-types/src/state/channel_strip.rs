use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::instrument::{
    decode_effect_cursor_from_slice, deserialize_sends, effects_max_cursor, ChannelConfig,
    EffectSlot, EffectType, EqConfig, FilterConfig, FilterType, MixerSend, OutputTarget,
    ProcessingStage,
};
use crate::{BusId, EffectId, ParamIndex};

/// Unified channel strip: level, pan, mute, solo, routing, processing chain.
///
/// Shared by `Instrument`, `MixerBus`, and `LayerGroupMixer` so that mixer
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
        Self {
            level: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            active: true,
            output_target: OutputTarget::Master,
            channel_config: ChannelConfig::default(),
            sends: BTreeMap::new(),
            processing_chain: vec![ProcessingStage::Eq(EqConfig::default())],
            next_effect_id: EffectId::new(0),
        }
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

    /// Get the EQ config (single instance).
    pub fn eq(&self) -> Option<&EqConfig> {
        self.processing_chain.iter().find_map(|s| match s {
            ProcessingStage::Eq(eq) => Some(eq),
            _ => None,
        })
    }

    /// Get the EQ config mutably.
    pub fn eq_mut(&mut self) -> Option<&mut EqConfig> {
        self.processing_chain.iter_mut().find_map(|s| match s {
            ProcessingStage::Eq(eq) => Some(eq),
            _ => None,
        })
    }

    /// Check whether an EQ is present.
    pub fn has_eq(&self) -> bool {
        self.processing_chain.iter().any(|s| s.is_eq())
    }

    /// Toggle EQ: remove if present, or insert after last filter.
    pub fn toggle_eq(&mut self) {
        if let Some(idx) = self.eq_chain_index() {
            self.processing_chain.remove(idx);
        } else {
            let insert_at = self
                .processing_chain
                .iter()
                .rposition(|s| s.is_filter())
                .map(|i| i + 1)
                .unwrap_or(0);
            self.processing_chain
                .insert(insert_at, ProcessingStage::Eq(EqConfig::default()));
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

    /// Chain index of the EQ.
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

    /// Recalculate next_effect_id from existing effects in the chain (used after loading).
    pub fn recalculate_next_effect_id(&mut self) {
        self.next_effect_id = self
            .effects()
            .map(|e| e.id.get())
            .max()
            .map_or(EffectId::new(0), |m| EffectId::new(m + 1));
    }
}
