//! Insertable/Queryable helper structs used by the manifest repository implementation.

use diesel::prelude::*;

use crate::schema::{asset_gaps, asset_manifest};

#[derive(Insertable, AsChangeset, Debug)]
#[diesel(table_name = asset_manifest)]
pub(crate) struct ManifestRow<'a> {
    pub(crate) symbol: &'a str,
    pub(crate) provider_code: &'a str,
    pub(crate) asset_class_code: &'a str,
    pub(crate) timeframe_amount: i32,
    pub(crate) timeframe_unit: &'a str,
    pub(crate) desired_start: &'a str,       // RFC3339 UTC
    pub(crate) desired_end: Option<&'a str>, // RFC3339 UTC
    pub(crate) watermark: Option<&'a str>,   // RFC3339 UTC
    pub(crate) last_error: Option<&'a str>,
}

#[derive(Insertable)]
#[diesel(table_name = asset_gaps)]
pub(crate) struct NewGap {
    pub(crate) manifest_id: i32,
    pub(crate) start_ts: String,
    pub(crate) end_ts: String,
    pub(crate) state: String,
}
