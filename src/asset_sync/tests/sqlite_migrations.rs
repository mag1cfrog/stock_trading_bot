mod common;
use common::{assert_sqlite_pragmas, seed_min_catalog, setup_db};

use diesel::QueryableByName;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Text};

#[derive(QueryableByName)]
struct TblCnt {
    #[diesel(sql_type = Integer)]
    cnt: i32,
}
#[derive(QueryableByName)]
struct TimeStr {
    #[diesel(sql_type = Text)]
    t: String,
}

#[test]
fn migrations_apply_and_pragmas_are_set() {
    let (_db, mut conn) = setup_db();

    seed_min_catalog(&mut conn).expect("seed catalog");

    // PRAGMAs (WAL is a persistent property of the .db file; FKs/timeout are per-connection)
    assert_sqlite_pragmas(&mut conn);

    // Schema objects exist
    let tbls: TblCnt = sql_query(
        "SELECT COUNT(*) AS cnt
            FROM sqlite_master
            WHERE type='table'
            AND name IN ('asset_manifest','asset_coverage_bitmap','asset_gaps');",
    )
    .get_result(&mut conn)
    .unwrap();
    assert_eq!(tbls.cnt, 3, "expected three tables to be present");

    // --- Insert a manifest row and prove updated_at moves on UPDATE ---
    // created_at/updated_at default to CURRENT_TIMESTAMP in SQLite (second precision).
    sql_query(
        "
        INSERT INTO asset_manifest (
            symbol, provider, asset_class, timeframe_amount, timeframe_unit,
            desired_start, desired_end, watermark, last_error
        ) VALUES (
            'AAPL','alpaca','us_equity',1,'Minute',
            '2010-01-01T00:00:00Z', NULL, NULL, NULL
        );
    ",
    )
    .execute(&mut conn)
    .unwrap();

    let before: TimeStr =
        sql_query("SELECT updated_at AS t FROM asset_manifest WHERE symbol='AAPL' LIMIT 1;")
            .get_result(&mut conn)
            .unwrap();

    // Touch the row
    sql_query("UPDATE asset_manifest SET last_error='test' WHERE symbol='AAPL';")
        .execute(&mut conn)
        .unwrap();

    let after: TimeStr =
        sql_query("SELECT updated_at AS t FROM asset_manifest WHERE symbol='AAPL' LIMIT 1;")
            .get_result(&mut conn)
            .unwrap();

    assert_ne!(before.t, after.t, "updated_at should change on UPDATE");

    // Attempt to insert a coverage row referencing a non-existent manifest
    let orphan = diesel::sql_query(
        "INSERT INTO asset_coverage_bitmap (manifest_id, bitmap) VALUES (999999, x'00');",
    )
    .execute(&mut conn);
    assert!(orphan.is_err(), "FK should reject orphan coverage row");

    // Insert a manifest
    diesel::sql_query(
        "
  INSERT INTO asset_manifest (
    symbol, provider, asset_class, timeframe_amount, timeframe_unit,
    desired_start, desired_end, watermark, last_error
  ) VALUES (
    'CASCADE','alpaca','us_equity',1,'Minute','2010-01-01T00:00:00Z',NULL,NULL,NULL
  );
",
    )
    .execute(&mut conn)
    .unwrap();

    // Fetch its id (SQLite: simplest is select by unique identity)
    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        id: i32,
    }
    let mid: IdRow = diesel::sql_query(
        "
  SELECT id FROM asset_manifest WHERE symbol='CASCADE' LIMIT 1;
",
    )
    .get_result(&mut conn)
    .unwrap();

    // Insert a gap tied to this manifest
    diesel::sql_query(format!(
        "INSERT INTO asset_gaps (manifest_id, start_ts, end_ts, state)
   VALUES ({}, '2010-01-01T00:00:00Z', '2010-01-02T00:00:00Z', 'queued');",
        mid.id
    ))
    .execute(&mut conn)
    .unwrap();

    // Delete the manifest; dependent gaps should be removed if ON DELETE CASCADE
    diesel::sql_query(format!("DELETE FROM asset_manifest WHERE id={};", mid.id))
        .execute(&mut conn)
        .unwrap();

    // Count remaining gaps for that manifest_id
    #[derive(diesel::QueryableByName)]
    struct Cnt {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        c: i32,
    }
    let cnt: Cnt = diesel::sql_query(format!(
        "SELECT COUNT(*) AS c FROM asset_gaps WHERE manifest_id={};",
        mid.id
    ))
    .get_result(&mut conn)
    .unwrap();
    assert_eq!(cnt.c, 0, "gaps should be removed by ON DELETE CASCADE");
}
