use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::persistence::{load_project, save_project};

mod arrangement;
mod basic;
mod decoders;
mod mixer;
mod tracks;

fn temp_db_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("imbolc_persistence_test_{}.sqlite", nanos));
    path
}

/// Insert a minimal sample_blobs row for test round-trips.
fn insert_test_blob(db_path: &std::path::Path, sample_id: imbolc_types::SampleId, name: &str) {
    let conn = rusqlite::Connection::open(db_path).expect("open db");
    conn.execute(
        "INSERT OR IGNORE INTO sample_blobs (id, name, content_hash, sample_rate, num_channels, num_frames, duration_secs, data)
         VALUES (?1, ?2, ?3, 44100, 1, 0, 0.0, X'')",
        rusqlite::params![sample_id.get() as i64, name, format!("test_hash_{}", sample_id.get())],
    )
    .expect("insert test blob");
}
