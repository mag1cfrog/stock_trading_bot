//! Manifest + coverage + gaps repository (SQLite).
use chrono::{DateTime, Utc};
use roaring::RoaringBitmap;

#[derive(thiserror::Error, Debug)]
/// Errors that can occur while interacting with the manifest repository.
pub enum RepoError {
    #[error("coverage version conflict (expected {expected})")]
    /// Raised when the coverage version does not match the expected value.
    CoverageConflict {
        /// The expected coverage version.
        expected: i32,
    },
}

/// Result type used throughout the manifest repository for fallible operations.
pub type RepoResult<T> = anyhow::Result<T>;

/// Portable surface, SQLite implementation lives in `repo.rs`.
pub trait ManifestRepo {
    /// Inserts or updates a manifest record and returns its identifier.
    fn upsert_manifest(
        &self,
        conn: &mut diesel::SqliteConnection,
        spec: &crate::spec::AssetSpec,
    ) -> RepoResult<i64>;

    /// Retrieves the coverage bitmap and its version for the specified manifest record.
    fn coverage_get(
        &self,
        conn: &mut diesel::SqliteConnection,
        manifest_id: i64,
    ) -> RepoResult<(RoaringBitmap, i32)>;

    /// Bumps version if `expected_version` matches. Returns new version on success.
    fn coverage_put(
        &self,
        conn: &mut diesel::SqliteConnection,
        manifest_id: i64,
        rb: &RoaringBitmap,
        expected_version: i32,
    ) -> RepoResult<i32>;

    /// Computes the manifest time ranges that lack coverage within the provided window.
    fn compute_missing(
        &self,
        conn: &mut diesel::SqliteConnection,
        manifest_id: i64,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    ) -> RepoResult<Vec<(DateTime<Utc>, DateTime<Utc>)>>;

    /// Inserts or updates gap records for the specified manifest with the provided time ranges.
    fn gaps_upsert(
        &self,
        conn: &mut diesel::SqliteConnection,
        manifest_id: i64,
        ranges: &[(DateTime<Utc>, DateTime<Utc>)],
    ) -> RepoResult<()>;

    /// Leases up to `limit` gaps, returning their IDs.
    fn gaps_lease(
        &self,
        conn: &mut diesel::SqliteConnection,
        worker: &str,
        limit: i64,
        ttl: chrono::Duration,
    ) -> RepoResult<Vec<i64>>;

    /// Marks the specified gap as completed for the given manifest.
    fn gaps_complete(&self, conn: &mut diesel::SqliteConnection, gap_id: i64) -> RepoResult<()>;
}
