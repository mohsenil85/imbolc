//! Patch types for exporting/importing track signal chains.

use serde::{Deserialize, Serialize};

use super::groove::GrooveConfig;
use super::track::{
    LayerConfig, ModulationConfig, NoteInputConfig, ProcessingStage, SourceExtra, SourceType,
};
use crate::{EffectId, Param};

/// A portable track signal chain — everything that defines a sound,
/// independent of the project it lives in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    /// Format version (currently 1).
    pub version: u32,
    /// Human-readable name (becomes the track name on import).
    pub name: String,
    pub source: SourceType,
    pub source_params: Vec<Param>,
    pub source_extra: SourceExtra,
    pub processing_chain: Vec<ProcessingStage>,
    pub next_effect_id: EffectId,
    pub modulation: ModulationConfig,
    pub note_input: NoteInputConfig,
    /// Maps to `PatchSample.original_id` for convolution IR sample lookup.
    pub convolution_ir_original_id: Option<u32>,
    pub layer: LayerConfig,
    pub groove: GrooveConfig,
    pub polyphonic: bool,
    /// Embedded sample data (base64-encoded WAV blobs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub samples: Vec<PatchSample>,
}

/// An embedded sample blob within a patch file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchSample {
    /// The SampleId from the source project (used as a mapping key during import).
    pub original_id: u32,
    /// Display name (original filename stem).
    pub name: String,
    /// Base64-encoded WAV bytes.
    pub data: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_round_trip_toml() {
        let patch = Patch {
            version: 1,
            name: "Test Patch".to_string(),
            source: SourceType::Saw,
            source_params: SourceType::Saw.default_params(),
            source_extra: SourceExtra::None,
            processing_chain: vec![],
            next_effect_id: EffectId::new(0),
            modulation: ModulationConfig::default(),
            note_input: NoteInputConfig::default(),
            convolution_ir_original_id: None,
            layer: LayerConfig::default(),
            groove: GrooveConfig::default(),
            polyphonic: true,
            samples: vec![],
        };

        let toml_str = toml::to_string_pretty(&patch).expect("serialize");
        let decoded: Patch = toml::from_str(&toml_str).expect("deserialize");

        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.name, "Test Patch");
        assert_eq!(decoded.source, SourceType::Saw);
        assert!(decoded.polyphonic);
        assert!(decoded.samples.is_empty());
    }

    #[test]
    fn patch_with_samples_round_trip() {
        let patch = Patch {
            version: 1,
            name: "Sampler Patch".to_string(),
            source: SourceType::PitchedSampler,
            source_params: SourceType::PitchedSampler.default_params(),
            source_extra: SourceExtra::Sampler(Default::default()),
            processing_chain: vec![],
            next_effect_id: EffectId::new(0),
            modulation: ModulationConfig::default(),
            note_input: NoteInputConfig::default(),
            convolution_ir_original_id: None,
            layer: LayerConfig::default(),
            groove: GrooveConfig::default(),
            polyphonic: true,
            samples: vec![PatchSample {
                original_id: 42,
                name: "kick.wav".to_string(),
                data: "AAAA".to_string(), // dummy base64
            }],
        };

        let toml_str = toml::to_string_pretty(&patch).expect("serialize");
        let decoded: Patch = toml::from_str(&toml_str).expect("deserialize");

        assert_eq!(decoded.samples.len(), 1);
        assert_eq!(decoded.samples[0].original_id, 42);
        assert_eq!(decoded.samples[0].name, "kick.wav");
        assert_eq!(decoded.samples[0].data, "AAAA");
    }
}
