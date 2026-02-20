use crate::action::{AudioEffect, DispatchResult, NavIntent};
use crate::state::AppState;
use imbolc_audio::AudioHandle;

pub(super) fn handle_load_sample_result(
    state: &mut AppState,
    audio: &mut AudioHandle,
    instrument_id: crate::state::TrackId,
    path: &std::path::Path,
) -> DispatchResult {
    let sample_ref = match super::super::helpers::import_sample_blob(state, path) {
        Ok(sr) => sr,
        Err(e) => {
            return DispatchResult::with_status(audio.status(), format!("Import failed: {}", e));
        }
    };

    let path_str = sample_ref
        .cache_path
        .as_deref()
        .unwrap_or_default()
        .to_string();

    let buffer_id = state.tracks.next_sampler_buffer_id;
    state.tracks.next_sampler_buffer_id = buffer_id.next();

    if audio.is_running() {
        let _ = audio.load_sample(buffer_id, &path_str);
    }

    if let Some(instrument) = state.tracks.track_mut(instrument_id) {
        if let Some(config) = instrument.sampler_config_mut() {
            config.buffer_id = Some(buffer_id);
            config.sample_ref = Some(sample_ref);
        }
    }

    let mut result = DispatchResult::with_nav(NavIntent::Pop);
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}
