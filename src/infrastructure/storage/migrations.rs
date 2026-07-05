use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::error::MuonError;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(conn: &mut diesel::SqliteConnection) -> Result<(), MuonError> {
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| MuonError::Database(e.to_string()))?;
    Ok(())
}
