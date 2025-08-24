//! SQLite connection helpers.
//!
//! Provides [`connect_sqlite`] that opens a connection and applies recommended PRAGMAs
//! for local development: WAL journaling, foreign_keys=ON, and a 5000ms busy_timeout.
//!
//! Example:
//! ```no_run
//! use asset_sync::db::connection::connect_sqlite;
//!
//! let path = std::env::temp_dir().join("asset_sync_example.db");
//! let _conn = connect_sqlite(path.to_str().unwrap()).expect("open sqlite");
//! ```

use diesel::{Connection, RunQueryDsl, SqliteConnection, sql_query};

/// Open a SQLite connection and apply conneciton-wide PRAGMAs.
pub fn connect_sqlite(database_url: &str) -> anyhow::Result<SqliteConnection> {
    let mut conn = SqliteConnection::establish(database_url)?;

    // Better read concurrency + nicer dev ergomonics
    sql_query("PRAGMA journal_mode=WAL;").execute(&mut conn)?;
    sql_query("PRAGMA foreign_keys=ON;").execute(&mut conn)?;
    sql_query("PRAGMA busy_timeout=5000;").execute(&mut conn)?;
    Ok(conn)
}
