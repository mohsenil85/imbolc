mod blob;
pub mod checkpoint;
pub mod load;
pub mod save;
pub mod schema;
#[cfg(test)]
mod tests;

pub use checkpoint::CheckpointInfo;

use std::path::Path;

use rusqlite::{Connection as SqlConnection, Result as SqlResult};

use super::instrument_state::InstrumentState;
use super::session::SessionState;

/// Save project using relational schema.
///
/// Uses WAL mode and an explicit transaction so the write is atomic:
/// if the process crashes mid-save the previous data remains intact.
/// Drops and recreates all tables on every save so schema changes
/// are applied automatically without migrations.
pub fn save_project(
    path: &Path,
    session: &SessionState,
    instruments: &InstrumentState,
) -> SqlResult<()> {
    let conn = SqlConnection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;

    let tx = conn.unchecked_transaction()?;
    schema::drop_all_tables(&tx)?;
    schema::create_tables(&tx)?;
    save::save_relational(&tx, session, instruments)?;
    tx.commit()?;

    Ok(())
}

/// Load project from relational format.
pub fn load_project(path: &Path) -> SqlResult<(SessionState, InstrumentState)> {
    let conn = SqlConnection::open(path)?;

    let (mut session, instruments) = load::load_relational(&conn)?;
    session.recompute_next_bus_id();
    Ok((session, instruments))
}
