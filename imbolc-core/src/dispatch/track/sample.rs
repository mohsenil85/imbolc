use crate::action::{AudioEffect, DispatchResult, NavIntent};
use crate::state::AppState;
use imbolc_audio::AudioHandle;

pub(super) fn handle_load_sample_result(
    state: &mut AppState,
    audio: &mut AudioHandle,
    instrument_id: crate::state::TrackId,
    path: &std::path::Path,
) -> DispatchResult {
    // Copy to project assets if project is saved
    let asset_path = if let Some(ref project_path) = state.project.path {
        match super::super::helpers::copy_to_project_assets(path, project_path) {
            Ok(p) => p,
            Err(e) => {
                return DispatchResult::with_status(
                    audio.status(),
                    format!("Asset copy failed: {}", e),
                );
            }
        }
    } else {
        return DispatchResult::with_status(
            audio.status(),
            "Save project before importing samples",
        );
    };

    let path_str = asset_path.to_string_lossy().to_string();
    let sample_name = asset_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string());

    let buffer_id = state.tracks.next_sampler_buffer_id;
    state.tracks.next_sampler_buffer_id = buffer_id.next();

    if audio.is_running() {
        let _ = audio.load_sample(buffer_id, &path_str);
    }

    if let Some(instrument) = state.tracks.track_mut(instrument_id) {
        if let Some(ref mut config) = instrument.sampler_config_mut() {
            config.buffer_id = Some(buffer_id);
            config.sample_name = sample_name;
            config.sample_path = Some(path_str);
        }
    }

    let mut result = DispatchResult::with_nav(NavIntent::Pop);
    result.audio_effects.push(AudioEffect::RebuildInstruments);
    result
}
