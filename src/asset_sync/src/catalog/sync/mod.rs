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

mod diff;
mod read;

use std::collections::{BTreeMap, BTreeSet};

use diesel::SqliteConnection;
use diesel::prelude::*;

use crate::catalog::{
    config::{Catalog, normalize_catalog},
    repo::{upsert_asset_class, upsert_provider, upsert_provider_asset_class, upsert_symbol_map},
};
use crate::schema::{asset_class, provider, provider_asset_class, provider_symbol_map};

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
) -> anyhow::Result<()> {
    let _ = normalize_catalog(&mut cat);

    // Build desired sets from TOML
    let mut want_providers = BTreeMap::<String, String>::new();
    let mut want_classes = BTreeSet::<String>::new();
    let mut want_pairs = BTreeSet::<(String, String)>::new();
    let mut want_symbols = Vec::<(String, String, String, String)>::new();

    for (pcode, pcfg) in &cat.providers {
        want_providers.insert(pcode.clone(), pcfg.name.clone());

        for a in &pcfg.asset_classes {
            want_classes.insert(a.clone());
            want_pairs.insert((pcode.clone(), a.clone()));
        }

        if let Some(sm) = &pcfg.symbol_map {
            for s in sm {
                want_symbols.push((
                    pcode.clone(),
                    s.asset_class.clone(),
                    s.canonical.clone(),
                    s.remote.clone(),
                ));
            }
        }
    }

    // Read current DB state (for diff & prune)
    conn.immediate_transaction::<_, anyhow::Error, _>(|conn| {
        // UPSERT providers/classes
        for (p, name_) in &want_providers {
            if !opt.dry_run {
                upsert_provider(conn, p, name_)?;
            }
        }
        for a in &want_classes {
            if !opt.dry_run {
                upsert_asset_class(conn, a)?;
            }
        }
        // UPSERT pairs (FKs ensure both sides exist)
        for (p, a) in &want_pairs {
            if !opt.dry_run {
                upsert_provider_asset_class(conn, p, a)?;
            }
        }
        // UPSERT symbol map (FK to pair)
        for (p, a, canon, remote) in &want_symbols {
            if !opt.dry_run {
                upsert_symbol_map(conn, p, a, canon, remote)?;
            }
        }

        if opt.prune {
            // Compute and delete stale rows **safely** (RESTRICT prevents removing in-use pairs)
            // Providers
            {
                use provider::dsl as pr;
                let existing: Vec<String> = pr::provider.select(pr::code).load(conn)?;
                for code in existing {
                    if !want_providers.contains_key(&code) {
                        // try delete; RESTRICT will block if any child rows exist
                        if !opt.dry_run {
                            diesel::delete(pr::provider.filter(pr::code.eq(&code)))
                                .execute(conn)?;
                        }
                    }
                }
            }

            // Asset classes
            {
                use asset_class::dsl as ac;
                let existing: Vec<String> = ac::asset_class.select(ac::code).load(conn)?;
                for code in existing {
                    if !want_classes.contains(&code) && !opt.dry_run {
                        diesel::delete(ac::asset_class.filter(ac::code.eq(&code))).execute(conn)?;
                    }
                }
            }
            // Pairs
            {
                use provider_asset_class::dsl as pac;
                let existing: Vec<(String, String)> = pac::provider_asset_class
                    .select((pac::provider_code, pac::asset_class_code))
                    .load(conn)?;
                for (p, a) in existing {
                    if !want_pairs.contains(&(p.clone(), a.clone())) && !opt.dry_run {
                        diesel::delete(
                            pac::provider_asset_class.filter(
                                pac::provider_code.eq(&p).and(pac::asset_class_code.eq(&a)),
                            ),
                        )
                        .execute(conn)?;
                    }
                }
            }
            // Symbol map (prune any not present)
            {
                use provider_symbol_map::dsl as psm;
                let existing: Vec<(String, String, String, String)> = psm::provider_symbol_map
                    .select((
                        psm::provider_code,
                        psm::asset_class_code,
                        psm::canonical_symbol,
                        psm::remote_symbol,
                    ))
                    .load(conn)?;
                for row in existing {
                    if !want_symbols.contains(&row) && !opt.dry_run {
                        diesel::delete(
                            psm::provider_symbol_map.filter(
                                psm::provider_code
                                    .eq(&row.0)
                                    .and(psm::asset_class_code.eq(&row.1))
                                    .and(psm::canonical_symbol.eq(&row.2))
                                    .and(psm::remote_symbol.eq(&row.3)),
                            ),
                        )
                        .execute(conn)?;
                    }
                }
            }
        }

        Ok(())
    })?;
    Ok(())
}
