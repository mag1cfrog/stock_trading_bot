mod common;
use common::{count, fk_check_empty, setup_db};

use asset_sync::catalog::config::Catalog;
use asset_sync::catalog::sync::{SyncOptions, sync_catalog};
use asset_sync::schema;

use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};

fn tiny_toml() -> String {
    r#"
[providers.alpaca]
name = "Alpaca"
asset_classes = ["us_equity"]
  [[providers.alpaca.symbol_map]]
  asset_class = "us_equity"
  canonical   = "AAPL"
  remote      = "AAPL"

[providers.polygon]
name = "Polygon"
asset_classes = ["futures"]
"#
    .to_string()
}

#[test]
fn sync_happy_path_and_idempotent() {
    let (_db, mut conn) = setup_db();

    // Parse TOML → Catalog
    let cat: Catalog = toml::from_str(&tiny_toml()).unwrap();

    // First run (apply)
    let diff = sync_catalog(
        &mut conn,
        cat.clone(),
        SyncOptions {
            dry_run: false,
            prune: false,
        },
    )
    .expect("sync");

    // Sanity: some adds happened
    assert!(diff.providers_upsert.len() >= 2);
    assert!(diff.classes_upsert.len() >= 2);

    // Idempotence: second run is a no-op
    let diff2 = sync_catalog(
        &mut conn,
        cat,
        SyncOptions {
            dry_run: false,
            prune: false,
        },
    )
    .expect("sync-2");
    assert!(diff2.is_noop());

    // Verify counts
    assert_eq!(count(&mut conn, "provider"), 2);
    assert_eq!(count(&mut conn, "asset_class"), 2);
    assert_eq!(count(&mut conn, "provider_asset_class"), 2);
    assert_eq!(count(&mut conn, "provider_symbol_map"), 1);

    // FK integrity
    fk_check_empty(&mut conn);
}

#[test]
fn dry_run_does_not_write() {
    let (_db, mut conn) = setup_db();

    let cat: Catalog = toml::from_str(&tiny_toml()).unwrap();

    let diff = sync_catalog(
        &mut conn,
        cat,
        SyncOptions {
            dry_run: true,
            prune: true,
        },
    )
    .expect("dry-run");

    // Diff should not be empty…
    assert!(!diff.is_noop());
    // …but DB remains empty.
    assert_eq!(count(&mut conn, "provider"), 0);
    assert_eq!(count(&mut conn, "asset_class"), 0);
    assert_eq!(count(&mut conn, "provider_asset_class"), 0);
    assert_eq!(count(&mut conn, "provider_symbol_map"), 0);
}

#[test]
fn symbol_map_upsert_updates_remote() {
    let (_db, mut conn) = setup_db();

    // First TOML with AAPL→AAPL
    let t1 = r#"
[providers.alpaca]
name = "Alpaca"
asset_classes = ["us_equity"]
  [[providers.alpaca.symbol_map]]
  asset_class = "us_equity"
  canonical   = "AAPL"
  remote      = "AAPL"
"#;
    let cat1: Catalog = toml::from_str(t1).unwrap();
    sync_catalog(
        &mut conn,
        cat1,
        SyncOptions {
            dry_run: false,
            prune: false,
        },
    )
    .unwrap();

    // Second TOML updates remote symbol to AAPL.X
    let t2 = r#"
[providers.alpaca]
name = "Alpaca"
asset_classes = ["us_equity"]
  [[providers.alpaca.symbol_map]]
  asset_class = "us_equity"
  canonical   = "AAPL"
  remote      = "AAPL.X"
"#;
    let cat2: Catalog = toml::from_str(t2).unwrap();
    sync_catalog(
        &mut conn,
        cat2,
        SyncOptions {
            dry_run: false,
            prune: false,
        },
    )
    .unwrap();

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        remote: String,
    }
    let row: Row = diesel::sql_query(
        "SELECT remote_symbol AS remote
         FROM provider_symbol_map
         WHERE provider_code='alpaca' AND asset_class_code='us_equity' AND canonical_symbol='AAPL'",
    )
    .get_result(&mut conn)
    .unwrap();
    assert_eq!(row.remote, "AAPL.X");

    fk_check_empty(&mut conn);
}

#[test]
fn prune_respects_fk_restrict() {
    let (_db, mut conn) = setup_db();

    // Seed catalog with a pair.
    let seed = r#"
[providers.alpaca]
name = "Alpaca"
asset_classes = ["us_equity"]
"#;
    let cat: Catalog = toml::from_str(seed).unwrap();
    sync_catalog(
        &mut conn,
        cat,
        SyncOptions {
            dry_run: false,
            prune: false,
        },
    )
    .unwrap();

    // Reference the pair from asset_manifest so RESTRICT will bite on prune.
    diesel::insert_into(schema::asset_manifest::table)
        .values((
            schema::asset_manifest::symbol.eq("AAPL"),
            schema::asset_manifest::provider_code.eq("alpaca"),
            schema::asset_manifest::asset_class_code.eq("us_equity"),
            schema::asset_manifest::timeframe_amount.eq(1),
            schema::asset_manifest::timeframe_unit.eq("Day"),
            schema::asset_manifest::desired_start.eq("2020-01-01T00:00:00Z"),
            schema::asset_manifest::desired_end.eq::<Option<&str>>(None),
            schema::asset_manifest::watermark.eq::<Option<&str>>(None),
            schema::asset_manifest::last_error.eq::<Option<&str>>(None),
        ))
        .execute(&mut conn)
        .unwrap();

    // New TOML *omits* the pair → prune should attempt delete and fail by FK.
    let prune_all = r#"
[providers.polygon]
name = "Polygon"
asset_classes = []
"#;
    let cat2: Catalog = toml::from_str(prune_all).unwrap();
    let err = sync_catalog(
        &mut conn,
        cat2,
        SyncOptions {
            dry_run: false,
            prune: true,
        },
    )
    .unwrap_err();

    // Diesel should surface a FK violation from SQLite.
    let msg = err.to_string();
    // Check it maps to a Diesel DB error of kind ForeignKeyViolation on other backends too.
    let is_fk = matches!(
        err.downcast_ref::<Error>(),
        Some(Error::DatabaseError(
            DatabaseErrorKind::ForeignKeyViolation,
            _
        ))
    );
    assert!(is_fk || msg.contains("foreign key constraint failed"));

    // Pair must still exist.
    assert_eq!(count(&mut conn, "provider_asset_class"), 1);
    fk_check_empty(&mut conn);
}

#[test]
fn cache_refreshes_after_sync() {
    let (_db, mut conn) = setup_db();

    // Before refresh, cache is empty → not allowed
    asset_sync::catalog::clear_allowed_cache();
    assert!(!asset_sync::catalog::is_allowed_provider_class(
        "alpaca",
        "us_equity"
    ));

    // Sync a catalog enabling (alpaca, us_equity)
    let cat: Catalog = toml::from_str(
        r#"[providers.alpaca]
           name="Alpaca"
           asset_classes=["us_equity"]"#,
    )
    .unwrap();
    let _ = sync_catalog(
        &mut conn,
        cat,
        SyncOptions {
            dry_run: false,
            prune: false,
        },
    )
    .unwrap();

    // sync_catalog should have refreshed the cache
    assert!(asset_sync::catalog::is_allowed_provider_class(
        "alpaca",
        "us_equity"
    ));
}
