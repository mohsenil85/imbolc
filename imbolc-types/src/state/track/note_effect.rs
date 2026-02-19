use serde::{Deserialize, Serialize};

use crate::state::arpeggiator::GlideRate;
use crate::{EffectId, Param, ParamValue};

/// Types of note effects that can appear in the processing chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteEffectType {
    /// No retrigger, voice reuse + pitch glide (portamento)
    Legato,
    /// Shorten gate duration
    Staccato,
}

impl NoteEffectType {
    pub fn name(&self) -> &'static str {
        match self {
            NoteEffectType::Legato => "Legato",
            NoteEffectType::Staccato => "Staccato",
        }
    }

    /// Default parameters for this note effect type.
    pub fn default_params(&self) -> Vec<Param> {
        match self {
            NoteEffectType::Legato => vec![
                Param {
                    name: "sync".to_string(),
                    value: ParamValue::Bool(true),
                    min: 0.0,
                    max: 1.0,
                },
                Param {
                    name: "rate".to_string(),
                    value: ParamValue::Int(2), // 0=1/4, 1=1/8, 2=1/16, 3=1/32
                    min: 0.0,
                    max: 3.0,
                },
                Param {
                    name: "time".to_string(),
                    value: ParamValue::Float(0.1),
                    min: 0.01,
                    max: 1.0,
                },
            ],
            NoteEffectType::Staccato => vec![Param {
                name: "amount".to_string(),
                value: ParamValue::Float(0.5),
                min: 0.1,
                max: 1.0,
            }],
        }
    }

    /// All available note effect types for the add-effect menu.
    pub const ALL: &'static [NoteEffectType] = &[NoteEffectType::Legato, NoteEffectType::Staccato];
}

/// A note effect slot in the processing chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteEffectSlot {
    pub id: EffectId,
    pub effect_type: NoteEffectType,
    pub params: Vec<Param>,
    pub enabled: bool,
}

impl NoteEffectSlot {
    pub fn new(id: EffectId, effect_type: NoteEffectType) -> Self {
        Self {
            id,
            effect_type,
            params: effect_type.default_params(),
            enabled: true,
        }
    }

    /// Extract the glide time in seconds from Legato params.
    /// Returns None if this is not a Legato effect or params are missing.
    pub fn glide_secs(&self, bpm: f32) -> Option<f32> {
        if self.effect_type != NoteEffectType::Legato {
            return None;
        }
        let sync = self
            .params
            .first()
            .map(|p| p.value.to_f32() > 0.5)
            .unwrap_or(true);
        if sync {
            let rate_idx = self
                .params
                .get(1)
                .map(|p| p.value.to_f32() as i32)
                .unwrap_or(2);
            let rate = GlideRate::from_index(rate_idx);
            Some(rate.to_secs(bpm))
        } else {
            let time = self.params.get(2).map(|p| p.value.to_f32()).unwrap_or(0.1);
            Some(time)
        }
    }

    /// Extract the staccato amount (0.1-1.0) from Staccato params.
    pub fn staccato_amount(&self) -> Option<f32> {
        if self.effect_type != NoteEffectType::Staccato {
            return None;
        }
        Some(self.params.first().map(|p| p.value.to_f32()).unwrap_or(0.5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legato_default_params() {
        let params = NoteEffectType::Legato.default_params();
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].name, "sync");
        assert_eq!(params[1].name, "rate");
        assert_eq!(params[2].name, "time");
    }

    #[test]
    fn staccato_default_params() {
        let params = NoteEffectType::Staccato.default_params();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "amount");
    }

    #[test]
    fn note_effect_slot_new() {
        let slot = NoteEffectSlot::new(EffectId::new(0), NoteEffectType::Legato);
        assert!(slot.enabled);
        assert_eq!(slot.effect_type, NoteEffectType::Legato);
        assert_eq!(slot.params.len(), 3);
    }

    #[test]
    fn legato_glide_secs_sync() {
        let slot = NoteEffectSlot::new(EffectId::new(0), NoteEffectType::Legato);
        // Default: sync=true, rate=2 (1/16)
        let secs = slot.glide_secs(120.0).unwrap();
        assert!((secs - GlideRate::Sixteenth.to_secs(120.0)).abs() < 0.001);
    }

    #[test]
    fn legato_glide_secs_free() {
        let mut slot = NoteEffectSlot::new(EffectId::new(0), NoteEffectType::Legato);
        slot.params[0].value = ParamValue::Bool(false); // sync off
        slot.params[2].value = ParamValue::Float(0.3);
        let secs = slot.glide_secs(120.0).unwrap();
        assert!((secs - 0.3).abs() < 0.001);
    }

    #[test]
    fn staccato_amount() {
        let slot = NoteEffectSlot::new(EffectId::new(0), NoteEffectType::Staccato);
        assert!((slot.staccato_amount().unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn glide_secs_returns_none_for_staccato() {
        let slot = NoteEffectSlot::new(EffectId::new(0), NoteEffectType::Staccato);
        assert!(slot.glide_secs(120.0).is_none());
    }

    #[test]
    fn staccato_amount_returns_none_for_legato() {
        let slot = NoteEffectSlot::new(EffectId::new(0), NoteEffectType::Legato);
        assert!(slot.staccato_amount().is_none());
    }
}
