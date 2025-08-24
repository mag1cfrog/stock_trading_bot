#![allow(dead_code)]

use asset_sync::db::{connection, migrate}; // ensure these are `pub` in your crate
use diesel::QueryableByName;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use std::path::PathBuf;
use tempfile::TempDir;

#[derive(QueryableByName)]
struct JournalMode {
    #[diesel(sql_type = Text)]
    journal_mode: String,
}
#[derive(QueryableByName)]
struct ForeignKeys {
    #[diesel(sql_type = Integer)]
    foreign_keys: i32,
}
#[derive(QueryableByName)]
struct BusyTimeout {
    #[diesel(sql_type = Integer, column_name = "timeout")]
    busy_timeout: i32,
}

pub struct TestDb {
    _dir: TempDir,    // keep alive for the life of the test
    pub path: String, // <tmpdir>/test.db
}

pub fn setup_db() -> (TestDb, SqliteConnection) {
    let dir = TempDir::new().expect("tempdir");
    let mut p = PathBuf::from(dir.path());
    p.push("test.db");
    let path = p.to_string_lossy().to_string();

    // run migrations via your public API
    migrate::run_all(&path).expect("migrations");

    // open a connection with PRAGMAs applied
    let conn = connection::connect_sqlite(&path).expect("connect");
    (TestDb { _dir: dir, path }, conn)
}

pub fn assert_sqlite_pragmas(conn: &mut SqliteConnection) {
    use diesel::sql_query;

    let jm: JournalMode = sql_query("PRAGMA journal_mode;").get_result(conn).unwrap();
    assert_eq!(jm.journal_mode.to_lowercase(), "wal"); // WAL is persistent per DB file

    let fk: ForeignKeys = sql_query("PRAGMA foreign_keys;").get_result(conn).unwrap();
    assert_eq!(fk.foreign_keys, 1);

    let bt: BusyTimeout = sql_query("PRAGMA busy_timeout;").get_result(conn).unwrap();
    assert_eq!(bt.busy_timeout, 5000);
}
