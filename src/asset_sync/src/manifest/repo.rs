use anyhow::{Context, Ok};
use chrono::{DateTime, Utc};
use diesel::{associations::HasTable, prelude::*};
use roaring::RoaringBitmap;

use crate::{
    bucket::{self, bucket_end_exclusive_utc, bucket_id, bucket_start_utc},
    manifest::{ManifestRepo, RepoError, RepoResult},
    roaring_bytes,
    schema::asset_manifest,
    spec::{ProviderId, Range},
    timeframe::{Timeframe, db as tf_db},
    tz,
};

use crate::schema::asset_manifest::dsl as am;

#[derive(Insertable, AsChangeset, Debug)]
#[diesel(table_name = asset_manifest)]
struct ManifestRow<'a> {
    symbol: &'a str,
    provider_code: &'a str,
    asset_class_code: &'a str,
    timeframe_amount: i32,
    timeframe_unit: &'a str,
    desired_start: &'a str,       // RFC3339 UTC
    desired_end: Option<&'a str>, // RFC3339 UTC
    watermark: Option<&'a str>,   // RFC3339 UTC
    last_error: Option<&'a str>,
}

// ---- helpers: map the enums to catalog codes / strings ----
fn provider_code_map(p: ProviderId) -> &'static str {
    match p {
        ProviderId::Alpaca => "alpaca",
    }
}

fn asset_class_code_map(ac: market_data_ingestor::models::asset::AssetClass) -> &'static str {
    use market_data_ingestor::models::asset::AssetClass::*;
    match ac {
        UsEquity => "us_equity",
        Futures => "futures",
    }
}

fn timeframe_parts(tf: &market_data_ingestor::models::timeframe::TimeFrame) -> (i32, &'static str) {
    use market_data_ingestor::models::timeframe::TimeFrameUnit::*;

    let amount = tf.amount as i32;
    let unit = match tf.unit {
        Minute => "Minute",
        Hour => "Hour",
        Day => "Day",
        Week => "Week",
        Month => "Month",
    };
    (amount, unit)
}

fn desired_start_end(r: &Range) -> (DateTime<Utc>, Option<DateTime<Utc>>) {
    match *r {
        Range::Open { start } => (start, None),
        Range::Closed { start, end } => (start, Some(end)),
    }
}

/// Repository for managing asset manifest data in a SQLite database.
pub struct SqliteRepo;

impl SqliteRepo {
    pub fn new() -> Self {
        Self
    }
}

#[inline]
fn end_bucket_exclusive(window_end: DateTime<Utc>, tf: Timeframe) -> u64 {
    let end_id = bucket::bucket_id(window_end, tf);
    let end_id_start = bucket::bucket_start_utc(end_id, tf);
    if end_id_start < window_end {
        end_id + 1
    } else {
        end_id
    }
}

impl ManifestRepo for SqliteRepo {
    fn upsert_manifest(
        &self,
        conn: &mut diesel::SqliteConnection,
        spec: &crate::spec::AssetSpec,
    ) -> RepoResult<i64> {
        use crate::schema::asset_manifest::dsl::*;

        // Map the spec to row fields
        let (tf_amount, tf_unit) = timeframe_parts(&spec.timeframe);
        let (start_dt, end_dt_opt) = desired_start_end(&spec.range);

        let desired_start_rfc3339 = tz::to_rfc3339_millis(start_dt);
        let desired_end_rfc3339 = end_dt_opt.map(tz::to_rfc3339_millis);

        let row = ManifestRow {
            symbol: &spec.symbol,
            provider_code: provider_code_map(spec.provider),
            asset_class_code: asset_class_code_map(spec.asset_class.clone()),
            timeframe_amount: tf_amount,
            timeframe_unit: tf_unit,
            desired_start: &desired_start_rfc3339,
            desired_end: desired_end_rfc3339.as_deref(),
            watermark: None,
            last_error: None,
        };

        // Insert .. ON CONFLICT (..) DO UPDATE .. RETURNING id (Sqlite 3.35+)
        let manifest_id_i32: i32 = diesel::insert_into(asset_manifest::table())
            .values(&row)
            .on_conflict((
                symbol,
                provider_code,
                asset_class_code,
                timeframe_amount,
                timeframe_unit,
            ))
            .do_update()
            .set(&row)
            .returning(id)
            .get_result(conn)?;

        let manifest_id = manifest_id_i32 as i64;

        // Ensure coverage row exists (idempotent)
        let bytes = roaring_bytes::rb_to_bytes(&RoaringBitmap::new());

        use crate::schema::asset_coverage_bitmap as acb;
        let _ = diesel::insert_into(acb::table)
            .values((
                acb::manifest_id.eq(manifest_id as i32),
                acb::bitmap.eq(bytes),
                acb::version.eq(0),
            ))
            .on_conflict(acb::manifest_id)
            .do_nothing()
            .execute(conn)?;

        Ok(manifest_id)
    }

    fn coverage_get(
        &self,
        conn: &mut diesel::SqliteConnection,
        manifest_id_v: i64,
    ) -> RepoResult<(RoaringBitmap, i32)> {
        use crate::schema::asset_coverage_bitmap::dsl::*;

        if let Some((b, v)) = asset_coverage_bitmap
            .filter(manifest_id.eq(manifest_id_v as i32))
            .select((bitmap, version))
            .first::<(Vec<u8>, i32)>(conn)
            .optional()?
        {
            Ok((roaring_bytes::rb_from_bytes(&b), v))
        } else {
            Ok((RoaringBitmap::new(), 0))
        }
    }

    fn coverage_put(
        &self,
        conn: &mut diesel::SqliteConnection,
        manifest_id_v: i64,
        rb: &RoaringBitmap,
        expected_version: i32,
    ) -> RepoResult<i32> {
        use crate::schema::asset_coverage_bitmap::dsl::*;

        let bytes = roaring_bytes::rb_to_bytes(rb);
        let mid_i32 = manifest_id_v as i32;
        let new_version = expected_version + 1;

        let got = diesel::update(
            asset_coverage_bitmap.filter(manifest_id.eq(mid_i32).and(version.eq(expected_version))),
        )
        .set((bitmap.eq(bytes), version.eq(new_version)))
        .returning(version)
        .get_result(conn)
        .optional()?;

        match got {
            Some(v) => Ok(v),
            None => Err(RepoError::CoverageConflict {
                expected: expected_version,
            }
            .into()),
        }
    }

    fn compute_missing(
        &self,
        conn: &mut diesel::SqliteConnection,
        manifest_id_v: i64,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    ) -> RepoResult<Vec<(DateTime<Utc>, DateTime<Utc>)>> {
        if window_end <= window_start {
            return Ok(vec![]);
        }

        // 1) Load timeframe for this manifest from DB
        let (amt, unit_str): (i32, String) = am::asset_manifest
            .find(manifest_id_v as i32)
            .select((am::timeframe_amount, am::timeframe_unit))
            .first(conn)
            .with_context(|| format!("manifest {manifest_id_v} not found"))?;

        let tf: Timeframe = tf_db::from_db_row(amt, &unit_str)?;

        // 2) Translate window to bucket IDs (exclusive end)
        let start_id_u64 = bucket_id(window_start, tf);
        let end_id_u64 = bucket_id(window_end, tf);
        if end_id_u64 <= start_id_u64 {
            return Ok(vec![]);
        }

        // Coverage bitmap + version
        let (present, _ver) = self.coverage_get(conn, manifest_id_v)?;

        // 3) Build window bitmap efficiently
        let mut window = RoaringBitmap::new();
        // Roaring is u32; our bucket IDs must fit. Unix epoch + minute/hour/day/week/month do
        let start_id = u32::try_from(start_id_u64).context("bucket id overflow (start)")?;
        let end_id = u32::try_from(end_id_u64).context("bucket id overflow (end)")?;
        window.insert_range(start_id..end_id); // fill contiguous window quicky

        // 4) missing = window - present (set difference) -- fast, container-wise.
        let missing = &window - &present; // uses `Sub` impl for RoaringBitmap

        // 5) Coalesce the missing bucket IDs into contiguous runs and map back to UTC
        Ok(coalesce_runs_to_utc_ranges(&missing, tf))
    }

    fn gaps_complete(&self, conn: &mut diesel::SqliteConnection, gap_id: i64) -> RepoResult<()> {}

    fn gaps_lease(
        &self,
        conn: &mut diesel::SqliteConnection,
        worker: &str,
        limit: i64,
        ttl: chrono::Duration,
    ) -> RepoResult<Vec<i64>> {
    }

    fn gaps_upsert(
        &self,
        conn: &mut diesel::SqliteConnection,
        manifest_id: i64,
        ranges: &[(DateTime<Utc>, DateTime<Utc>)],
    ) -> RepoResult<()> {
    }
}

fn coalesce_runs_to_utc_ranges(
    rb: &RoaringBitmap,
    tf: Timeframe,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let mut out = Vec::new();
    let mut it = rb.iter();
    if let Some(mut run_start) = it.next() {
        let mut prev = run_start;
        for x in it {
            if x == prev + 1 {
                prev = x;
                continue;
            }
            // close [run_start, prev] -> [start_utc, end_utc_exclusive]
            out.push((
                bucket_start_utc(run_start as u64, tf),
                bucket_end_exclusive_utc((prev as u64) + 1, tf),
            ));
            run_start = x;
            prev = x;
        }
        out.push((
            bucket_start_utc(run_start as u64, tf),
            bucket_end_exclusive_utc((prev as u64) + 1, tf),
        ));
    }
    out
}
