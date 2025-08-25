//! Diesel models mapping to the database schema.
//!
//! These types mirror the tables defined in the embedded migrations and in
//! [`crate::schema`] for use with Diesel’s Queryable/Insertable APIs:
//! - [`crate::schema::asset_manifest`] — desired coverage, progress, and bookkeeping
//! - [`crate::schema::asset_coverage_bitmap`] — roaring bitmap storing covered bars
//! - [`crate::schema::asset_gaps`] — durable backlog of requested backfills
//!
//! See migrations for constraints and triggers (e.g., `updated_at` trigger on
//! `asset_manifest` and `ON DELETE CASCADE` FKs).

use diesel::prelude::*;
use crate::schema::*;

/// A row in [`crate::schema::asset_manifest`]: one tracked symbol/timeframe on a provider.
///
/// Used for SELECT/UPDATE operations (Queryable, Identifiable, AsChangeset, Selectable).
#[derive(Debug, Clone, Queryable, Identifiable, AsChangeset, Selectable)]
#[diesel(table_name = asset_manifest, check_for_backend(diesel::sqlite::Sqlite))]
pub struct AssetManifest {
    /// Database primary key (SQLite INTEGER PRIMARY KEY rowid). Populated by the DB.
    pub id: i32,
    /// Symbol identifier (e.g., "AAPL", "ESU25").
    pub symbol: String,
    /// Provider identifier (e.g., "alpaca").
    pub provider: String,
    /// Asset class (e.g., "us_equity", "futures").
    pub asset_class: String,
    /// Timeframe amount component (e.g., 1 in "1 Minute").
    pub timeframe_amount: i32,
    /// Timeframe unit component; constrained to "Minute" or "Day".
    pub timeframe_unit: String,
    /// Inclusive desired coverage start in RFC3339 UTC (e.g., "2010-01-01T00:00:00Z").
    pub desired_start: String,
    /// Optional inclusive desired coverage end in RFC3339 UTC; NULL means open-ended.
    pub desired_end: Option<String>,
    /// Optional contiguous progress watermark in RFC3339 UTC.
    pub watermark: Option<String>,
    /// Optional last sync error message.
    pub last_error: Option<String>,
    /// Row creation timestamp in RFC3339 UTC.
    pub created_at: String,
    /// Row update timestamp in RFC3339 UTC (maintained by trigger on UPDATE).
    pub updated_at: String,
}

/// Insertable form of [`AssetManifest`] for creating new rows.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = asset_manifest)]
pub struct NewAssetManifest<'a> {
    /// Symbol identifier (e.g., "AAPL", "ESU25").
    pub symbol: &'a str,
    /// Provider identifier (e.g., "alpaca").
    pub provider: &'a str,
    /// Asset class (e.g., "us_equity", "futures").
    pub asset_class: &'a str,
    /// Timeframe amount component (e.g., 1 in "1 Minute").
    pub timeframe_amount: i32,
    /// Timeframe unit component; must be "Minute" or "Day".
    pub timeframe_unit: &'a str,
    /// Inclusive desired coverage start in RFC3339 UTC.
    pub desired_start: &'a str,
    /// Optional inclusive desired coverage end in RFC3339 UTC; None for open-ended.
    pub desired_end: Option<&'a str>,
}

/// A row in [`crate::schema::asset_coverage_bitmap`]: roaring bitmap of acquired bars.
///
/// Unique per manifest; cleaned up via FK `ON DELETE CASCADE`.
#[derive(Debug, Clone, Queryable, Identifiable, Associations, AsChangeset, Selectable)]
#[diesel(table_name = asset_coverage_bitmap, check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(AssetManifest, foreign_key = manifest_id))]
pub struct CoverageBlob {
    /// Database primary key.
    pub id: i32,
    /// FK to [`AssetManifest::id`].
    pub manifest_id: i32,
    /// Roaring bitmap serialized bytes representing coverage.
    pub bitmap: Vec<u8>,
    /// Optimistic concurrency counter (application-managed).
    pub version: i32,
}

/// Insertable form of [`CoverageBlob`].
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = asset_coverage_bitmap)]
pub struct NewCoverageBlob<'a> {
    /// FK to [`AssetManifest::id`].
    pub manifest_id: i32,
    /// Roaring bitmap serialized bytes representing coverage.
    pub bitmap: &'a [u8],
}

/// A row in [`crate::schema::asset_gaps`]: durable backlog item for backfill work.
///
/// Constrained `state` values: "queued" | "leased" | "done" | "failed".
#[derive(Debug, Clone, Queryable, Identifiable, Associations, AsChangeset, Selectable)]
#[diesel(table_name = asset_gaps, check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(AssetManifest, foreign_key = manifest_id))]
pub struct AssetGap {
    /// Database primary key.
    pub id: i32,
    /// FK to [`AssetManifest::id`].
    pub manifest_id: i32,
    /// Inclusive start timestamp (RFC3339 UTC).
    pub start_ts: String,
    /// Inclusive end timestamp (RFC3339 UTC).
    pub end_ts: String,
    /// Work item state: "queued" | "leased" | "done" | "failed".
    pub state: String,
    /// Optional lease owner identifier (e.g., worker ID).
    pub lease_owner: Option<String>,
    /// Optional lease expiration timestamp (RFC3339 UTC).
    pub lease_expires_at: Option<String>,
}

/// Insertable form of [`AssetGap`].
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = asset_gaps)]
pub struct NewAssetGap<'a> {
    /// FK to [`AssetManifest::id`].
    pub manifest_id: i32,
    /// Inclusive start timestamp (RFC3339 UTC).
    pub start_ts: &'a str,
    /// Inclusive end timestamp (RFC3339 UTC).
    pub end_ts: &'a str,
    /// Initial work item state (typically "queued").
    pub state: &'a str,
}