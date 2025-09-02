#![allow(dead_code)]

use asset_sync::db::{connection, migrate}; // ensure these are `pub` in your crate
use diesel::QueryableByName;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Text};
use std::path::PathBuf;
use tempfile::TempDir;

use asset_sync::schema::{asset_class, provider, provider_asset_class};

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

pub fn seed_min_catalog(conn: &mut SqliteConnection) -> diesel::QueryResult<()> {
    // provider: 'alpaca'
    diesel::insert_into(provider::table)
        .values((provider::code.eq("alpaca"), provider::name.eq("Alpaca")))
        .on_conflict(provider::code)
        .do_nothing()
        .execute(conn)?;

    // asset_class: 'us_equity'
    diesel::insert_into(asset_class::table)
        .values(asset_class::code.eq("us_equity"))
        .on_conflict(asset_class::code)
        .do_nothing()
        .execute(conn)?;

    // link pair: ('alpaca','us_equity')
    diesel::insert_into(provider_asset_class::table)
        .values((
            provider_asset_class::provider_code.eq("alpaca"),
            provider_asset_class::asset_class_code.eq("us_equity"),
        ))
        .on_conflict((
            provider_asset_class::provider_code,
            provider_asset_class::asset_class_code,
        ))
        .do_nothing()
        .execute(conn)?;

    Ok(())
}

pub fn fk_check_empty(conn: &mut SqliteConnection) {
    // PRAGMA foreign_key_check returns rows if there are violations.
    // We assert there are none.
    #[derive(diesel::QueryableByName, Debug)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        table: String,
    }
    let rows: Vec<Row> = sql_query("PRAGMA foreign_key_check;")
        .load(conn)
        .expect("fk_check");

    assert!(rows.is_empty(), "foreign key check not empty: {rows:?}");
}

pub fn count(conn: &mut SqliteConnection, table: &str) -> i64 {
    #[derive(diesel::QueryableByName)]
    struct C {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        c: i64,
    }
    let q = format!("SELECT COUNT(*) AS c FROM {table}");
    diesel::sql_query(q).get_result::<C>(conn).unwrap().c
}
