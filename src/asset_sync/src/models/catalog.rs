//! Catalog models for provider metadata and symbol mappings.
//!
//! These types mirror lookup/metadata tables defined in the schema and are used
//! to populate and query provider capabilities and symbol translations:
//! - [`crate::schema::provider`] — provider registry (code, name)
//! - [`crate::schema::asset_class`] — asset class registry (codes like "us_equity")
//! - [`crate::schema::provider_asset_class`] — which provider supports which asset class
//! - [`crate::schema::provider_symbol_map`] — canonical-to-remote symbol mapping per provider/class
//!
//! Notes
//! - All structs are Diesel-compatible (Queryable/Insertable/Selectable) for SQLite.
//! - Composite keys are declared where needed (e.g., provider_asset_class).
//!
//! Example (no_run)
//! ```no_run
//! use asset_sync::schema;
//! use asset_sync::models::catalog::*;
//! use diesel::prelude::*;
//!
//! fn seed(conn: &mut SqliteConnection) -> diesel::QueryResult<()> {
//!     // Providers and classes
//!     diesel::insert_into(schema::provider::table)
//!         .values(NewProvider { code: "alpaca", name: "Alpaca Markets" })
//!         .execute(conn)?;
//!     diesel::insert_into(schema::asset_class::table)
//!         .values(NewAssetClass { code: "us_equity" })
//!         .execute(conn)?;
//!
//!     // Capability
//!     diesel::insert_into(schema::provider_asset_class::table)
//!         .values(NewProviderAssetClass { provider_code: "alpaca", asset_class_code: "us_equity" })
//!         .execute(conn)?;
//!
//!     // Symbol mapping
//!     diesel::insert_into(schema::provider_symbol_map::table)
//!         .values(NewProviderSymbolMap {
//!             provider_code: "alpaca",
//!             asset_class_code: "us_equity",
//!             canonical_symbol: "AAPL",
//!             remote_symbol: "AAPL",
//!         })
//!         .execute(conn)?;
//!     Ok(())
//! }
//! ```

use diesel::prelude::*;

// ----------------------- provider -----------------------

/// A provider registry row in [`crate::schema::provider`](crate::schema::provider).
///
/// Identified by a normalized, lowercase `code` (e.g., "alpaca").
#[derive(Debug, Queryable, Identifiable, AsChangeset, Selectable)]
#[diesel(table_name = crate::schema::provider)]
#[diesel(primary_key(code))]
pub struct Provider {
    /// Provider code (primary key), e.g., "alpaca".
    pub code: String,
    /// Human-readable provider name, e.g., "Alpaca Markets".
    pub name: String,
}

/// Insertable form of [`Provider`], used for creating new providers.
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::provider)]
pub struct NewProvider<'a> {
    /// Provider code (primary key), normalized lowercase.
    pub code: &'a str,
    /// Human-readable provider name.
    pub name: &'a str,
}

// ----------------------- asset_class --------------------

/// An asset class row in [`crate::schema::asset_class`](crate::schema::asset_class).
///
/// Example codes: "us_equity", "futures", "crypto".
#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::asset_class)]
#[diesel(primary_key(code))]
pub struct AssetClass {
    /// Asset class code (primary key), e.g., "us_equity".
    pub code: String,
}

/// Insertable form of [`AssetClass`].
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::asset_class)]
pub struct NewAssetClass<'a> {
    /// Asset class code (primary key).
    pub code: &'a str,
}

// ------------------- provider_asset_class ---------------
// Composite PK => must declare both columns.

/// Capability mapping: which provider supports which asset class.
///
/// Backed by [`crate::schema::provider_asset_class`](crate::schema::provider_asset_class).
#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::provider_asset_class)]
#[diesel(primary_key(provider_code, asset_class_code))]
pub struct ProviderAssetClass {
    /// Foreign key to [`Provider::code`](crate::models::catalog::Provider).
    pub provider_code: String,
    /// Foreign key to [`AssetClass::code`](crate::models::catalog::AssetClass).
    pub asset_class_code: String,
}

/// Insertable form of [`ProviderAssetClass`].
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::provider_asset_class)]
pub struct NewProviderAssetClass<'a> {
    /// Foreign key to [`Provider::code`](crate::models::catalog::Provider).
    pub provider_code: &'a str,
    /// Foreign key to [`AssetClass::code`](crate::models::catalog::AssetClass).
    pub asset_class_code: &'a str,
}

// ------------------- provider_symbol_map ----------------
// This table has UNIQUE constraints but no PK column => do NOT derive Identifiable.
// Provide two structs: owned (Queryable/Selectable) and borrowed (Insertable).

/// Canonical-to-remote symbol mapping for a provider and asset class.
///
/// Backed by [`crate::schema::provider_symbol_map`](crate::schema::provider_symbol_map).
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::provider_symbol_map)]
pub struct ProviderSymbolMapRow {
    /// Foreign key to [`Provider::code`](crate::models::catalog::Provider).
    pub provider_code: String,
    /// Foreign key to [`AssetClass::code`](crate::models::catalog::AssetClass).
    pub asset_class_code: String,
    /// Canonical (internal) symbol (e.g., "AAPL", "ES").
    pub canonical_symbol: String,
    /// Provider-specific remote symbol (e.g., "AAPL", "ESZ5").
    pub remote_symbol: String,
}

/// Insertable/changeset form for creating or upserting symbol mappings.
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::provider_symbol_map)]
pub struct NewProviderSymbolMap<'a> {
    /// Foreign key to [`Provider::code`](crate::models::catalog::Provider).
    pub provider_code: &'a str,
    /// Foreign key to [`AssetClass::code`](crate::models::catalog::AssetClass).
    pub asset_class_code: &'a str,
    /// Canonical (internal) symbol.
    pub canonical_symbol: &'a str,
    /// Provider-specific remote symbol.
    pub remote_symbol: &'a str,
}

/// Changeset for updating only the remote symbol of an existing mapping.
///
/// Use alongside a WHERE clause that identifies the row by keys.
#[derive(Debug, AsChangeset)]
#[diesel(table_name = crate::schema::provider_symbol_map)]
pub struct ProviderSymbolMapUpdate<'a> {
    /// Replacement value for the `remote_symbol` column.
    pub remote_symbol: &'a str,
}
