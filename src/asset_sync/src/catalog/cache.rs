//! Lock-free, read-mostly cache for allowed (provider, asset_class) pairs.
//!
//! Readers call [`is_allowed_provider_class`] which loads an `Arc<HashSet<..>>`
//! snapshot with no locking contention. Writers call [`refresh_allowed`] after
//! syncing the catalog to atomically swap in a new snapshot.
//!
//! Implementation notes:
//! - Uses `arc-swap` for atomic pointer swaps + cheap reads (no RwLock).
//! - Initializes to an empty set; until you call `refresh_allowed`, all lookups
//!   return `false`.
//! - We allocate on lookup (`String`) to keep the code simple. If it ever shows
//!   up hot in profiles, we can switch to a key newtype that supports borrowed
//!   lookups.

use std::{collections::HashSet, sync::Arc};

use crate::schema::provider_asset_class::dsl as pac;

use arc_swap::ArcSwap;
use diesel::prelude::*;
use once_cell::sync::Lazy;

/// Snapshot type held inside the cache.
type AllowedSet = HashSet<(String, String)>;

/// Global cache: starts as empty; refreshed by `refresh_allowed`.
static ALLOWED: Lazy<ArcSwap<AllowedSet>> = Lazy::new(|| ArcSwap::from_pointee(AllowedSet::new()));

/// Returns `true` if the (provider_code, asset_class_code) pair is currently allowed
/// according to the in-memory snapshot.
///
/// Fast path: one atomic load + a HashSet lookup. No database access.
///
/// Note: returns `false` until someone calls `refresh_allowed`.
pub fn is_allowed_provider_class(provider_code: &str, asset_class_code: &str) -> bool {
    // Cheap atomic load of the current Arc<AllowedSet>
    let snap = ALLOWED.load();
    // Simple approach: allocate a small tuple for the lookup.
    snap.contains(&(provider_code.to_string(), asset_class_code.to_string()))
}

/// Rebuilds the allowed pair set from the database and atomically swaps it in.
///
/// Call this after `catalog::sync` finishes, or at app start.
/// It’s safe to call from any thread; readers see either the old or new snapshot.
pub fn refresh_allowed(conn: &mut SqliteConnection) -> anyhow::Result<()> {
    // Load all pairs from provider_asset_class.
    let rows: Vec<(String, String)> = pac::provider_asset_class
        .select((pac::provider_code, pac::asset_class_code))
        .load(conn)?;

    let new_set: AllowedSet = rows.into_iter().collect();
    ALLOWED.store(Arc::new(new_set));
    Ok(())
}

/// Clears the cache to an empty set. Useful for tests.
pub fn clear_allowed_cache() {
    ALLOWED.store(Arc::new(AllowedSet::new()));
}

/// Returns an `Arc` snapshot (if a caller needs to iterate or inspect).
pub fn snapshot() -> Arc<AllowedSet> {
    ALLOWED.load_full()
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::prelude::*;
    use tempfile::NamedTempFile;

    use crate::db::{connection::connect_sqlite, migrate};

    #[test]
    fn allowed_cache_roundtrip() {
        // temp DB with schema
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        migrate::run_sqlite(&path).unwrap();
        let mut conn = connect_sqlite(&path).unwrap();

        // Seed one pair
        diesel::insert_into(crate::schema::provider::table)
            .values((
                crate::schema::provider::code.eq("alpaca"),
                crate::schema::provider::name.eq("Alpaca Markets"),
            ))
            .execute(&mut conn)
            .unwrap();
        diesel::insert_into(crate::schema::asset_class::table)
            .values(crate::schema::asset_class::code.eq("us_equity"))
            .execute(&mut conn)
            .unwrap();
        diesel::insert_into(crate::schema::provider_asset_class::table)
            .values((
                crate::schema::provider_asset_class::provider_code.eq("alpaca"),
                crate::schema::provider_asset_class::asset_class_code.eq("us_equity"),
            ))
            .execute(&mut conn)
            .unwrap();

        clear_allowed_cache();
        assert!(!is_allowed_provider_class("alpaca", "us_equity")); // empty snapshot

        refresh_allowed(&mut conn).unwrap();
        assert!(is_allowed_provider_class("alpaca", "us_equity"));

        // Add a new pair; prove readers don’t see it until refresh
        diesel::insert_into(crate::schema::asset_class::table)
            .values(crate::schema::asset_class::code.eq("futures"))
            .execute(&mut conn)
            .unwrap();
        diesel::insert_into(crate::schema::provider_asset_class::table)
            .values((
                crate::schema::provider_asset_class::provider_code.eq("alpaca"),
                crate::schema::provider_asset_class::asset_class_code.eq("futures"),
            ))
            .execute(&mut conn)
            .unwrap();

        assert!(!is_allowed_provider_class("alpaca", "futures"));
        refresh_allowed(&mut conn).unwrap();
        assert!(is_allowed_provider_class("alpaca", "futures"));
    }
}
