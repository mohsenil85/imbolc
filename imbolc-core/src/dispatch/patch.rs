//! Patch export/import logic.
//!
//! Patches are portable track signal chains serialized as TOML with
//! embedded sample data (base64-encoded WAV blobs).

use std::collections::HashMap;
use std::path::Path;

use base64::prelude::*;
use rusqlite::Connection as SqlConnection;

use crate::state::persistence::{sample_store, SampleCache};
use crate::state::AppState;
use imbolc_types::{Patch, PatchSample, SampleRef, SourceExtra, Track, TrackId};

/// Collect all unique SampleRefs from a track's signal chain.
fn collect_sample_refs(track: &Track) -> Vec<SampleRef> {
    let mut refs: Vec<SampleRef> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut push = |r: &SampleRef| {
        if seen.insert(r.id) {
            refs.push(r.clone());
        }
    };

    // Pitched sampler / time-stretch
    if let Some(config) = track.sampler_config() {
        if let Some(ref sr) = config.sample_ref {
            push(sr);
        }
    }

    // Drum kit pads
    if let Some(seq) = track.drum_sequencer() {
        for pad in &seq.pads {
            if let Some(ref sr) = pad.sample_ref {
                push(sr);
            }
        }
        // Chopper/slicer
        if let Some(ref chopper) = seq.chopper {
            if let Some(ref sr) = chopper.sample_ref {
                push(sr);
            }
        }
    }

    // Convolution IR
    if let Some(ref sr) = track.convolution_ir_sample {
        push(sr);
    }

    refs
}

/// Export a track's signal chain as a Patch.
///
/// Reads sample blobs from the project DB and base64-encodes them.
pub fn export_patch(track: &Track, project_path: &Path) -> Result<String, String> {
    let conn = SqlConnection::open(project_path)
        .map_err(|e| format!("Failed to open project DB: {}", e))?;
    sample_store::ensure_table(&conn).map_err(|e| e.to_string())?;

    // Collect and embed samples
    let sample_refs = collect_sample_refs(track);
    let mut samples = Vec::new();
    for sr in &sample_refs {
        let blob = sample_store::read_blob(&conn, sr.id)?;
        samples.push(PatchSample {
            original_id: sr.id.get(),
            name: sr.name.clone(),
            data: BASE64_STANDARD.encode(&blob),
        });
    }

    let patch = Patch {
        version: 1,
        name: track.name.clone(),
        source: track.source,
        source_params: track.source_params.clone(),
        source_extra: track.source_extra.clone(),
        processing_chain: track.channel_strip.processing_chain.clone(),
        next_effect_id: track.channel_strip.next_effect_id,
        modulation: track.modulation.clone(),
        note_input: track.note_input.clone(),
        convolution_ir_original_id: track.convolution_ir_sample.as_ref().map(|sr| sr.id.get()),
        layer: track.layer,
        groove: track.groove,
        polyphonic: track.polyphonic,
        samples,
    };

    toml::to_string_pretty(&patch).map_err(|e| format!("Failed to serialize patch: {}", e))
}

/// Import a patch from a TOML file into the project state.
///
/// Creates a new track, imports embedded samples into the project DB,
/// and remaps all sample references to the new IDs.
pub fn import_patch_from_file(path: &Path, state: &mut AppState) -> Result<TrackId, String> {
    let toml_str =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read patch file: {}", e))?;
    let patch: Patch =
        toml::from_str(&toml_str).map_err(|e| format!("Failed to parse patch: {}", e))?;

    let project_path = state
        .project
        .path
        .as_ref()
        .ok_or_else(|| "Save project before importing patches".to_string())?
        .clone();

    let conn = SqlConnection::open(&project_path)
        .map_err(|e| format!("Failed to open project DB: {}", e))?;
    sample_store::ensure_table(&conn).map_err(|e| e.to_string())?;

    // Import embedded samples and build old_id → new_ref mapping
    let mut sample_map: HashMap<u32, SampleRef> = HashMap::new();
    for ps in &patch.samples {
        let data = BASE64_STANDARD
            .decode(&ps.data)
            .map_err(|e| format!("Failed to decode sample '{}': {}", ps.name, e))?;

        let next_id = state.tracks.next_sample_id;
        let (mut sample_ref, consumed) =
            sample_store::import_sample_bytes(&conn, &ps.name, &data, next_id)?;

        if consumed {
            state.tracks.next_sample_id = sample_ref.id.next();
        }

        // Materialize to temp file for SC
        let cache = state
            .sample_cache
            .get_or_insert_with(|| SampleCache::in_temp_dir("project"));
        let cache_path = cache.materialize_str(&conn, sample_ref.id)?;
        sample_ref.cache_path = Some(cache_path);

        sample_map.insert(ps.original_id, sample_ref);
    }

    // Create new track
    let track_id = state.add_track(patch.source);
    let track = state
        .tracks
        .track_mut(track_id)
        .ok_or_else(|| "Failed to create track".to_string())?;

    // Overwrite signal chain fields
    track.name = patch.name;
    track.source_params = patch.source_params;
    track.channel_strip.processing_chain = patch.processing_chain;
    track.channel_strip.next_effect_id = patch.next_effect_id;
    track.modulation = patch.modulation;
    track.note_input = patch.note_input;
    track.groove = patch.groove;
    track.polyphonic = patch.polyphonic;
    track.layer = imbolc_types::LayerConfig {
        group: None, // clear group — target project may not have it
        octave_offset: patch.layer.octave_offset,
    };

    // Remap sample references in source_extra
    track.source_extra = remap_source_extra(patch.source_extra, &sample_map);

    // Remap convolution IR
    track.convolution_ir_sample = patch
        .convolution_ir_original_id
        .and_then(|oid| sample_map.get(&oid).cloned());

    Ok(track_id)
}

/// Remap SampleRefs in SourceExtra using the old_id → new_ref mapping.
fn remap_source_extra(extra: SourceExtra, sample_map: &HashMap<u32, SampleRef>) -> SourceExtra {
    match extra {
        SourceExtra::Sampler(mut config) => {
            config.sample_ref = config
                .sample_ref
                .and_then(|sr| sample_map.get(&sr.id.get()).cloned());
            SourceExtra::Sampler(config)
        }
        SourceExtra::Kit(mut seq) => {
            for pad in &mut seq.pads {
                pad.sample_ref = pad
                    .sample_ref
                    .as_ref()
                    .and_then(|sr| sample_map.get(&sr.id.get()).cloned());
                // Clear instrument_id references — they belong to the source project
                pad.instrument_id = None;
            }
            if let Some(ref mut chopper) = seq.chopper {
                if let Some(ref old_sr) = chopper.sample_ref {
                    chopper.sample_ref = sample_map.get(&old_sr.id.get()).cloned();
                }
            }
            SourceExtra::Kit(seq)
        }
        SourceExtra::Vst {
            param_values,
            state_path: _,
        } => SourceExtra::Vst {
            param_values,
            state_path: None, // host-local, not portable
        },
        SourceExtra::None => SourceExtra::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imbolc_types::*;

    #[test]
    fn collect_sample_refs_empty_for_synth() {
        let track = Track::new(TrackId::new(1), SourceType::Saw);
        let refs = collect_sample_refs(&track);
        assert!(refs.is_empty());
    }

    #[test]
    fn collect_sample_refs_from_sampler() {
        let mut track = Track::new(TrackId::new(1), SourceType::PitchedSampler);
        if let Some(config) = track.sampler_config_mut() {
            config.sample_ref = Some(SampleRef::new(SampleId::new(42), "kick.wav".to_string()));
        }
        let refs = collect_sample_refs(&track);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id, SampleId::new(42));
    }

    #[test]
    fn collect_sample_refs_deduplicates() {
        let mut track = Track::new(TrackId::new(1), SourceType::PitchedSampler);
        let sr = SampleRef::new(SampleId::new(42), "kick.wav".to_string());
        if let Some(config) = track.sampler_config_mut() {
            config.sample_ref = Some(sr.clone());
        }
        // Also set convolution IR to same sample
        track
            .channel_strip
            .processing_chain
            .push(ProcessingStage::Effect(EffectSlot::new(
                EffectId::new(0),
                EffectType::ConvolutionReverb,
            )));
        track.convolution_ir_sample = Some(sr);
        let refs = collect_sample_refs(&track);
        assert_eq!(refs.len(), 1); // deduplicated
    }

    #[test]
    fn remap_source_extra_sampler() {
        let config = SamplerConfig {
            sample_ref: Some(SampleRef::new(SampleId::new(10), "old.wav".to_string())),
            ..Default::default()
        };
        let extra = SourceExtra::Sampler(config);

        let mut map = HashMap::new();
        map.insert(10, SampleRef::new(SampleId::new(99), "old.wav".to_string()));

        let remapped = remap_source_extra(extra, &map);
        if let SourceExtra::Sampler(c) = remapped {
            assert_eq!(c.sample_ref.unwrap().id, SampleId::new(99));
        } else {
            panic!("Expected Sampler variant");
        }
    }

    #[test]
    fn remap_source_extra_vst_clears_state_path() {
        let extra = SourceExtra::Vst {
            param_values: vec![(0, 0.5)],
            state_path: Some("/tmp/state.bin".into()),
        };
        let remapped = remap_source_extra(extra, &HashMap::new());
        if let SourceExtra::Vst {
            state_path,
            param_values,
        } = remapped
        {
            assert!(state_path.is_none());
            assert_eq!(param_values.len(), 1);
        } else {
            panic!("Expected Vst variant");
        }
    }
}
