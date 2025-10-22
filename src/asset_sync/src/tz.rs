//! Time zone parsing and conversion helpers.
//!
//! What this module provides:
//! - [`parse_ts_to_utc`]: Parse RFC-3339 timestamps with an explicit offset and convert to UTC.
//! - [`from_local_naive`]: Convert a naive local timestamp with an IANA time zone (e.g.,
//!   "America/New_York") to UTC, erroring on DST gaps (spring-forward) and ambiguous times
//!   (fall-back), matching strict behavior.
//! - [`from_local_naive_tz`]: Same as above but accepts a pre-parsed [`chrono_tz::Tz`].
//! - [`from_local_naive_with_policy`]: Like `from_local_naive_tz` but allows choosing a
//!   policy for handling DST gaps and ambiguities via [`DstPolicy`].
//!
//! Notes:
//! - Ambiguous local times happen during “fall back” when a wall time occurs twice.
//! - Nonexistent local times happen during “spring forward” when a wall time is skipped.
//! - When you need a deterministic mapping for planning or scheduling, prefer
//!   [`DstPolicy::PreferEarliest`] or [`DstPolicy::PreferLatest`], or fall back to
//!   [`DstPolicy::Strict`] and decide at a higher layer.
//! - All database writes are RFC-3339 UTC strings; all bucket math uses UTC. Local times
//!   are only accepted at API/CLI edges and must resolve deterministically or error.
//!
//! Examples
//! - RFC-3339 with offset to UTC:
//!   "2024-03-10T09:30:00-05:00" -> "2024-03-10T14:30:00Z"
//! - New York “fall back” ambiguity (2024-11-03 01:30 occurs twice):
//!   PreferEarliest -> 05:30Z, PreferLatest -> 06:30Z.

use anyhow::Context;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

/// RFC-3339 with offset -> UTC.
///
/// Example:
/// - "2024-03-10T09:30:00-05:00" -> "2024-03-10T14:30:00Z"
pub fn parse_ts_to_utc(s: &str) -> anyhow::Result<DateTime<Utc>> {
    let dt = DateTime::parse_from_rfc3339(s).with_context(|| format!("bad rfc3339: {s}"))?;
    Ok(dt.with_timezone(&Utc))
}

/// Policy for handling DST edge cases when converting local naive timestamps to UTC.
pub enum DstPolicy {
    /// Strict behavior: error on ambiguous (fall-back) or nonexistent (spring-forward) local times.
    Strict,
    /// For ambiguous local times (two possible instants), pick the earliest instant
    /// (typically the DST occurrence).
    PreferEarliest,
    /// For ambiguous local times (two possible instants), pick the latest instant
    /// (typically the standard-time occurrence).
    PreferLatest,
    /// For nonexistent local times (spring-forward gap), shift forward in one-minute
    /// increments until the first valid instant is found (capped at 2 hours).
    ShiftForward,
}

/// Convert a naive local timestamp to UTC using a specific IANA time zone and DST policy.
///
/// Behavior:
/// - If the local time maps to a single instant, that instant is returned.
/// - If the local time is ambiguous (fall-back), behavior depends on `policy`:
///   - PreferEarliest -> pick the earlier instant
///   - PreferLatest -> pick the later instant
///   - Strict/ShiftForward -> return an error
/// - If the local time is nonexistent (spring-forward gap), behavior depends on `policy`:
///   - ShiftForward -> step forward minute-by-minute until a valid instant is found (max 2 hours)
///   - Strict/PreferEarliest/PreferLatest -> return an error
///
/// Errors:
/// - Returns an error if the time is ambiguous or nonexistent and the chosen policy does not resolve it.
pub fn from_local_naive_with_policy(
    naive: NaiveDateTime,
    tz: Tz,
    policy: DstPolicy,
) -> anyhow::Result<DateTime<Utc>> {
    use chrono::offset::LocalResult::*;
    match tz.from_local_datetime(&naive) {
        Single(dt) => Ok(dt.with_timezone(&Utc)),
        Ambiguous(a, b) => match policy {
            DstPolicy::PreferEarliest => Ok(a.with_timezone(&Utc)),
            DstPolicy::PreferLatest => Ok(b.with_timezone(&Utc)),
            _ => Err(anyhow::anyhow!("ambiguous local time")),
        },
        None => match policy {
            DstPolicy::ShiftForward => {
                // minimal nudge forward: try +1 minute until Single
                let mut t = naive;
                for _ in 0..120 {
                    // cap at 2 hours
                    t += chrono::Duration::minutes(1);
                    if let Single(dt) = tz.from_local_datetime(&t) {
                        return Ok(dt.with_timezone(&Utc));
                    }
                }
                Err(anyhow::anyhow!("nonexistent local time"))
            }
            _ => Err(anyhow::anyhow!("nonexistent local time")),
        },
    }
}

/// Convert a naive local timestamp to UTC with a pre-parsed time zone (`Tz`) using strict behavior.
///
/// See [`from_local_naive_with_policy`] for a variant that allows picking a policy.
pub fn from_local_naive_tz(
    naive: NaiveDateTime,
    tz: chrono_tz::Tz,
) -> anyhow::Result<DateTime<Utc>> {
    from_local_naive_with_policy(naive, tz, DstPolicy::Strict)
}

/// Convert a naive local timestamp to UTC by parsing an IANA time zone name (e.g., "America/New_York").
///
/// This uses strict behavior (errors on ambiguous/nonexistent local times). Use
/// [`from_local_naive_with_policy`] if you need custom handling.
///
/// Errors:
/// - Invalid time zone name
/// - Ambiguous or nonexistent local time (strict mode)
pub fn from_local_naive(naive: NaiveDateTime, tz_name: &str) -> anyhow::Result<DateTime<Utc>> {
    let tz: Tz = tz_name
        .parse()
        .with_context(|| format!("bad tz: {tz_name}"))?;
    from_local_naive_tz(naive, tz)
}

/// Format a UTC datetime as an RFC-3339 string with millisecond precision.
pub fn to_rfc3339_millis(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    #[test]
    fn parse_rfc3339_offset_to_utc() {
        // Offset timestamp: 2024-03-10 09:30 at -05:00 -> 14:30Z
        let ts = "2024-03-10T09:30:00-05:00";
        let got = parse_ts_to_utc(ts).expect("parse");
        let want = Utc.with_ymd_and_hms(2024, 3, 10, 14, 30, 0).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn ny_spring_forward_gap_is_error_strict() {
        // America/New_York jumps from 02:00 to 03:00 on 2024-03-10.
        // 02:30 local does not exist and should error under Strict.
        let naive = NaiveDate::from_ymd_opt(2024, 3, 10)
            .unwrap()
            .and_hms_opt(2, 30, 0)
            .unwrap();
        let res = from_local_naive(naive, "America/New_York");
        assert!(res.is_err(), "expected error on spring-forward gap");
    }

    #[test]
    fn ny_spring_forward_gap_shift_forward_to_3am() {
        // 02:30 local does not exist; ShiftForward should land at 03:00 local,
        // which is 07:00Z once DST (-04:00) begins.
        let naive = NaiveDate::from_ymd_opt(2024, 3, 10)
            .unwrap()
            .and_hms_opt(2, 30, 0)
            .unwrap();
        let tz: Tz = "America/New_York".parse().unwrap();
        let got = from_local_naive_with_policy(naive, tz, DstPolicy::ShiftForward).unwrap();
        let want = Utc.with_ymd_and_hms(2024, 3, 10, 7, 0, 0).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn ny_fall_back_ambiguous_is_error_strict() {
        // America/New_York repeats 01:xx on 2024-11-03.
        // 01:30 local is ambiguous and should error under Strict.
        let naive = NaiveDate::from_ymd_opt(2024, 11, 3)
            .unwrap()
            .and_hms_opt(1, 30, 0)
            .unwrap();
        let res = from_local_naive(naive, "America/New_York");
        assert!(res.is_err(), "expected error on fall-back ambiguity");
    }

    #[test]
    fn ny_fall_back_prefer_earliest_and_latest() {
        // 2024-11-03 America/New_York 01:30 occurs twice:
        // - 01:30 EDT (UTC-4)  -> 05:30Z  (earlier instant)
        // - 01:30 EST (UTC-5)  -> 06:30Z  (later instant)
        let naive = NaiveDate::from_ymd_opt(2024, 11, 3)
            .unwrap()
            .and_hms_opt(1, 30, 0)
            .unwrap();
        let tz: Tz = "America/New_York".parse().unwrap();

        let got_earliest =
            from_local_naive_with_policy(naive, tz, DstPolicy::PreferEarliest).unwrap();
        let want_earliest = Utc.with_ymd_and_hms(2024, 11, 3, 5, 30, 0).unwrap();
        assert_eq!(got_earliest, want_earliest);

        let got_latest = from_local_naive_with_policy(naive, tz, DstPolicy::PreferLatest).unwrap();
        let want_latest = Utc.with_ymd_and_hms(2024, 11, 3, 6, 30, 0).unwrap();
        assert_eq!(got_latest, want_latest);
    }

    #[test]
    fn ny_valid_conversion_est() {
        // A normal EST time (winter): 2024-01-15 09:30 local -> 14:30Z
        let naive = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(9, 30, 0)
            .unwrap();
        let got = from_local_naive(naive, "America/New_York").expect("convert");
        let want = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn from_local_naive_tz_matches_from_local_naive() {
        let naive = NaiveDate::from_ymd_opt(2024, 1, 20)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let tz: Tz = "America/New_York".parse().unwrap();

        let a = from_local_naive(naive, "America/New_York").unwrap();
        let b = from_local_naive_tz(naive, tz).unwrap();
        assert_eq!(a, b);
    }
}
