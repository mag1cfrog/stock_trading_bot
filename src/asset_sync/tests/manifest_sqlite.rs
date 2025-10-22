use asset_sync::bucket;
use asset_sync::manifest::{ManifestRepo, RepoError, SqliteRepo};
use asset_sync::roaring_bytes;
use asset_sync::schema::{asset_coverage_bitmap, asset_gaps, asset_manifest};
use asset_sync::spec::{AssetSpec, ProviderId, Range};
use asset_sync::timeframe::{Timeframe as RepoTimeframe, TimeframeUnit as RepoTimeframeUnit};
use asset_sync::tz;
use chrono::{Duration, TimeZone, Utc};
use diesel::Queryable;
use diesel::prelude::*;
use market_data_ingestor::models::{
    asset::AssetClass,
    timeframe::{TimeFrame, TimeFrameUnit},
};
use roaring::RoaringBitmap;
use std::num::NonZeroU32;

mod common;

#[derive(Debug, Queryable)]
struct ManifestProjection {
    symbol: String,
    provider_code: String,
    asset_class_code: String,
    timeframe_amount: i32,
    timeframe_unit: String,
    desired_start: String,
    desired_end: Option<String>,
    watermark: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Queryable)]
struct CoverageProjection {
    manifest_id: i32,
    bitmap: Vec<u8>,
    version: i32,
}

#[derive(Debug, Queryable)]
struct GapProjection {
    id: i32,
    state: String,
    lease_owner: Option<String>,
    lease_expires_at: Option<String>,
}

#[derive(Debug, Queryable)]
struct GapFullProjection {
    _id: i32,
    manifest_id: i32,
    start_ts: String,
    end_ts: String,
    state: String,
    lease_owner: Option<String>,
    lease_expires_at: Option<String>,
}

#[test]
fn upsert_manifest_inserts_open_range_and_creates_coverage() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");
    common::assert_sqlite_pragmas(&mut conn);

    let repo = SqliteRepo::new();
    let start = Utc.with_ymd_and_hms(2024, 1, 2, 9, 30, 0).unwrap();
    let spec = AssetSpec {
        symbol: "AAPL".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(5, TimeFrameUnit::Minute),
        range: Range::Open { start },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");
    assert!(manifest_id > 0);

    use asset_manifest::dsl as am;
    let row: ManifestProjection = am::asset_manifest
        .find(manifest_id as i32)
        .select((
            am::symbol,
            am::provider_code,
            am::asset_class_code,
            am::timeframe_amount,
            am::timeframe_unit,
            am::desired_start,
            am::desired_end,
            am::watermark,
            am::last_error,
        ))
        .first(&mut conn)
        .expect("manifest row");

    assert_eq!(row.symbol, "AAPL");
    assert_eq!(row.provider_code, "alpaca");
    assert_eq!(row.asset_class_code, "us_equity");
    assert_eq!(row.timeframe_amount, 5);
    assert_eq!(row.timeframe_unit, "Minute");
    assert_eq!(row.desired_start, tz::to_rfc3339_millis(start));
    assert_eq!(row.desired_end, None);
    assert!(row.watermark.is_none());
    assert!(row.last_error.is_none());

    use asset_coverage_bitmap::dsl as acb;
    let coverage: CoverageProjection = acb::asset_coverage_bitmap
        .filter(acb::manifest_id.eq(manifest_id as i32))
        .select((acb::manifest_id, acb::bitmap, acb::version))
        .first(&mut conn)
        .expect("coverage row");

    assert_eq!(coverage.manifest_id, manifest_id as i32);
    assert_eq!(coverage.version, 0);
    let expected_bytes = roaring_bytes::rb_to_bytes(&RoaringBitmap::new());
    assert_eq!(coverage.bitmap, expected_bytes);

    common::fk_check_empty(&mut conn);
}

#[test]
fn upsert_manifest_conflict_preserves_progress_and_updates_range() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let start = Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "MSFT".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(2, TimeFrameUnit::Hour),
        range: Range::Open { start },
    };

    let first_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    // Simulate prior progress and an error to verify the upsert leaves them untouched.
    use asset_manifest::dsl as am;
    let watermark_value = tz::to_rfc3339_millis(start + Duration::days(1));
    diesel::update(am::asset_manifest.find(first_id as i32))
        .set((
            am::watermark.eq(Some(watermark_value.clone())),
            am::last_error.eq(Some("boom".to_string())),
        ))
        .execute(&mut conn)
        .expect("seed state");

    let end = start + Duration::days(10);
    let closed_spec = AssetSpec {
        range: Range::Closed { start, end },
        ..spec.clone()
    };

    let second_id = repo
        .upsert_manifest(&mut conn, &closed_spec)
        .expect("update manifest");
    assert_eq!(second_id, first_id);

    let row: ManifestProjection = am::asset_manifest
        .find(first_id as i32)
        .select((
            am::symbol,
            am::provider_code,
            am::asset_class_code,
            am::timeframe_amount,
            am::timeframe_unit,
            am::desired_start,
            am::desired_end,
            am::watermark,
            am::last_error,
        ))
        .first(&mut conn)
        .expect("manifest row");

    assert_eq!(row.symbol, "MSFT");
    assert_eq!(row.provider_code, "alpaca");
    assert_eq!(row.asset_class_code, "us_equity");
    assert_eq!(row.timeframe_amount, 2);
    assert_eq!(row.timeframe_unit, "Hour");
    assert_eq!(row.desired_start, tz::to_rfc3339_millis(start));
    assert_eq!(row.desired_end, Some(tz::to_rfc3339_millis(end)));
    assert_eq!(row.watermark.as_deref(), Some(watermark_value.as_str()));
    assert_eq!(row.last_error.as_deref(), Some("boom"));

    use asset_coverage_bitmap::dsl as acb;
    let coverage_rows: Vec<CoverageProjection> = acb::asset_coverage_bitmap
        .select((acb::manifest_id, acb::bitmap, acb::version))
        .load(&mut conn)
        .expect("coverage rows");
    assert_eq!(coverage_rows.len(), 1);
    assert_eq!(coverage_rows[0].manifest_id, first_id as i32);
    assert_eq!(coverage_rows[0].version, 0);

    common::fk_check_empty(&mut conn);
}

#[test]
fn upsert_manifest_supports_multiple_asset_classes() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    use asset_sync::schema::asset_class::dsl as ac;
    use asset_sync::schema::provider_asset_class::dsl as pac;

    diesel::insert_into(ac::asset_class)
        .values(ac::code.eq("futures"))
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .expect("seed futures asset class");

    diesel::insert_into(pac::provider_asset_class)
        .values((
            pac::provider_code.eq("alpaca"),
            pac::asset_class_code.eq("futures"),
        ))
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .expect("link futures class");

    let repo = SqliteRepo::new();
    let start = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "ESU25".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::Futures,
        timeframe: TimeFrame::new(1, TimeFrameUnit::Day),
        range: Range::Closed {
            start,
            end: start + Duration::days(30),
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert futures manifest");
    assert!(manifest_id > 0);

    use asset_manifest::dsl as am;
    let row: ManifestProjection = am::asset_manifest
        .find(manifest_id as i32)
        .select((
            am::symbol,
            am::provider_code,
            am::asset_class_code,
            am::timeframe_amount,
            am::timeframe_unit,
            am::desired_start,
            am::desired_end,
            am::watermark,
            am::last_error,
        ))
        .first(&mut conn)
        .expect("manifest row");

    assert_eq!(row.symbol, "ESU25");
    assert_eq!(row.asset_class_code, "futures");
    assert_eq!(row.timeframe_unit, "Day");
    assert_eq!(row.timeframe_amount, 1);

    common::fk_check_empty(&mut conn);
}

#[test]
fn coverage_get_returns_empty_for_unknown_manifest() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let (bitmap, version) = repo.coverage_get(&mut conn, 42).expect("coverage get");

    assert!(bitmap.is_empty());
    assert_eq!(version, 0);
    common::fk_check_empty(&mut conn);
}

#[test]
fn coverage_get_reads_existing_bitmap_and_version() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let start = Utc.with_ymd_and_hms(2024, 7, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "NFLX".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(15, TimeFrameUnit::Minute),
        range: Range::Open { start },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    use asset_coverage_bitmap::dsl as acb;
    let mut expected_bitmap = RoaringBitmap::new();
    expected_bitmap.insert(3);
    expected_bitmap.insert(4);
    expected_bitmap.insert(10);
    let bytes = roaring_bytes::rb_to_bytes(&expected_bitmap);

    diesel::update(acb::asset_coverage_bitmap.filter(acb::manifest_id.eq(manifest_id as i32)))
        .set((acb::bitmap.eq(bytes), acb::version.eq(5)))
        .execute(&mut conn)
        .expect("seed bitmap");

    let (bitmap, version) = repo
        .coverage_get(&mut conn, manifest_id)
        .expect("coverage get");

    assert_eq!(version, 5);
    assert_eq!(bitmap, expected_bitmap);
    common::fk_check_empty(&mut conn);
}

#[test]
fn coverage_put_updates_bitmap_and_version() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let start = Utc.with_ymd_and_hms(2024, 8, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "AMZN".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(30, TimeFrameUnit::Minute),
        range: Range::Open { start },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let mut bitmap = RoaringBitmap::new();
    bitmap.insert(1);
    bitmap.insert(2);
    bitmap.insert(32);

    let version = repo
        .coverage_put(&mut conn, manifest_id, &bitmap, 0)
        .expect("coverage put");
    assert_eq!(version, 1);

    use asset_coverage_bitmap::dsl as acb;
    let stored: CoverageProjection = acb::asset_coverage_bitmap
        .filter(acb::manifest_id.eq(manifest_id as i32))
        .select((acb::manifest_id, acb::bitmap, acb::version))
        .first(&mut conn)
        .expect("coverage row");

    assert_eq!(stored.version, 1);
    assert_eq!(stored.bitmap, roaring_bytes::rb_to_bytes(&bitmap));
    common::fk_check_empty(&mut conn);
}

#[test]
fn coverage_put_conflict_on_stale_version() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let start = Utc.with_ymd_and_hms(2024, 9, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "TSLA".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(1, TimeFrameUnit::Hour),
        range: Range::Open { start },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let mut initial = RoaringBitmap::new();
    initial.insert(5);
    initial.insert(8);
    repo.coverage_put(&mut conn, manifest_id, &initial, 0)
        .expect("initial put");

    let mut stale_attempt = RoaringBitmap::new();
    stale_attempt.insert(99);
    let err = repo
        .coverage_put(&mut conn, manifest_id, &stale_attempt, 0)
        .unwrap_err();
    let repo_err = err.downcast::<RepoError>().expect("repo error");
    match repo_err {
        RepoError::CoverageConflict { expected } => assert_eq!(expected, 0),
    }

    use asset_coverage_bitmap::dsl as acb;
    let stored: CoverageProjection = acb::asset_coverage_bitmap
        .filter(acb::manifest_id.eq(manifest_id as i32))
        .select((acb::manifest_id, acb::bitmap, acb::version))
        .first(&mut conn)
        .expect("coverage row");

    assert_eq!(stored.version, 1);
    assert_eq!(stored.bitmap, roaring_bytes::rb_to_bytes(&initial));
    common::fk_check_empty(&mut conn);
}

#[test]
fn coverage_put_conflict_when_manifest_missing() {
    let (_db, mut conn) = common::setup_db();
    let repo = SqliteRepo::new();

    let err = repo
        .coverage_put(&mut conn, 999, &RoaringBitmap::new(), 0)
        .unwrap_err();
    let repo_err = err.downcast::<RepoError>().expect("repo error");
    match repo_err {
        RepoError::CoverageConflict { expected } => assert_eq!(expected, 0),
    }

    common::fk_check_empty(&mut conn);
}

#[test]
fn compute_missing_returns_empty_when_window_end_not_after_start() {
    let (_db, mut conn) = common::setup_db();
    let repo = SqliteRepo::new();

    let window_start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let window_end = window_start;

    let missing = repo
        .compute_missing(&mut conn, 123, window_start, window_end)
        .expect("should short-circuit on empty window");

    assert!(missing.is_empty());
}

#[test]
fn compute_missing_errors_when_manifest_missing() {
    let (_db, mut conn) = common::setup_db();
    let repo = SqliteRepo::new();

    let window_start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let window_end = window_start + Duration::hours(1);

    let err = repo
        .compute_missing(&mut conn, 987, window_start, window_end)
        .expect_err("missing manifest should error");

    let msg = err.to_string();
    assert!(
        msg.contains("manifest 987 not found"),
        "unexpected error: {msg}"
    );
}

#[test]
fn compute_missing_returns_full_range_when_no_coverage() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 3, 10, 9, 30, 0).unwrap();
    let spec = AssetSpec {
        symbol: "AAPL".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(5, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let window_start = Utc.with_ymd_and_hms(2024, 3, 11, 9, 30, 0).unwrap();
    let window_end = window_start + Duration::minutes(20);

    let missing = repo
        .compute_missing(&mut conn, manifest_id, window_start, window_end)
        .expect("compute missing");

    let repo_tf = RepoTimeframe::new(NonZeroU32::new(5).unwrap(), RepoTimeframeUnit::Minute);
    let start_id = bucket::bucket_id(window_start, repo_tf);
    let end_id = bucket::bucket_id(window_end, repo_tf);
    let expected_start = bucket::bucket_start_utc(start_id, repo_tf);
    let expected_end = bucket::bucket_end_exclusive_utc(end_id, repo_tf);

    assert_eq!(missing, vec![(expected_start, expected_end)]);

    common::fk_check_empty(&mut conn);
}

#[test]
fn compute_missing_respects_existing_coverage_and_coalesces() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "MSFT".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(1, TimeFrameUnit::Hour),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let window_start = Utc.with_ymd_and_hms(2024, 1, 5, 0, 0, 0).unwrap();
    let window_end = window_start + Duration::hours(7);

    let repo_tf = RepoTimeframe::new(NonZeroU32::new(1).unwrap(), RepoTimeframeUnit::Hour);
    let base = bucket::bucket_id(window_start, repo_tf);
    let base_u32 = u32::try_from(base).expect("bucket fits in u32");

    let mut present = RoaringBitmap::new();
    for offset in [1, 2, 4] {
        present.insert(base_u32 + offset);
    }

    let bytes = roaring_bytes::rb_to_bytes(&present);
    use asset_coverage_bitmap::dsl as acb;
    diesel::update(acb::asset_coverage_bitmap.filter(acb::manifest_id.eq(manifest_id as i32)))
        .set((acb::bitmap.eq(bytes), acb::version.eq(3)))
        .execute(&mut conn)
        .expect("seed coverage");

    let missing = repo
        .compute_missing(&mut conn, manifest_id, window_start, window_end)
        .expect("compute missing");

    let (stored_bitmap, _) = repo
        .coverage_get(&mut conn, manifest_id)
        .expect("verify coverage");

    let start_id = bucket::bucket_id(window_start, repo_tf);
    let end_id = bucket::bucket_id(window_end, repo_tf);
    let start_id_u32 = u32::try_from(start_id).expect("window start fits in u32");
    let end_id_u32 = u32::try_from(end_id).expect("window end fits in u32");

    let mut window = RoaringBitmap::new();
    window.insert_range(start_id_u32..end_id_u32);

    let diff_ids: Vec<u32> = (&window - &stored_bitmap).iter().collect();

    let mut expected = Vec::new();
    if let Some(first) = diff_ids.first() {
        let mut run_start = *first as u64;
        let mut prev = *first as u64;
        for &id in &diff_ids[1..] {
            let id_u64 = id as u64;
            if id_u64 == prev + 1 {
                prev = id_u64;
                continue;
            }
            expected.push((
                bucket::bucket_start_utc(run_start, repo_tf),
                bucket::bucket_end_exclusive_utc(prev + 1, repo_tf),
            ));
            run_start = id_u64;
            prev = id_u64;
        }
        expected.push((
            bucket::bucket_start_utc(run_start, repo_tf),
            bucket::bucket_end_exclusive_utc(prev + 1, repo_tf),
        ));
    }

    assert_eq!(missing, expected);

    common::fk_check_empty(&mut conn);
}

#[test]
fn compute_missing_returns_empty_when_window_within_single_bucket() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "META".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(30, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let window_start = Utc.with_ymd_and_hms(2024, 6, 2, 0, 5, 0).unwrap();
    let window_end = window_start + Duration::minutes(10);

    let missing = repo
        .compute_missing(&mut conn, manifest_id, window_start, window_end)
        .expect("compute missing");

    assert!(missing.is_empty());

    common::fk_check_empty(&mut conn);
}

#[test]
fn compute_missing_errors_on_start_bucket_overflow() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "GOOG".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(1, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let overflow_start_secs = (u32::MAX as i64 + 1) * 60;
    let window_start = Utc.timestamp_opt(overflow_start_secs, 0).unwrap();
    let window_end = window_start + Duration::minutes(1);

    let err = repo
        .compute_missing(&mut conn, manifest_id, window_start, window_end)
        .expect_err("bucket id overflow should error");

    let msg = err.to_string();
    assert!(
        msg.contains("bucket id overflow (start)"),
        "unexpected error: {msg}"
    );

    common::fk_check_empty(&mut conn);
}

#[test]
fn compute_missing_errors_on_end_bucket_overflow() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "NVDA".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(1, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let base_secs = (u32::MAX as i64 - 1) * 60;
    let window_start = Utc.timestamp_opt(base_secs, 0).unwrap();
    let window_end = window_start + Duration::minutes(5);

    let err = repo
        .compute_missing(&mut conn, manifest_id, window_start, window_end)
        .expect_err("bucket id overflow should error");

    let msg = err.to_string();
    assert!(
        msg.contains("bucket id overflow (end)"),
        "unexpected error: {msg}"
    );

    common::fk_check_empty(&mut conn);
}

#[test]
fn gaps_complete_marks_row_done_and_preserves_leases() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "AAPL".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(15, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let gap_start = Utc.with_ymd_and_hms(2024, 4, 2, 0, 0, 0).unwrap();
    let gap_end = gap_start + Duration::hours(2);
    repo.gaps_upsert(&mut conn, manifest_id, &[(gap_start, gap_end)])
        .expect("insert gap");

    use asset_gaps::dsl as ag;
    let initial: GapProjection = ag::asset_gaps
        .select((ag::id, ag::state, ag::lease_owner, ag::lease_expires_at))
        .first(&mut conn)
        .expect("gap row");
    assert_eq!(initial.state, "queued");

    let lease_owner = "worker-42".to_string();
    let lease_expiry = tz::to_rfc3339_millis(gap_start + Duration::minutes(45));
    diesel::update(ag::asset_gaps.find(initial.id))
        .set((
            ag::state.eq("leased"),
            ag::lease_owner.eq(Some(lease_owner.clone())),
            ag::lease_expires_at.eq(Some(lease_expiry.clone())),
        ))
        .execute(&mut conn)
        .expect("lease gap");

    repo.gaps_complete(&mut conn, initial.id as i64)
        .expect("complete gap");

    let completed: GapProjection = ag::asset_gaps
        .find(initial.id)
        .select((ag::id, ag::state, ag::lease_owner, ag::lease_expires_at))
        .first(&mut conn)
        .expect("completed gap");

    assert_eq!(completed.state, "done");
    assert_eq!(completed.lease_owner.as_deref(), Some(lease_owner.as_str()));
    assert_eq!(
        completed.lease_expires_at.as_deref(),
        Some(lease_expiry.as_str())
    );

    repo.gaps_complete(&mut conn, initial.id as i64)
        .expect("idempotent completion");

    let done_again: GapProjection = ag::asset_gaps
        .find(initial.id)
        .select((ag::id, ag::state, ag::lease_owner, ag::lease_expires_at))
        .first(&mut conn)
        .expect("gap after second completion");

    assert_eq!(done_again.state, "done");
    assert_eq!(
        done_again.lease_owner.as_deref(),
        Some(lease_owner.as_str())
    );
    assert_eq!(
        done_again.lease_expires_at.as_deref(),
        Some(lease_expiry.as_str())
    );

    common::fk_check_empty(&mut conn);
}

#[test]
fn gaps_complete_errors_when_gap_missing() {
    let (_db, mut conn) = common::setup_db();
    let repo = SqliteRepo::new();

    let err = repo
        .gaps_complete(&mut conn, 12345)
        .expect_err("missing gap should error");

    let msg = err.to_string();
    assert!(
        msg.contains("gap not found: 12345"),
        "unexpected error: {msg}"
    );

    common::fk_check_empty(&mut conn);
}

#[test]
fn gaps_lease_returns_empty_when_limit_non_positive() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "AAPL".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(5, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let gap_start = desired_start + Duration::hours(1);
    repo.gaps_upsert(
        &mut conn,
        manifest_id,
        &[(gap_start, gap_start + Duration::minutes(30))],
    )
    .expect("insert gap");

    let leased = repo
        .gaps_lease(&mut conn, "worker", 0, Duration::minutes(15))
        .expect("lease call with zero limit");
    assert!(leased.is_empty());

    use asset_gaps::dsl as ag;
    let row: GapProjection = ag::asset_gaps
        .select((ag::id, ag::state, ag::lease_owner, ag::lease_expires_at))
        .first(&mut conn)
        .expect("gap row");
    assert_eq!(row.state, "queued");
    assert!(row.lease_owner.is_none());
    assert!(row.lease_expires_at.is_none());

    common::fk_check_empty(&mut conn);
}

#[test]
fn gaps_lease_leases_rows_up_to_limit_in_order() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 5, 10, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "MSFT".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(15, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let mut ranges = Vec::new();
    for offset in 0..3 {
        let start = desired_start + Duration::hours(offset * 2);
        ranges.push((start, start + Duration::hours(1)));
    }
    repo.gaps_upsert(&mut conn, manifest_id, &ranges)
        .expect("insert gaps");

    use asset_gaps::dsl as ag;
    let ids: Vec<i32> = ag::asset_gaps
        .order(ag::id.asc())
        .select(ag::id)
        .load(&mut conn)
        .expect("gap ids");
    assert_eq!(ids.len(), 3);

    let ttl = Duration::minutes(45);
    let before = Utc::now();
    let leased = repo
        .gaps_lease(&mut conn, "worker-1", 2, ttl)
        .expect("lease gaps");
    let after = Utc::now();

    assert_eq!(leased, vec![ids[0] as i64, ids[1] as i64]);

    let rows: Vec<GapProjection> = ag::asset_gaps
        .order(ag::id.asc())
        .select((ag::id, ag::state, ag::lease_owner, ag::lease_expires_at))
        .load(&mut conn)
        .expect("gap rows");

    for (idx, row) in rows.iter().enumerate() {
        if idx < 2 {
            assert_eq!(row.state, "leased");
            assert_eq!(row.lease_owner.as_deref(), Some("worker-1"));
            let expires = tz::parse_ts_to_utc(row.lease_expires_at.as_deref().unwrap())
                .expect("parse expiry");
            let lower_bound = (before + ttl) - Duration::seconds(5);
            let upper_bound = after + ttl + Duration::seconds(5);
            assert!(expires >= lower_bound);
            assert!(expires <= upper_bound);
        } else {
            assert_eq!(row.state, "queued");
            assert!(row.lease_owner.is_none());
            assert!(row.lease_expires_at.is_none());
        }
    }

    common::fk_check_empty(&mut conn);
}

#[test]
fn gaps_lease_reacquires_gap_when_previous_lease_expired() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "AMZN".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(30, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let range = (desired_start, desired_start + Duration::hours(3));
    repo.gaps_upsert(&mut conn, manifest_id, &[range])
        .expect("insert gap");

    let ttl = Duration::minutes(20);
    let first = repo
        .gaps_lease(&mut conn, "worker-old", 1, ttl)
        .expect("first lease");
    assert_eq!(first.len(), 1);

    use asset_gaps::dsl as ag;
    let gap_id = first[0] as i32;
    let expired_ts = tz::to_rfc3339_millis(Utc::now() - Duration::minutes(5));
    diesel::update(ag::asset_gaps.find(gap_id))
        .set((
            ag::state.eq("queued"),
            ag::lease_owner.eq(Some("worker-old".to_string())),
            ag::lease_expires_at.eq(Some(expired_ts)),
        ))
        .execute(&mut conn)
        .expect("reset gap to queued with stale lease");

    let before = Utc::now();
    let second = repo
        .gaps_lease(&mut conn, "worker-new", 1, ttl)
        .expect("second lease");
    let after = Utc::now();
    assert_eq!(second, vec![gap_id as i64]);

    let row: GapProjection = ag::asset_gaps
        .find(gap_id)
        .select((ag::id, ag::state, ag::lease_owner, ag::lease_expires_at))
        .first(&mut conn)
        .expect("gap row after re-lease");

    assert_eq!(row.state, "leased");
    assert_eq!(row.lease_owner.as_deref(), Some("worker-new"));
    let expires =
        tz::parse_ts_to_utc(row.lease_expires_at.as_deref().unwrap()).expect("parse expiry");
    let lower_bound = (before + ttl) - Duration::seconds(5);
    let upper_bound = after + ttl + Duration::seconds(5);
    assert!(expires >= lower_bound);
    assert!(expires <= upper_bound);

    common::fk_check_empty(&mut conn);
}

#[test]
fn gaps_lease_ignores_rows_not_in_queued_state() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 7, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "TSLA".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(1, TimeFrameUnit::Hour),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    repo.gaps_upsert(
        &mut conn,
        manifest_id,
        &[(desired_start, desired_start + Duration::hours(2))],
    )
    .expect("insert gap");

    use asset_gaps::dsl as ag;
    let row: GapProjection = ag::asset_gaps
        .select((ag::id, ag::state, ag::lease_owner, ag::lease_expires_at))
        .first(&mut conn)
        .expect("gap row");

    let future_expiry = tz::to_rfc3339_millis(Utc::now() + Duration::minutes(15));
    diesel::update(ag::asset_gaps.find(row.id))
        .set((
            ag::state.eq("leased"),
            ag::lease_owner.eq(Some("worker-existing".to_string())),
            ag::lease_expires_at.eq(Some(future_expiry.clone())),
        ))
        .execute(&mut conn)
        .expect("mark as leased");

    let leased = repo
        .gaps_lease(&mut conn, "worker", 5, Duration::minutes(10))
        .expect("lease attempt");
    assert!(leased.is_empty());

    let stored: GapProjection = ag::asset_gaps
        .find(row.id)
        .select((ag::id, ag::state, ag::lease_owner, ag::lease_expires_at))
        .first(&mut conn)
        .expect("gap after skipped lease");

    assert_eq!(stored.state, "leased");
    assert_eq!(stored.lease_owner.as_deref(), Some("worker-existing"));
    assert_eq!(
        stored.lease_expires_at.as_deref(),
        Some(future_expiry.as_str())
    );

    common::fk_check_empty(&mut conn);
}

#[test]
fn gaps_upsert_noop_on_empty_ranges() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 8, 1, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "AAPL".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(5, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    repo.gaps_upsert(&mut conn, manifest_id, &[])
        .expect("upsert with empty ranges");

    assert_eq!(common::count(&mut conn, "asset_gaps"), 0);

    common::fk_check_empty(&mut conn);
}

#[test]
fn gaps_upsert_inserts_rows_with_defaults() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 8, 2, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "MSFT".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(15, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let ranges = [
        (desired_start, desired_start + Duration::minutes(45)),
        (
            desired_start + Duration::hours(2),
            desired_start + Duration::hours(3),
        ),
    ];

    repo.gaps_upsert(&mut conn, manifest_id, &ranges)
        .expect("insert gaps");

    use asset_gaps::dsl as ag;
    let rows: Vec<GapFullProjection> = ag::asset_gaps
        .order(ag::start_ts.asc())
        .select((
            ag::id,
            ag::manifest_id,
            ag::start_ts,
            ag::end_ts,
            ag::state,
            ag::lease_owner,
            ag::lease_expires_at,
        ))
        .load(&mut conn)
        .expect("gap rows");

    assert_eq!(rows.len(), 2);
    for (row, (start, end)) in rows.iter().zip(ranges.iter()) {
        assert_eq!(row.manifest_id, manifest_id as i32);
        assert_eq!(row.state, "queued");
        assert!(row.lease_owner.is_none());
        assert!(row.lease_expires_at.is_none());
        assert_eq!(row.start_ts, tz::to_rfc3339_millis(*start));
        assert_eq!(row.end_ts, tz::to_rfc3339_millis(*end));
    }

    common::fk_check_empty(&mut conn);
}

#[test]
fn gaps_upsert_ignores_duplicate_ranges() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 8, 3, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "AMZN".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(30, TimeFrameUnit::Minute),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let primary = (desired_start, desired_start + Duration::hours(2));
    let secondary = (
        desired_start + Duration::hours(4),
        desired_start + Duration::hours(5),
    );

    repo.gaps_upsert(
        &mut conn,
        manifest_id,
        &[primary, secondary, primary, secondary],
    )
    .expect("insert with duplicates");

    use asset_gaps::dsl as ag;
    let rows: Vec<GapFullProjection> = ag::asset_gaps
        .order(ag::start_ts.asc())
        .select((
            ag::id,
            ag::manifest_id,
            ag::start_ts,
            ag::end_ts,
            ag::state,
            ag::lease_owner,
            ag::lease_expires_at,
        ))
        .load(&mut conn)
        .expect("gap rows");

    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows.iter()
            .map(|r| (r.start_ts.clone(), r.end_ts.clone()))
            .collect::<Vec<_>>(),
        vec![
            (
                tz::to_rfc3339_millis(primary.0),
                tz::to_rfc3339_millis(primary.1),
            ),
            (
                tz::to_rfc3339_millis(secondary.0),
                tz::to_rfc3339_millis(secondary.1),
            ),
        ]
    );

    common::fk_check_empty(&mut conn);
}

#[test]
fn gaps_upsert_handles_large_batches_with_chunking() {
    let (_db, mut conn) = common::setup_db();
    common::seed_min_catalog(&mut conn).expect("seed");

    let repo = SqliteRepo::new();
    let desired_start = Utc.with_ymd_and_hms(2024, 8, 4, 0, 0, 0).unwrap();
    let spec = AssetSpec {
        symbol: "TSLA".into(),
        provider: ProviderId::Alpaca,
        asset_class: AssetClass::UsEquity,
        timeframe: TimeFrame::new(1, TimeFrameUnit::Hour),
        range: Range::Open {
            start: desired_start,
        },
    };

    let manifest_id = repo
        .upsert_manifest(&mut conn, &spec)
        .expect("insert manifest");

    let mut ranges = Vec::new();
    for idx in 0..205 {
        let start = desired_start + Duration::minutes((idx * 10) as i64);
        let end = start + Duration::minutes(5);
        ranges.push((start, end));
    }

    repo.gaps_upsert(&mut conn, manifest_id, &ranges)
        .expect("insert large batch");

    assert_eq!(common::count(&mut conn, "asset_gaps"), ranges.len() as i64);

    common::fk_check_empty(&mut conn);
}
