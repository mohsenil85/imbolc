use crate::action::{AudioEffect, DispatchResult};
use crate::state::automation::AutomationTarget;
use crate::state::AppState;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Derive the assets directory for a project: foo.sqlite → foo_assets/
pub fn assets_dir_for_project(project_path: &Path) -> PathBuf {
    let stem = project_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("project");
    let parent = project_path.parent().unwrap_or(Path::new("."));
    parent.join(format!("{}_assets", stem))
}

/// Copy a file into the project's assets directory.
/// Returns the new absolute path. Deduplicates filenames with _1, _2, etc.
/// Skips copy if source is already inside the assets dir.
pub fn copy_to_project_assets(source: &Path, project_path: &Path) -> Result<PathBuf, String> {
    let assets_dir = assets_dir_for_project(project_path);
    std::fs::create_dir_all(&assets_dir)
        .map_err(|e| format!("Failed to create assets dir: {}", e))?;

    // Skip if already inside assets dir
    if let (Ok(canon_source), Ok(canon_assets)) = (source.canonicalize(), assets_dir.canonicalize())
    {
        if canon_source.starts_with(&canon_assets) {
            return Ok(canon_source);
        }
    }

    let file_name = source
        .file_name()
        .ok_or_else(|| "Source has no filename".to_string())?;
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sample");
    let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("wav");

    let mut dest = assets_dir.join(file_name);
    let mut counter = 1u32;
    while dest.exists() {
        // Check if existing file is identical (same size)
        if let (Ok(src_meta), Ok(dst_meta)) = (source.metadata(), dest.metadata()) {
            if src_meta.len() == dst_meta.len() {
                return Ok(dest);
            }
        }
        dest = assets_dir.join(format!("{}_{}.{}", stem, counter, ext));
        counter += 1;
    }

    std::fs::copy(source, &dest).map_err(|e| format!("Failed to copy sample to assets: {}", e))?;
    Ok(dest)
}

use super::automation::record_automation_point;

/// Record automation point if currently recording and playing.
/// Returns true if a point was recorded (for pushing AudioEffect::UpdateAutomation).
pub fn maybe_record_automation(
    state: &mut AppState,
    result: &mut DispatchResult,
    target: AutomationTarget,
    value: f32,
) {
    if state.recording.automation_recording && state.audio.playing {
        record_automation_point(state, target, value);
        result.audio_effects.push(AudioEffect::UpdateAutomation);
    }
}

/// Set bus mixer params directly if audio is running.
pub fn apply_bus_update(
    audio: &mut imbolc_audio::AudioHandle,
    update: Option<(imbolc_types::BusId, f32, bool, f32)>,
) {
    if let Some((bus_id, level, mute, pan)) = update {
        if audio.is_running() {
            let _ = audio.set_bus_mixer_params(bus_id, level, mute, pan);
        }
    }
}

/// Set layer group mixer params directly if audio is running.
pub fn apply_layer_group_update(
    audio: &mut imbolc_audio::AudioHandle,
    update: Option<(imbolc_types::GroupId, f32, bool, f32)>,
) {
    if let Some((group_id, level, mute, pan)) = update {
        if audio.is_running() {
            let _ = audio.set_layer_group_mixer_params(group_id, level, mute, pan);
        }
    }
}

/// Compute waveform peaks from a WAV file for display
pub fn compute_waveform_peaks(path: &str) -> (Vec<f32>, f32) {
    let reader = match hound::WavReader::open(path) {
        Ok(r) => r,
        Err(_) => return (Vec::new(), 0.0),
    };
    let spec = reader.spec();
    let num_channels = spec.channels as usize;
    let sample_rate = spec.sample_rate;
    let num_samples = reader.len() as usize;
    let duration_secs = num_samples as f32 / (sample_rate as f32 * num_channels as f32);

    let target_peaks = 512;
    let samples_per_peak = (num_samples / target_peaks).max(1);

    let mut peaks = Vec::with_capacity(target_peaks);
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let max_val = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .into_samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / max_val)
                .collect()
        }
        hound::SampleFormat::Float => reader
            .into_samples::<f32>()
            .filter_map(|s| s.ok())
            .collect(),
    };

    for chunk in samples.chunks(samples_per_peak) {
        let peak = chunk.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
        peaks.push(peak);
    }

    (peaks, duration_secs)
}

/// Append `segment` WAV samples to `target` WAV in-place.
///
/// If `target` doesn't exist yet, this moves `segment` to `target`.
pub fn append_wav_files(target: &Path, segment: &Path) -> Result<(), String> {
    if !segment.exists() {
        return Err(format!(
            "append source missing: {}",
            segment.to_string_lossy()
        ));
    }

    if !target.exists() {
        std::fs::rename(segment, target).map_err(|e| e.to_string())?;
        return Ok(());
    }

    let mut target_reader = hound::WavReader::open(target).map_err(|e| e.to_string())?;
    let mut segment_reader = hound::WavReader::open(segment).map_err(|e| e.to_string())?;
    let target_spec = target_reader.spec();
    let segment_spec = segment_reader.spec();

    if target_spec != segment_spec {
        return Err("Cannot append WAV files with different formats".to_string());
    }

    let tmp_out = target.with_extension("append.tmp.wav");
    let mut writer = hound::WavWriter::create(&tmp_out, target_spec).map_err(|e| e.to_string())?;

    match target_spec.sample_format {
        hound::SampleFormat::Float => {
            for sample in target_reader.samples::<f32>() {
                writer
                    .write_sample(sample.map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            }
            for sample in segment_reader.samples::<f32>() {
                writer
                    .write_sample(sample.map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            }
        }
        hound::SampleFormat::Int => {
            if target_spec.bits_per_sample <= 16 {
                for sample in target_reader.samples::<i16>() {
                    writer
                        .write_sample(sample.map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                }
                for sample in segment_reader.samples::<i16>() {
                    writer
                        .write_sample(sample.map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                }
            } else {
                for sample in target_reader.samples::<i32>() {
                    writer
                        .write_sample(sample.map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                }
                for sample in segment_reader.samples::<i32>() {
                    writer
                        .write_sample(sample.map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
    }

    writer.finalize().map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_out, target).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(segment);
    Ok(())
}

fn normalized_frame_range(total_frames: usize, start_norm: f32, end_norm: f32) -> (usize, usize) {
    if total_frames == 0 {
        return (0, 0);
    }
    let a = start_norm.clamp(0.0, 1.0);
    let b = end_norm.clamp(0.0, 1.0);
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    let mut start = (lo * total_frames as f32).floor() as usize;
    let mut end = (hi * total_frames as f32).ceil() as usize;
    start = start.min(total_frames.saturating_sub(1));
    end = end.min(total_frames);
    if end <= start {
        end = (start + 1).min(total_frames);
    }
    (start, end)
}

/// Edit a WAV file in place by deleting a frame range or trimming to that range.
///
/// `start_norm` and `end_norm` are in 0.0..=1.0 over the full clip duration.
pub fn edit_wav_file_selection(
    path: &Path,
    start_norm: f32,
    end_norm: f32,
    trim: bool,
) -> Result<(), String> {
    let reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let channels = spec.channels as usize;
    if channels == 0 {
        return Err("Invalid WAV: zero channels".to_string());
    }

    let tmp_out = path.with_extension("waveedit.tmp.wav");
    let selection_error = "Selection removes entire clip".to_string();

    match spec.sample_format {
        hound::SampleFormat::Float => {
            let samples: Vec<f32> = reader
                .into_samples::<f32>()
                .map(|s| s.map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            let total_frames = samples.len() / channels;
            if total_frames == 0 {
                return Err("WAV has no samples".to_string());
            }
            let (start, end) = normalized_frame_range(total_frames, start_norm, end_norm);
            let mut kept: Vec<f32> = Vec::with_capacity(samples.len());
            if trim {
                kept.extend_from_slice(&samples[start * channels..end * channels]);
            } else {
                kept.extend_from_slice(&samples[..start * channels]);
                kept.extend_from_slice(&samples[end * channels..]);
            }
            if kept.is_empty() {
                return Err(selection_error);
            }
            let mut writer = hound::WavWriter::create(&tmp_out, spec).map_err(|e| e.to_string())?;
            for sample in kept {
                writer.write_sample(sample).map_err(|e| e.to_string())?;
            }
            writer.finalize().map_err(|e| e.to_string())?;
        }
        hound::SampleFormat::Int if spec.bits_per_sample <= 16 => {
            let samples: Vec<i16> = reader
                .into_samples::<i16>()
                .map(|s| s.map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            let total_frames = samples.len() / channels;
            if total_frames == 0 {
                return Err("WAV has no samples".to_string());
            }
            let (start, end) = normalized_frame_range(total_frames, start_norm, end_norm);
            let mut kept: Vec<i16> = Vec::with_capacity(samples.len());
            if trim {
                kept.extend_from_slice(&samples[start * channels..end * channels]);
            } else {
                kept.extend_from_slice(&samples[..start * channels]);
                kept.extend_from_slice(&samples[end * channels..]);
            }
            if kept.is_empty() {
                return Err(selection_error);
            }
            let mut writer = hound::WavWriter::create(&tmp_out, spec).map_err(|e| e.to_string())?;
            for sample in kept {
                writer.write_sample(sample).map_err(|e| e.to_string())?;
            }
            writer.finalize().map_err(|e| e.to_string())?;
        }
        hound::SampleFormat::Int => {
            let samples: Vec<i32> = reader
                .into_samples::<i32>()
                .map(|s| s.map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            let total_frames = samples.len() / channels;
            if total_frames == 0 {
                return Err("WAV has no samples".to_string());
            }
            let (start, end) = normalized_frame_range(total_frames, start_norm, end_norm);
            let mut kept: Vec<i32> = Vec::with_capacity(samples.len());
            if trim {
                kept.extend_from_slice(&samples[start * channels..end * channels]);
            } else {
                kept.extend_from_slice(&samples[..start * channels]);
                kept.extend_from_slice(&samples[end * channels..]);
            }
            if kept.is_empty() {
                return Err(selection_error);
            }
            let mut writer = hound::WavWriter::create(&tmp_out, spec).map_err(|e| e.to_string())?;
            for sample in kept {
                writer.write_sample(sample).map_err(|e| e.to_string())?;
            }
            writer.finalize().map_err(|e| e.to_string())?;
        }
    }

    std::fs::rename(&tmp_out, path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Extract a normalized WAV range into a new temporary WAV file.
///
/// The output preserves source format/channels/sample rate and contains only the
/// selected range (`start_norm..end_norm`) of the source clip.
pub fn extract_wav_selection_to_temp(
    source_path: &Path,
    start_norm: f32,
    end_norm: f32,
    suffix: &str,
) -> Result<PathBuf, String> {
    if !source_path.exists() {
        return Err(format!(
            "Source WAV does not exist: {}",
            source_path.to_string_lossy()
        ));
    }

    let tmp_dir = std::env::temp_dir().join("imbolc-waveform-clips");
    std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;

    let stem = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("clip");
    let safe_stem: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let out_path = tmp_dir.join(format!("{}_{}_{}.wav", safe_stem, suffix, ts));

    std::fs::copy(source_path, &out_path).map_err(|e| e.to_string())?;
    edit_wav_file_selection(&out_path, start_norm, end_norm, true)?;
    Ok(out_path)
}
