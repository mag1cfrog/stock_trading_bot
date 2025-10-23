use asset_sync::bucket;
use asset_sync::db::connection::connect_sqlite;
use asset_sync::manifest::{ManifestRepo, RepoError, SqliteRepo};
use asset_sync::roaring_bytes;
use asset_sync::schema::{asset_coverage_bitmap, asset_gaps};
use asset_sync::spec::{AssetSpec, ProviderId, Range};
use asset_sync::timeframe::{Timeframe as RepoTimeframe, TimeframeUnit as RepoTimeframeUnit};
use asset_sync::tz;
use chrono::{Duration, TimeZone, Utc};
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sql_query;
use roaring::RoaringBitmap;
use std::num::NonZeroU32;

mod common;

#[derive(Debug, Queryable)]
struct CoverageRow {
    _manifest_id: i32,
    bitmap: Vec<u8>,
    version: i32,
}

#[derive(Debug, Queryable)]
struct GapRow {
    id: i32,
    state: String,
    lease_owner: Option<String>,
    _lease_expires_at: Option<String>,
}

#[test]
fn sqlite_connection_applies_pragmas() {
    let (db, mut conn) = common::setup_db();
    common::assert_sqlite_pragmas(&mut conn);

    let mut second = connect_sqlite(&db.path).expect("connect second");
    common::assert_sqlite_pragmas(&mut second);

    drop(second);
    common::fk_check_empty(&mut conn);
}

#[test]
fn sqlite_manifest_coverage_round_trip() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2010, 1, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "AAPL".into(),
        provider: ProviderId::Alpaca,
        asset_class: market_data_ingestor::models::asset::AssetClass::UsEquity,
        timeframe: market_data_ingestor::models::timeframe::TimeFrame::new(
            1,
            market_data_ingestor::models::timeframe::TimeFrameUnit::Hour,
        ),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let window_start = desired_start;
    let window_end = desired_start + Duration::hours(4);

    let missing_initial = repo
        .compute_missing(&mut conn, manifest_id, window_start, window_end)
        .expect("compute missing initial");
    assert_eq!(missing_initial.len(), 1);
    let tf = RepoTimeframe::new(NonZeroU32::new(1).unwrap(), RepoTimeframeUnit::Hour);
    let base_id_u64 = bucket::bucket_id(window_start, tf);
    let expected_start = bucket::bucket_start_utc(base_id_u64, tf);
    let expected_end = bucket::bucket_end_exclusive_utc(base_id_u64 + 4, tf);
    assert_eq!(missing_initial[0], (expected_start, expected_end));

    let base_id = base_id_u64 as u32;

    let mut coverage = RoaringBitmap::new();
    coverage.insert(base_id);
    coverage.insert(base_id + 1);

    let version = repo
        .coverage_put(&mut conn, manifest_id, &coverage, 0)
        .expect("initial coverage put");
    assert_eq!(version, 1);

    use asset_coverage_bitmap::dsl as acb;
    let stored: CoverageRow = acb::asset_coverage_bitmap
        .filter(acb::manifest_id.eq(manifest_id as i32))
        .select((acb::manifest_id, acb::bitmap, acb::version))
        .first(&mut conn)
        .expect("coverage row");
    assert_eq!(stored.version, 1);
    assert_eq!(roaring_bytes::rb_from_bytes(&stored.bitmap), coverage);

    let err = repo
        .coverage_put(&mut conn, manifest_id, &coverage, 0)
        .unwrap_err();
    let repo_err = err.downcast::<RepoError>().expect("conflict error");
    match repo_err {
        RepoError::CoverageConflict { expected } => assert_eq!(expected, 0),
    }

    let (mut latest, latest_version) = repo
        .coverage_get(&mut conn, manifest_id)
        .expect("coverage get");
    assert_eq!(latest_version, 1);
    assert_eq!(latest, coverage);

    latest.insert(base_id + 2);
    latest.insert(base_id + 3);

    let new_version = repo
        .coverage_put(&mut conn, manifest_id, &latest, latest_version)
        .expect("second coverage put");
    assert_eq!(new_version, 2);

    let missing_after = repo
        .compute_missing(&mut conn, manifest_id, window_start, window_end)
        .expect("compute missing after coverage");
    assert!(missing_after.is_empty());

    common::fk_check_empty(&mut conn);
}

#[test]
fn sqlite_gaps_leasing_round_trip() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2015, 6, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "MSFT".into(),
        provider: ProviderId::Alpaca,
        asset_class: market_data_ingestor::models::asset::AssetClass::UsEquity,
        timeframe: market_data_ingestor::models::timeframe::TimeFrame::new(
            1,
            market_data_ingestor::models::timeframe::TimeFrameUnit::Hour,
        ),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let ranges = [
        (desired_start, desired_start + Duration::hours(1)),
        (
            desired_start + Duration::hours(1),
            desired_start + Duration::hours(2),
        ),
        (
            desired_start + Duration::hours(2),
            desired_start + Duration::hours(3),
        ),
    ];
    repo.gaps_upsert(&mut conn, manifest_id, &ranges)
        .expect("insert gaps");

    let ttl = Duration::minutes(30);
    let leased = repo
        .gaps_lease(&mut conn, "worker-a", 2, ttl)
        .expect("initial lease");
    assert_eq!(leased.len(), 2);

    use asset_gaps::dsl as ag;
    let mut rows: Vec<GapRow> = ag::asset_gaps
        .order(ag::id.asc())
        .select((ag::id, ag::state, ag::lease_owner, ag::lease_expires_at))
        .load(&mut conn)
        .expect("gap rows");

    for (idx, row) in rows.iter().enumerate() {
        if idx < 2 {
            assert_eq!(row.state, "leased");
            assert_eq!(row.lease_owner.as_deref(), Some("worker-a"));
        } else {
            assert_eq!(row.state, "queued");
            assert!(row.lease_owner.is_none());
        }
    }

    let expired = tz::to_rfc3339_millis(Utc::now() - Duration::minutes(10));
    diesel::update(ag::asset_gaps.find(rows[0].id))
        .set((
            ag::state.eq("queued"),
            ag::lease_owner.eq(Some("worker-a".to_string())),
            ag::lease_expires_at.eq(Some(expired.clone())),
        ))
        .execute(&mut conn)
        .expect("expire first gap");

    let leased_again = repo
        .gaps_lease(&mut conn, "worker-b", 2, ttl)
        .expect("second lease");
    assert_eq!(leased_again.len(), 2);
    assert!(leased_again.contains(&(rows[0].id as i64)));

    rows = ag::asset_gaps
        .order(ag::id.asc())
        .select((ag::id, ag::state, ag::lease_owner, ag::lease_expires_at))
        .load(&mut conn)
        .expect("gap rows after re-lease");

    let mut reassigned = 0;
    for row in rows {
        if leased_again.contains(&(row.id as i64)) {
            assert_eq!(row.state, "leased");
            assert_eq!(row.lease_owner.as_deref(), Some("worker-b"));
            reassigned += 1;
        }
    }
    assert_eq!(reassigned, leased_again.len());

    common::fk_check_empty(&mut conn);
}

#[test]
fn sqlite_begin_immediate_locking_smoke() {
    let (db, mut conn_a) = common::setup_db();
    let mut conn_b = connect_sqlite(&db.path).expect("connect second");

    sql_query("BEGIN IMMEDIATE;")
        .execute(&mut conn_a)
        .expect("begin immediate on first connection");

    let err = sql_query("BEGIN IMMEDIATE;").execute(&mut conn_b);
    assert!(err.is_err(), "expected second BEGIN IMMEDIATE to block");
    if let Err(e) = err {
        match e {
            DieselError::DatabaseError(DatabaseErrorKind::UnableToSendCommand, info) => {
                assert!(info.message().contains("database is locked"));
            }
            DieselError::DatabaseError(_, info) => {
                assert!(info.message().contains("database is locked"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    sql_query("ROLLBACK;")
        .execute(&mut conn_a)
        .expect("rollback first connection");

    sql_query("BEGIN IMMEDIATE;")
        .execute(&mut conn_b)
        .expect("begin immediate after release");
    sql_query("ROLLBACK;")
        .execute(&mut conn_b)
        .expect("rollback second connection");
}
