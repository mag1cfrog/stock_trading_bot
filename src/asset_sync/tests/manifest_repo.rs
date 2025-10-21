use asset_sync::manifest::{ManifestRepo, repo::SqliteRepo};
use asset_sync::roaring_bytes;
use asset_sync::schema::{asset_coverage_bitmap, asset_manifest};
use asset_sync::spec::{AssetSpec, ProviderId, Range};
use asset_sync::tz;
use chrono::{Duration, TimeZone, Utc};
use diesel::Queryable;
use diesel::prelude::*;
use market_data_ingestor::models::{
    asset::AssetClass,
    timeframe::{TimeFrame, TimeFrameUnit},
};
use roaring::RoaringBitmap;

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
