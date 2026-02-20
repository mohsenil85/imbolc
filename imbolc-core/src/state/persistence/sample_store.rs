//! Blob I/O for sample storage in the project SQLite database.
//!
//! Samples are stored as blobs with SHA-256 content deduplication.
//! Each blob also carries WAV metadata (sample rate, channels, frames, duration)
//! so we can reconstruct display info without decoding.

use std::io::Cursor;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use sha2::{Digest, Sha256};

use imbolc_types::{SampleId, SampleRef};

/// Ensure the `sample_blobs` table exists. Called on project open/save.
/// This table is NOT dropped during normal save cycles — it's append-only
/// content-addressed storage.
pub fn ensure_table(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sample_blobs (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            content_hash TEXT NOT NULL UNIQUE,
            sample_rate INTEGER NOT NULL,
            num_channels INTEGER NOT NULL,
            num_frames INTEGER NOT NULL,
            duration_secs REAL NOT NULL,
            data BLOB NOT NULL
        );",
    )
}

/// Compute SHA-256 hex digest of a byte slice.
fn content_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Extract WAV metadata from raw bytes without reading all samples.
fn wav_metadata(data: &[u8]) -> Result<(u32, u16, u32, f32), String> {
    let reader = hound::WavReader::new(Cursor::new(data)).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let num_frames = reader.len() / spec.channels as u32;
    let duration_secs = num_frames as f32 / spec.sample_rate as f32;
    Ok((spec.sample_rate, spec.channels, num_frames, duration_secs))
}

/// Import a sample from a file path into the blob store.
///
/// Deduplicates by SHA-256: if an identical blob already exists, returns
/// a `SampleRef` pointing to the existing row (with the original name).
/// Otherwise inserts a new row using `next_id`.
///
/// Returns `(SampleRef, bool)` where the bool indicates whether the ID was
/// consumed (true = new insert, false = dedup hit, ID not used).
pub fn import_sample(
    conn: &Connection,
    path: &Path,
    next_id: SampleId,
) -> Result<(SampleRef, bool), String> {
    let data = std::fs::read(path).map_err(|e| format!("Failed to read sample file: {}", e))?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sample")
        .to_string();
    import_sample_bytes(conn, &name, &data, next_id)
}

/// Import sample bytes directly into the blob store (for recordings, edits).
///
/// Returns `(SampleRef, bool)` where the bool indicates whether the ID was
/// consumed (true = new insert, false = dedup hit).
pub fn import_sample_bytes(
    conn: &Connection,
    name: &str,
    data: &[u8],
    next_id: SampleId,
) -> Result<(SampleRef, bool), String> {
    let hash = content_hash(data);

    // Check for existing blob with same content
    let existing: Option<(i64, String)> = conn
        .query_row(
            "SELECT id, name FROM sample_blobs WHERE content_hash = ?1",
            params![hash],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some((existing_id, existing_name)) = existing {
        return Ok((
            SampleRef::new(SampleId::new(existing_id as u32), existing_name),
            false,
        ));
    }

    // Extract WAV metadata
    let (sample_rate, num_channels, num_frames, duration_secs) = wav_metadata(data)?;

    conn.execute(
        "INSERT INTO sample_blobs (id, name, content_hash, sample_rate, num_channels, num_frames, duration_secs, data)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            next_id.get() as i64,
            name,
            hash,
            sample_rate,
            num_channels as i64,
            num_frames as i64,
            duration_secs as f64,
            data,
        ],
    )
    .map_err(|e| format!("Failed to insert sample blob: {}", e))?;

    Ok((SampleRef::new(next_id, name.to_string()), true))
}

/// Read the raw WAV bytes for a stored sample.
pub fn read_blob(conn: &Connection, sample_id: SampleId) -> Result<Vec<u8>, String> {
    conn.query_row(
        "SELECT data FROM sample_blobs WHERE id = ?1",
        params![sample_id.get() as i64],
        |row| row.get(0),
    )
    .map_err(|e| format!("Failed to read sample blob {}: {}", sample_id, e))
}

/// Read the name for a stored sample.
pub fn read_name(conn: &Connection, sample_id: SampleId) -> Result<String, String> {
    conn.query_row(
        "SELECT name FROM sample_blobs WHERE id = ?1",
        params![sample_id.get() as i64],
        |row| row.get(0),
    )
    .map_err(|e| format!("Failed to read sample name {}: {}", sample_id, e))
}

/// Update the blob data for an existing sample (for in-place WAV edits).
///
/// Also updates the content hash and WAV metadata.
pub fn update_blob(conn: &Connection, sample_id: SampleId, new_data: &[u8]) -> Result<(), String> {
    let hash = content_hash(new_data);
    let (sample_rate, num_channels, num_frames, duration_secs) = wav_metadata(new_data)?;

    conn.execute(
        "UPDATE sample_blobs SET data = ?1, content_hash = ?2, sample_rate = ?3, num_channels = ?4, num_frames = ?5, duration_secs = ?6 WHERE id = ?7",
        params![
            new_data,
            hash,
            sample_rate,
            num_channels as i64,
            num_frames as i64,
            duration_secs as f64,
            sample_id.get() as i64,
        ],
    )
    .map_err(|e| format!("Failed to update sample blob {}: {}", sample_id, e))?;

    Ok(())
}

/// Check if a sample blob exists in the database.
pub fn blob_exists(conn: &Connection, sample_id: SampleId) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sample_blobs WHERE id = ?1",
            params![sample_id.get() as i64],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            for i in 0..44100 {
                let t = i as f32 / 44100.0;
                let sample = (t * 440.0 * std::f32::consts::TAU).sin() * 16000.0;
                writer.write_sample(sample as i16).unwrap();
            }
            writer.finalize().unwrap();
        }
        cursor.into_inner()
    }

    #[test]
    fn import_and_read_blob() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_table(&conn).unwrap();

        let wav = test_wav_bytes();
        let (sample_ref, consumed) =
            import_sample_bytes(&conn, "test_tone", &wav, SampleId::new(1)).unwrap();

        assert!(consumed);
        assert_eq!(sample_ref.id, SampleId::new(1));
        assert_eq!(sample_ref.name, "test_tone");

        let read_back = read_blob(&conn, sample_ref.id).unwrap();
        assert_eq!(read_back.len(), wav.len());
    }

    #[test]
    fn dedup_identical_blobs() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_table(&conn).unwrap();

        let wav = test_wav_bytes();
        let (ref1, consumed1) =
            import_sample_bytes(&conn, "first", &wav, SampleId::new(1)).unwrap();
        let (ref2, consumed2) =
            import_sample_bytes(&conn, "second", &wav, SampleId::new(2)).unwrap();

        assert!(consumed1);
        assert!(!consumed2); // dedup hit
        assert_eq!(ref1.id, ref2.id);
        assert_eq!(ref2.name, "first"); // keeps original name
    }

    #[test]
    fn update_blob_changes_data() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_table(&conn).unwrap();

        let wav = test_wav_bytes();
        let (sample_ref, _) = import_sample_bytes(&conn, "tone", &wav, SampleId::new(1)).unwrap();

        // Create different WAV data
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
            // Shorter clip
            for i in 0..22050 {
                let t = i as f32 / 44100.0;
                let sample = (t * 880.0 * std::f32::consts::TAU).sin() * 16000.0;
                writer.write_sample(sample as i16).unwrap();
            }
            writer.finalize().unwrap();
        }
        let new_wav = cursor.into_inner();

        update_blob(&conn, sample_ref.id, &new_wav).unwrap();

        let read_back = read_blob(&conn, sample_ref.id).unwrap();
        assert_eq!(read_back.len(), new_wav.len());
        assert_ne!(read_back.len(), wav.len());
    }

    #[test]
    fn blob_exists_check() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_table(&conn).unwrap();

        assert!(!blob_exists(&conn, SampleId::new(1)).unwrap());

        let wav = test_wav_bytes();
        import_sample_bytes(&conn, "tone", &wav, SampleId::new(1)).unwrap();

        assert!(blob_exists(&conn, SampleId::new(1)).unwrap());
        assert!(!blob_exists(&conn, SampleId::new(99)).unwrap());
    }
}
