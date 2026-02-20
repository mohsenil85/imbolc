//! Temp file cache for SuperCollider sample playback.
//!
//! SuperCollider requires filesystem paths for `/b_allocRead`. This module
//! extracts sample blobs from the project database into temp files so SC
//! can load them. The database blob is the source of truth; disk files are
//! a derived cache.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use imbolc_types::SampleId;

use super::sample_store;

/// Manages a temporary directory of extracted sample files.
pub struct SampleCache {
    cache_dir: PathBuf,
    cached: HashMap<SampleId, PathBuf>,
}

impl SampleCache {
    /// Create a new cache rooted at the given directory.
    /// The directory is created lazily on first materialize.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            cached: HashMap::new(),
        }
    }

    /// Create a cache in a subdirectory of the system temp dir.
    pub fn in_temp_dir(project_name: &str) -> Self {
        let dir = std::env::temp_dir()
            .join("imbolc-sample-cache")
            .join(project_name);
        Self::new(dir)
    }

    /// Extract a sample blob to a temp file if not already cached.
    /// Returns the filesystem path SC can use.
    pub fn materialize(
        &mut self,
        conn: &Connection,
        sample_id: SampleId,
    ) -> Result<PathBuf, String> {
        // Return cached path if already materialized
        if let Some(path) = self.cached.get(&sample_id) {
            if path.exists() {
                return Ok(path.clone());
            }
            // File was deleted externally; re-materialize
            self.cached.remove(&sample_id);
        }

        // Ensure cache dir exists
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|e| format!("Failed to create sample cache dir: {}", e))?;

        let data = sample_store::read_blob(conn, sample_id)?;
        let name = sample_store::read_name(conn, sample_id)?;

        // Use a deterministic filename: id_name.wav
        let safe_name: String = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let filename = format!("{}_{}.wav", sample_id.get(), safe_name);
        let path = self.cache_dir.join(filename);

        std::fs::write(&path, &data)
            .map_err(|e| format!("Failed to write sample cache file: {}", e))?;

        self.cached.insert(sample_id, path.clone());
        Ok(path)
    }

    /// Materialize a sample and return the path as a String.
    pub fn materialize_str(
        &mut self,
        conn: &Connection,
        sample_id: SampleId,
    ) -> Result<String, String> {
        let path = self.materialize(conn, sample_id)?;
        path.to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Cache path contains non-UTF8 characters".to_string())
    }

    /// Walk all tracks in the state and materialize every referenced sample.
    /// Populates `cache_path` on each `SampleRef` in the state.
    pub fn materialize_all(
        &mut self,
        conn: &Connection,
        tracks: &mut imbolc_types::TrackState,
    ) -> Result<(), String> {
        for track in &mut tracks.tracks {
            // Sampler config
            if let Some(config) = track.sampler_config_mut() {
                if let Some(ref mut sr) = config.sample_ref {
                    if sr.id.get() != 0 {
                        let path = self.materialize_str(conn, sr.id)?;
                        sr.cache_path = Some(path);
                    }
                }
            }

            // Drum pads
            if let Some(seq) = track.drum_sequencer_mut() {
                for pad in &mut seq.pads {
                    if let Some(ref mut sr) = pad.sample_ref {
                        if sr.id.get() != 0 {
                            let path = self.materialize_str(conn, sr.id)?;
                            sr.cache_path = Some(path);
                        }
                    }
                }

                // Chopper
                if let Some(ref mut chopper) = seq.chopper {
                    if let Some(ref mut sr) = chopper.sample_ref {
                        if sr.id.get() != 0 {
                            let path = self.materialize_str(conn, sr.id)?;
                            sr.cache_path = Some(path);
                        }
                    }
                }
            }

            // Convolution IR
            if let Some(ref mut sr) = track.convolution_ir_sample {
                if sr.id.get() != 0 {
                    let path = self.materialize_str(conn, sr.id)?;
                    sr.cache_path = Some(path);
                }
            }
        }

        Ok(())
    }

    /// Re-materialize a single sample (e.g., after an in-place edit).
    /// Removes the old cached file and writes a fresh one.
    pub fn invalidate(
        &mut self,
        conn: &Connection,
        sample_id: SampleId,
    ) -> Result<PathBuf, String> {
        if let Some(old_path) = self.cached.remove(&sample_id) {
            let _ = std::fs::remove_file(&old_path);
        }
        self.materialize(conn, sample_id)
    }

    /// Delete all cached files and reset the mapping.
    pub fn clear(&mut self) {
        for path in self.cached.values() {
            let _ = std::fs::remove_file(path);
        }
        self.cached.clear();
        let _ = std::fs::remove_dir_all(&self.cache_dir);
    }

    /// Get the cached path for a sample without materializing.
    pub fn get_path(&self, sample_id: SampleId) -> Option<&Path> {
        self.cached.get(&sample_id).map(|p| p.as_path())
    }
}

impl Drop for SampleCache {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn test_wav_bytes() -> Vec<u8> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
            for i in 0..4410 {
                let t = i as f32 / 44100.0;
                let sample = (t * 440.0 * std::f32::consts::TAU).sin() * 16000.0;
                writer.write_sample(sample as i16).unwrap();
            }
            writer.finalize().unwrap();
        }
        cursor.into_inner()
    }

    #[test]
    fn materialize_and_read_back() {
        let conn = Connection::open_in_memory().unwrap();
        sample_store::ensure_table(&conn).unwrap();

        let wav = test_wav_bytes();
        let (sr, _) =
            sample_store::import_sample_bytes(&conn, "test_tone", &wav, SampleId::new(1)).unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let mut cache = SampleCache::new(tmp.path().to_path_buf());

        let path = cache.materialize(&conn, sr.id).unwrap();
        assert!(path.exists());

        let read_back = std::fs::read(&path).unwrap();
        assert_eq!(read_back.len(), wav.len());

        // Second call should return cached path
        let path2 = cache.materialize(&conn, sr.id).unwrap();
        assert_eq!(path, path2);
    }

    #[test]
    fn invalidate_re_materializes() {
        let conn = Connection::open_in_memory().unwrap();
        sample_store::ensure_table(&conn).unwrap();

        let wav = test_wav_bytes();
        let (sr, _) =
            sample_store::import_sample_bytes(&conn, "tone", &wav, SampleId::new(1)).unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let mut cache = SampleCache::new(tmp.path().to_path_buf());

        let path1 = cache.materialize(&conn, sr.id).unwrap();
        assert!(path1.exists());

        let path2 = cache.invalidate(&conn, sr.id).unwrap();
        assert!(path2.exists());
        // File content should match original
        let data = std::fs::read(&path2).unwrap();
        assert_eq!(data.len(), wav.len());
    }

    #[test]
    fn clear_removes_files() {
        let conn = Connection::open_in_memory().unwrap();
        sample_store::ensure_table(&conn).unwrap();

        let wav = test_wav_bytes();
        sample_store::import_sample_bytes(&conn, "tone", &wav, SampleId::new(1)).unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let mut cache = SampleCache::new(tmp.path().join("cache"));

        let path = cache.materialize(&conn, SampleId::new(1)).unwrap();
        assert!(path.exists());

        cache.clear();
        assert!(!path.exists());
        assert!(cache.get_path(SampleId::new(1)).is_none());
    }
}
