//! Audio format normalization — converts AIFF, CAF, and other formats to WAV.
//!
//! The rest of the codebase (peak computation, waveform editing, SC loading, blob store)
//! assumes WAV. This module normalizes at the import boundary so nothing downstream
//! needs format awareness.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// If `data` is already WAV, return it unchanged. Otherwise decode via symphonia
/// and re-encode as 32-bit float WAV via hound.
pub fn normalize_to_wav(data: &[u8]) -> Result<Vec<u8>, String> {
    if is_wav(data) {
        return Ok(data.to_vec());
    }
    decode_to_wav(data)
}

/// If the file at `path` is already WAV, return the path unchanged. Otherwise decode
/// to a temporary WAV file alongside the original and return the new path.
pub fn ensure_wav_file(path: &Path) -> Result<PathBuf, String> {
    let data =
        std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    if is_wav(&data) {
        return Ok(path.to_path_buf());
    }

    let wav_data = decode_to_wav(&data)?;

    // Write converted WAV next to the original with .wav extension
    let wav_path = path.with_extension("converted.wav");
    std::fs::write(&wav_path, &wav_data)
        .map_err(|e| format!("Failed to write converted WAV: {}", e))?;
    Ok(wav_path)
}

/// Quick check: does the data start with a RIFF/WAVE header?
fn is_wav(data: &[u8]) -> bool {
    data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE"
}

/// Decode any symphonia-supported format to 32-bit float WAV bytes.
fn decode_to_wav(data: &[u8]) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(data.to_vec());
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| format!("Unsupported audio format: {}", e))?;

    let mut format_reader = probed.format;

    let track = format_reader
        .default_track()
        .ok_or_else(|| "No audio track found".to_string())?;

    let codec_params = track.codec_params.clone();
    let track_id = track.id;

    let sample_rate = codec_params
        .sample_rate
        .ok_or_else(|| "Unknown sample rate".to_string())?;
    let channels = codec_params
        .channels
        .ok_or_else(|| "Unknown channel layout".to_string())?
        .count();

    let decoder_opts = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &decoder_opts)
        .map_err(|e| format!("Failed to create decoder: {}", e))?;

    // Decode all packets into interleaved f32 samples
    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format_reader.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(format!("Error reading packet: {}", e)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = decoder
            .decode(&packet)
            .map_err(|e| format!("Decode error: {}", e))?;

        let spec = *decoded.spec();
        let num_frames = decoded.capacity();

        let mut sample_buf = SampleBuffer::<f32>::new(num_frames as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);
        all_samples.extend_from_slice(sample_buf.samples());
    }

    if all_samples.is_empty() {
        return Err("No audio samples decoded".to_string());
    }

    // Write as 32-bit float WAV
    let wav_spec = hound::WavSpec {
        channels: channels as u16,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut wav_buf = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut wav_buf, wav_spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;
        for &sample in &all_samples {
            writer
                .write_sample(sample)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }
        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    }

    Ok(wav_buf.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a short WAV file in memory via hound.
    fn make_wav(sample_rate: u32, channels: u16, num_frames: usize) -> Vec<u8> {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
            for i in 0..(num_frames * channels as usize) {
                let t = i as f32 / (sample_rate as f32 * channels as f32);
                let sample = (t * 440.0 * std::f32::consts::TAU).sin() * 16000.0;
                writer.write_sample(sample as i16).unwrap();
            }
            writer.finalize().unwrap();
        }
        cursor.into_inner()
    }

    #[test]
    fn wav_passthrough() {
        let wav = make_wav(44100, 1, 1000);
        let result = normalize_to_wav(&wav).unwrap();
        assert_eq!(result, wav, "WAV data should pass through unchanged");
    }

    #[test]
    fn is_wav_detection() {
        let wav = make_wav(44100, 1, 100);
        assert!(is_wav(&wav));
        assert!(!is_wav(b"not wav data at all"));
        assert!(!is_wav(&[]));
        assert!(!is_wav(&[0; 11]));
    }

    #[test]
    fn bad_data_returns_error() {
        let result = normalize_to_wav(b"this is not audio data");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Unsupported") || err.contains("error") || err.contains("Error"),
            "Expected meaningful error, got: {}",
            err
        );
    }

    #[test]
    fn empty_data_returns_error() {
        let result = normalize_to_wav(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn ensure_wav_file_passthrough() {
        let wav = make_wav(44100, 1, 500);
        let dir = tempfile::tempdir().unwrap();
        let wav_path = dir.path().join("test.wav");
        std::fs::write(&wav_path, &wav).unwrap();

        let result = ensure_wav_file(&wav_path).unwrap();
        assert_eq!(result, wav_path, "WAV file should return original path");
    }

    #[test]
    fn ensure_wav_file_missing() {
        let result = ensure_wav_file(Path::new("/nonexistent/audio.caf"));
        assert!(result.is_err());
    }
}
