mod common;
use common::{setup_db, assert_sqlite_pragmas};

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Text};
use diesel::QueryableByName;
use std::thread::sleep;
use std::time::Duration;

#[derive(QueryableByName)]
struct TblCnt { #[diesel(sql_type = Integer)] cnt: i32 }
#[derive(QueryableByName)]
struct TimeStr { #[diesel(sql_type = Text)] t: String }

#[test]
fn migrations_apply_and_pragmas_are_set() {
    let (_db, mut conn) = setup_db();

    // PRAGMAs (WAL is a persistent property of the .db file; FKs/timeout are per-connection)
    assert_sqlite_pragmas(&mut conn);

    // Schema objects exist
    let tbls: TblCnt = sql_query(
        "SELECT COUNT(*) AS cnt
            FROM sqlite_master
            WHERE type='table'
            AND name IN ('asset_manifest','asset_coverage_bitmap','asset_gaps');"
    ).get_result(&mut conn).unwrap();
    assert_eq!(tbls.cnt, 3, "expected three tables to be present");

    // --- Insert a manifest row and prove updated_at moves on UPDATE ---
    // created_at/updated_at default to CURRENT_TIMESTAMP in SQLite (second precision).
    sql_query("
        INSERT INTO asset_manifest (
            symbol, provider, asset_class, timeframe_amount, timeframe_unit,
            desired_start, desired_end, watermark, last_error
        ) VALUES (
            'AAPL','alpaca','us_equity',1,'Minute',
            '2010-01-01T00:00:00Z', NULL, NULL, NULL
        );
    ").execute(&mut conn).unwrap();

    let before: TimeStr =
        sql_query("SELECT updated_at AS t FROM asset_manifest WHERE symbol='AAPL' LIMIT 1;")
        .get_result(&mut conn).unwrap();

    // Sleep to avoid same-second equality
    sleep(Duration::from_millis(1100));

    // Touch the row
    sql_query("UPDATE asset_manifest SET last_error='test' WHERE symbol='AAPL';")
        .execute(&mut conn).unwrap();

    let after: TimeStr =
        sql_query("SELECT updated_at AS t FROM asset_manifest WHERE symbol='AAPL' LIMIT 1;")
        .get_result(&mut conn).unwrap();

    assert_ne!(before.t, after.t, "updated_at should change on UPDATE");
}