use asset_sync::bucket;
use asset_sync::manifest::{ManifestRepo, RepoError, SqliteRepo};
use asset_sync::roaring_bytes;
use asset_sync::schema::{asset_coverage_bitmap, asset_manifest};
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
