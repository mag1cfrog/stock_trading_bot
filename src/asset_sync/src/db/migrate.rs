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

/// Runs pending migrations for the given database URL by delegating to the appropriate backend.
///
/// Accepts URLs that start with "postgres://" or "postgresql://" for PostgreSQL and "sqlite:" for SQLite,
/// returning an error if the URL scheme is not recognized.
pub fn run_all(database_url: &str) -> anyhow::Result<()> {
    if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        run_postgres(database_url)
    } else if database_url.starts_with("sqlite:") {
        run_sqlite(database_url)
    } else {
        anyhow::bail!("Unsupported DATABASE_URL: {database_url}");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn migrations_apply_on_temp_file() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let path = temp.path().to_string_lossy().to_string();

        crate::db::migrate::run_sqlite(&path).expect("migration run");

        let mut conn = SqliteConnection::establish(&path).unwrap();

        conn.batch_execute("INSERT INTO engine_kv (k,v) VALUES ('hello', 'world')").unwrap();
    }
}
