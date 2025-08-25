//! bucket.rs — UTC bucket mapping utilities
//!
//! - One stable epoch: Unix (1970-01-01T00:00:00Z).
//! - Fixed-size frames (minute/hour/day): second-based math.
//! - Week: Monday 00:00:00Z–aligned using a week epoch of 1969-12-29.
//! - Month: linear (year, month) indexing relative to 1970-01.
//!
//! All functions assume the input timestamp is UTC.

use std::num::NonZeroU32;

use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};

/// Unix epoch start (1970-01-01T00:00:00Z).
pub const EPOCH_UNIX: DateTime<Utc> = DateTime::<Utc>::UNIX_EPOCH;

/// Number of seconds in a minute.
pub const SECS_PER_MINUTE: i64 = 60;
/// Number of seconds in an hour.
pub const SECS_PER_HOUR: i64 = 60 * SECS_PER_MINUTE;
/// Number of seconds in a day.
pub const SECS_PER_DAY: i64 = 24 * SECS_PER_HOUR;
/// Number of seconds in a week.
pub const SECS_PER_WEEK: i64 = 7 * SECS_PER_DAY;

/// shift so Monday 1969-12-29 00:00Z becomes index 0
const WEEK_MONDAY_ANCHOR_OFFSET_SECS: i64 = 3 * SECS_PER_DAY; // +3d

/// Timeframe granularity (calendar-aware where needed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeframeUnit {
    /// UTC minute
    Minute,
    /// UTC hour
    Hour,
    /// UTC day
    Day,
    /// Monday-based, UTC
    Week,
    /// calendar months, UTC  
    Month,
}

/// A timeframe = amount × unit (e.g., 5-Minute, 3-Hour, 2-Week, 6-Month).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeframe {
    amount: NonZeroU32,
    unit: TimeframeUnit,
}

impl Timeframe {
    /// Create a new TimeFrame
    pub const fn new(amount: NonZeroU32, unit: TimeframeUnit) -> Self {
        Self { amount, unit }
    }
}

/// Compute the bucket id for a UTC timestamp.
pub fn bucket_id(ts_utc: DateTime<Utc>, tf: Timeframe) -> u64 {
    match tf.unit {
        TimeframeUnit::Minute => id_fixed(ts_utc, SECS_PER_MINUTE * tf.amount.get() as i64),
        TimeframeUnit::Hour => id_fixed(ts_utc, SECS_PER_HOUR * tf.amount.get() as i64),
        TimeframeUnit::Day => id_fixed(ts_utc, SECS_PER_DAY * tf.amount.get() as i64),
        TimeframeUnit::Week => id_week(ts_utc, tf.amount.get()),
        TimeframeUnit::Month => id_month(ts_utc, tf.amount.get()),
    }
}

/// Get the UTC start instant for a bucket id.
pub fn bucket_start_utc(id: u64, tf: Timeframe) -> DateTime<Utc> {
    match tf.unit {
        TimeframeUnit::Minute => start_fixed(id, SECS_PER_MINUTE * tf.amount.get() as i64),
        TimeframeUnit::Hour => start_fixed(id, SECS_PER_HOUR * tf.amount.get() as i64),
        TimeframeUnit::Day => start_fixed(id, SECS_PER_DAY * tf.amount.get() as i64),
        TimeframeUnit::Week => start_week(id, tf.amount.get()),
        TimeframeUnit::Month => start_month(id, tf.amount.get()),
    }
}

/// Exclusive end instant for the bucket (start + width).
pub fn bucket_end_exclusive_utc(id: u64, tf: Timeframe) -> DateTime<Utc> {
    match tf.unit {
        TimeframeUnit::Minute => {
            bucket_start_utc(id, tf) + Duration::seconds(SECS_PER_MINUTE * tf.amount.get() as i64)
        }
        TimeframeUnit::Hour => {
            bucket_start_utc(id, tf) + Duration::seconds(SECS_PER_HOUR * tf.amount.get() as i64)
        }
        TimeframeUnit::Day => {
            bucket_start_utc(id, tf) + Duration::seconds(SECS_PER_DAY * tf.amount.get() as i64)
        }
        TimeframeUnit::Week => {
            bucket_start_utc(id, tf) + Duration::seconds(SECS_PER_WEEK * tf.amount.get() as i64)
        }
        TimeframeUnit::Month =>
        // month width varies: compute start of *next* bucket
        {
            bucket_start_utc(id + 1, tf)
        }
    }
}

// ----- fixed-size internals (minute/hour/day) -----

fn id_fixed(ts_utc: DateTime<Utc>, bucket_secs: i64) -> u64 {
    let secs = ts_utc.signed_duration_since(EPOCH_UNIX).num_seconds();
    (secs.div_euclid(bucket_secs)) as u64
}

fn start_fixed(id: u64, bucket_secs: i64) -> DateTime<Utc> {
    // Use i128 internally to avoid accidental overflow in extreme cases.
    let offset_secs = (id as i128) * (bucket_secs as i128);
    EPOCH_UNIX + Duration::seconds(offset_secs as i64)
}

// ----- week internals (Monday-aligned) -----

fn id_week(ts_utc: DateTime<Utc>, amount: u32) -> u64 {
    let secs = ts_utc.timestamp();
    let width = SECS_PER_WEEK * amount as i64;
    ((secs + WEEK_MONDAY_ANCHOR_OFFSET_SECS).div_euclid(width)) as u64
}

fn start_week(id: u64, amount: u32) -> DateTime<Utc> {
    let width = (SECS_PER_WEEK as i128) * (amount as i128);
    // seconds sine hte Monday anchor
    let since_anchor = (id as i128) * width;
    // convert back to unix seconds, sustract the +3d we added on the way in
    let unix_secs = since_anchor - (WEEK_MONDAY_ANCHOR_OFFSET_SECS as i128);
    Utc.timestamp_opt(unix_secs as i64, 0)
        .single()
        .expect("getting start timestamp from week id")
}

// ----- month internals (calendar-aware) -----

fn id_month(ts_utc: DateTime<Utc>, amount: u32) -> u64 {
    // Linear month index relative to 1970-01 (index 0).
    let y = ts_utc.year() as i64;
    let m = ts_utc.month() as i64; // 1..=12
    let idx = (y - 1970) * 12 + (m - 1);
    (idx.div_euclid(amount as i64)) as u64
}

fn start_month(id: u64, amount: u32) -> DateTime<Utc> {
    let start_idx = (id as i128) * (amount as i128);
    let y = 1970 + start_idx.div_euclid(12);
    let m0 = start_idx.rem_euclid(12); // 0..11
    let month = (m0 + 1) as u32; // 1..12
    Utc.with_ymd_and_hms(y as i32, month, 1, 0, 0, 0)
        .single()
        .expect("getting start timestamp from month id")
}
// -------------------- tests --------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU32;

    const M1: NonZeroU32 = match NonZeroU32::new(1) {
        Some(nz) => nz,
        None => unreachable!(),
    };

    #[test]
    fn minute_roundtrip() {
        let tf = Timeframe::new(M1, TimeframeUnit::Minute);
        let t = Utc.with_ymd_and_hms(2025, 1, 2, 3, 4, 5).unwrap();
        let id = bucket_id(t, tf);
        assert_eq!(bucket_id(bucket_start_utc(id, tf), tf), id);
    }

    #[test]
    fn month_roundtrip_and_boundaries() {
        let tf = Timeframe::new(M1, TimeframeUnit::Month);
        let t = Utc.with_ymd_and_hms(2024, 2, 29, 0, 0, 0).unwrap(); // leap day
        let id = bucket_id(t, tf);
        let start = bucket_start_utc(id, tf);
        assert_eq!(start, Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap());
        // end exclusive = start of next month
        let end = bucket_end_exclusive_utc(id, tf);
        assert_eq!(end, Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap());
    }
}
