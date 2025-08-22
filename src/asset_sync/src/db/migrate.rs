//! set up migrations 

use anyhow::anyhow;
use diesel::{connection::SimpleConnection, Connection, PgConnection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

/// Embedded Diesel migrations bundled with this crate.
/// 
/// These are applied by `run_sqlite` to bring the database schema up to date.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Runs pending Diesel migrations on a SQLite database at the given URL.
/// 
/// This sets the SQLite journal mode to WAL and applies all embedded migrations, returning an error on failure.
pub fn run_sqlite(url: &str) -> anyhow::Result<()> {
    let mut conn = SqliteConnection::establish(url)?;
    conn.batch_execute("PRAGMA journal_mode=WAL;")?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

/// Runs pending Diesel migrations on a PostgreSQL database at the given URL.
/// 
/// This connects to the database and applies all embedded migrations, returning an error on failure.
pub fn run_postgres(url: &str) -> anyhow::Result<()> {
    let mut conn = PgConnection::establish(url)?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow!(e))?;
    
    Ok(())
}