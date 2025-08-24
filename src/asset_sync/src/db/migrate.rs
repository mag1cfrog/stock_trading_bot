//! set up migrations

use anyhow::anyhow;
use diesel::{Connection, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::db::connection::connect_sqlite;

/// Embedded Diesel migrations bundled with this crate.
///
/// These are applied by `run_sqlite` to bring the database schema up to date.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Runs pending Diesel migrations on a SQLite database at the given URL.
///
/// This sets the SQLite journal mode to WAL and applies all embedded migrations, returning an error on failure.
pub fn run_sqlite(url: &str) -> anyhow::Result<()> {
    // Reuse the centralized PRAGMAs (WAL, FKs, busy_timeout)
    let mut conn = connect_sqlite(url)?;

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

/// Runs pending migrations for the given database URL by delegating to the appropriate backend.
///
/// Accepts URLs that start with "postgres://" or "postgresql://" for PostgreSQL and "sqlite:" for SQLite,
/// returning an error if the URL scheme is not recognized.
pub fn run_all(database_url: &str) -> anyhow::Result<()> {
    if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        run_postgres(database_url)
    } else {
        // Treat anything else as SQLite (supports bare file paths like "dev.db")
        run_sqlite(database_url)
    }
}
