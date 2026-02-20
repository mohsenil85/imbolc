mod blob;
pub mod checkpoint;
pub mod load;
mod migrate;
pub mod sample_cache;
pub mod sample_store;
pub mod save;
pub mod schema;
#[cfg(test)]
mod tests;

pub use checkpoint::CheckpointInfo;
pub use sample_cache::SampleCache;

use std::path::Path;

use rusqlite::{Connection as SqlConnection, Result as SqlResult};

use super::session::SessionState;
use super::track_state::TrackState;

/// Save project using relational schema.
///
/// Uses WAL mode and an explicit transaction so the write is atomic:
/// if the process crashes mid-save the previous data remains intact.
/// Drops and recreates all tables on every save so schema changes
/// are applied automatically without migrations.
pub fn save_project(path: &Path, session: &SessionState, tracks: &TrackState) -> SqlResult<()> {
    let conn = SqlConnection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;

    // Ensure sample_blobs exists before drop_all (it's excluded from drops)
    sample_store::ensure_table(&conn)?;

    let tx = conn.unchecked_transaction()?;
    schema::drop_all_tables(&tx)?;
    schema::create_tables(&tx)?;
    save::save_relational(&tx, session, tracks)?;
    tx.commit()?;

    Ok(())
}

/// Load project from relational format.
pub fn load_project(path: &Path) -> SqlResult<(SessionState, TrackState)> {
    let conn = SqlConnection::open(path)?;

    // Ensure sample_blobs table exists (for legacy projects)
    sample_store::ensure_table(&conn)?;

    // Migrate legacy sample references (file paths -> blob IDs)
    migrate::migrate_legacy_samples(&conn)?;

    // Add duration_secs column to drum_pads if missing
    migrate::migrate_drum_pad_duration(&conn)?;

    let (mut session, tracks) = load::load_relational(&conn)?;
    session.recompute_next_bus_id();
    Ok((session, tracks))
}
