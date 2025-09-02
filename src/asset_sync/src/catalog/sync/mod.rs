//! Catalog synchronization (providers, asset classes, allowed pairs, symbol map).
//!
//! ## What this does
//! - Parses a `Catalog` (TOML) and **normalizes** it (lowercase codes, trim, dedupe).
//! - Computes a **diff** between TOML (desired) and the DB (current).
//! - Applies the diff with UPSERTs (idempotent) and optional **prune** deletes.
//!
//! ## Transactions & consistency
//! Everything runs inside a single **`BEGIN IMMEDIATE`** transaction via
//! `SqliteConnection::immediate_transaction`. This reduces `SQLITE_BUSY` surprises and
//! ensures we either apply the whole diff or none of it.
//!
//! ## Dry-run
//! When `SyncOptions::dry_run` is `true`, we return a structured `CatalogDiff` and do
//! **not** write anything. Callers can pretty-print the diff or log it.
//!
//! ## Delete order (prune)
//! When pruning, we delete in dependency order: `provider_symbol_map` → `provider_asset_class`
//! → (`provider`, `asset_class`). This respects FKs with `ON DELETE RESTRICT`. We verify
//! referential integrity with `PRAGMA foreign_key_check` in tests.

mod apply;
mod diff;
mod read;
mod want;

use diesel::SqliteConnection;

use crate::catalog::config::{Catalog, normalize_catalog};
use crate::catalog::sync::apply::apply_diff;
use crate::catalog::sync::diff::CatalogDiff;
use crate::catalog::sync::diff::make_diff;
use crate::catalog::sync::read::read_current;
use crate::catalog::sync::want::wanted_from_catalog;

/// Options for catalog synchronization.
pub struct SyncOptions {
    /// If true, compute the diff only and print/log what would change.
    pub dry_run: bool,
    /// If true, delete rows from the DB that are not present in the TOML.
    pub prune: bool,
}

/// Sync the provider/asset-class catalog (and symbol map) into SQLite.
///
/// - Reads a TOML [`Catalog`], normalizes it, and UPSERTs providers, classes,
///   provider↔class pairs, and symbol mappings.
/// - When `opt.prune` is true, removes rows not present in the TOML.
/// - Runs in a single immediate transaction to reduce SQLITE_BUSY surprises.
pub fn sync_catalog(
    conn: &mut SqliteConnection,
    mut cat: Catalog,
    opt: SyncOptions,
) -> anyhow::Result<CatalogDiff> {
    let _ = normalize_catalog(&mut cat);

    let want = wanted_from_catalog(&cat);
    let cur = read_current(conn)?;
    let diff = make_diff(&want, &cur, opt.prune);

    if opt.dry_run {
        return Ok(diff);
    }

    // one-shot transactional apply, BEGIN IMMEDIATE
    conn.immediate_transaction::<_, anyhow::Error, _>(|tx| apply_diff(tx, &diff))?;

    Ok(diff)
}
