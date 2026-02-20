//! Legacy project migration.
//!
//! Detects old schema (file-path-based sample references) and migrates to
//! blob-based storage. This runs once during `load_project()` on legacy DBs.

use std::path::Path;

use rusqlite::{params, Connection, Result as SqlResult};

use super::sample_store;
use imbolc_types::SampleId;

/// Check if a table has a given column.
fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
    let sql = format!(
        "SELECT COUNT(*) FROM pragma_table_info('{}') WHERE name = ?1",
        table
    );
    conn.query_row(&sql, params![column], |row| row.get::<_, i32>(0))
        .unwrap_or(0)
        > 0
}

/// Add a column to a table if it doesn't exist. Returns true if added.
fn add_column_if_missing(conn: &Connection, table: &str, column: &str, col_type: &str) -> bool {
    if has_column(conn, table, column) {
        return false;
    }
    let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type);
    conn.execute_batch(&sql).is_ok()
}

/// Migrate a legacy project's sample references from file paths to blob IDs.
///
/// Detects old schema by checking for `sample_path` in `sampler_configs`.
/// If found, imports referenced sample files into `sample_blobs` and populates
/// the new `sample_id` columns. Files that can't be found on disk are skipped
/// with a warning (the sample reference will be None after loading).
///
/// This is idempotent — safe to call on already-migrated projects.
pub fn migrate_legacy_samples(conn: &Connection) -> SqlResult<()> {
    // Quick check: if sampler_configs has sample_path, it's a legacy DB.
    // Also check convolution_ir_path on tracks.
    let has_legacy_sampler = has_column(conn, "sampler_configs", "sample_path");
    let has_legacy_ir = has_column(conn, "tracks", "convolution_ir_path")
        && !has_column(conn, "tracks", "convolution_ir_sample_id");
    let has_legacy_pads =
        has_column(conn, "drum_pads", "path") && !has_column(conn, "drum_pads", "sample_id");
    let has_legacy_chopper = has_column(conn, "chopper_states", "path")
        && !has_column(conn, "chopper_states", "sample_id");
    let has_legacy_session = !has_column(conn, "session", "next_sample_id");

    if !has_legacy_sampler
        && !has_legacy_ir
        && !has_legacy_pads
        && !has_legacy_chopper
        && !has_legacy_session
    {
        return Ok(()); // Already migrated or new project
    }

    log::info!("Migrating legacy project to blob-based sample storage");

    // Ensure sample_blobs table exists
    sample_store::ensure_table(conn)?;

    // Add new columns (idempotent)
    add_column_if_missing(conn, "sampler_configs", "sample_id", "INTEGER");
    add_column_if_missing(conn, "drum_pads", "sample_id", "INTEGER");
    add_column_if_missing(conn, "chopper_states", "sample_id", "INTEGER");
    add_column_if_missing(conn, "tracks", "convolution_ir_sample_id", "INTEGER");
    add_column_if_missing(
        conn,
        "session",
        "next_sample_id",
        "INTEGER NOT NULL DEFAULT 0",
    );

    let mut next_id: u32 = 0;

    // Migrate sampler_configs: sample_path -> sample_id
    if has_legacy_sampler {
        migrate_sampler_configs(conn, &mut next_id);
    }

    // Migrate drum_pads: path -> sample_id
    if has_legacy_pads {
        migrate_drum_pads(conn, &mut next_id);
    }

    // Migrate chopper_states: path -> sample_id
    if has_legacy_chopper {
        migrate_chopper_states(conn, &mut next_id);
    }

    // Migrate tracks: convolution_ir_path -> convolution_ir_sample_id
    if has_legacy_ir {
        migrate_convolution_ir(conn, &mut next_id);
    }

    // Update session with next_sample_id
    if has_legacy_session || next_id > 0 {
        let _ = conn.execute(
            "UPDATE session SET next_sample_id = ?1 WHERE id = 1",
            params![next_id],
        );
    }

    log::info!(
        "Legacy sample migration complete, imported {} samples",
        next_id
    );
    Ok(())
}

/// Try to import a file at `path` into sample_blobs. Returns the SampleId on success.
fn try_import(conn: &Connection, path: &str, next_id: &mut u32) -> Option<SampleId> {
    let p = Path::new(path);
    if !p.exists() {
        log::warn!("Legacy migration: sample file not found: {}", path);
        return None;
    }

    let sample_id = SampleId::new(*next_id);
    match sample_store::import_sample(conn, p, sample_id) {
        Ok((sr, consumed)) => {
            if consumed {
                *next_id += 1;
            }
            Some(sr.id)
        }
        Err(e) => {
            log::warn!("Legacy migration: failed to import {}: {}", path, e);
            None
        }
    }
}

fn migrate_sampler_configs(conn: &Connection, next_id: &mut u32) {
    let rows: Vec<(u32, String)> = conn
        .prepare("SELECT track_id, sample_path FROM sampler_configs WHERE sample_path IS NOT NULL AND sample_id IS NULL")
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .and_then(|rows| rows.collect())
        })
        .unwrap_or_default();

    for (track_id, path) in rows {
        if let Some(sid) = try_import(conn, &path, next_id) {
            let _ = conn.execute(
                "UPDATE sampler_configs SET sample_id = ?1 WHERE track_id = ?2",
                params![sid.get() as i64, track_id],
            );
        }
    }
}

fn migrate_drum_pads(conn: &Connection, next_id: &mut u32) {
    let rows: Vec<(u32, i32, String)> = conn
        .prepare("SELECT track_id, pad_index, path FROM drum_pads WHERE path IS NOT NULL AND sample_id IS NULL")
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .and_then(|rows| rows.collect())
        })
        .unwrap_or_default();

    for (track_id, pad_index, path) in rows {
        if let Some(sid) = try_import(conn, &path, next_id) {
            let _ = conn.execute(
                "UPDATE drum_pads SET sample_id = ?1 WHERE track_id = ?2 AND pad_index = ?3",
                params![sid.get() as i64, track_id, pad_index],
            );
        }
    }
}

fn migrate_chopper_states(conn: &Connection, next_id: &mut u32) {
    let rows: Vec<(u32, String)> = conn
        .prepare("SELECT track_id, path FROM chopper_states WHERE path IS NOT NULL AND sample_id IS NULL")
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .and_then(|rows| rows.collect())
        })
        .unwrap_or_default();

    for (track_id, path) in rows {
        if let Some(sid) = try_import(conn, &path, next_id) {
            let _ = conn.execute(
                "UPDATE chopper_states SET sample_id = ?1 WHERE track_id = ?2",
                params![sid.get() as i64, track_id],
            );
        }
    }
}

fn migrate_convolution_ir(conn: &Connection, next_id: &mut u32) {
    let rows: Vec<(u32, String)> = conn
        .prepare("SELECT id, convolution_ir_path FROM tracks WHERE convolution_ir_path IS NOT NULL AND convolution_ir_sample_id IS NULL")
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .and_then(|rows| rows.collect())
        })
        .unwrap_or_default();

    for (track_id, path) in rows {
        if let Some(sid) = try_import(conn, &path, next_id) {
            let _ = conn.execute(
                "UPDATE tracks SET convolution_ir_sample_id = ?1 WHERE id = ?2",
                params![sid.get() as i64, track_id],
            );
        }
    }
}
