mod common;
use common::seed_min_catalog;

use asset_sync::db::{connection::connect_sqlite, migrate::run_sqlite};
use asset_sync::models::{AssetManifest, NewAssetManifest};
use asset_sync::schema::asset_manifest::dsl as am;
use diesel::prelude::*;
use tempfile::NamedTempFile;

#[test]
fn selectable_smoke_query_compiles_and_runs() {
    // temp file DB
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path().to_string_lossy().to_string();

    // apply migrations, then open with our PRAGMAs
    run_sqlite(&path).expect("migrations");
    let mut conn = connect_sqlite(&path).expect("connect");

    seed_min_catalog(&mut conn).expect("seed catalog");

    // insert one row (use your helper if you have one)
    let row = NewAssetManifest {
        symbol: "AAPL",
        provider: "alpaca",
        asset_class: "us_equity",
        timeframe_amount: 1,
        timeframe_unit: "Minute",
        desired_start: "2010-01-01T00:00:00Z",
        desired_end: None,
    };
    diesel::insert_into(am::asset_manifest)
        .values(&row)
        .execute(&mut conn)
        .unwrap();

    // the important part: .select(AssetManifest::as_select())
    let list = am::asset_manifest
        .select(AssetManifest::as_select())
        .load::<AssetManifest>(&mut conn)
        .unwrap();

    assert!(!list.is_empty());
}
