//! Time zone parsing and conversion helpers.
//!
//! - [`parse_ts_to_utc`] parses an RFC-3339 timestamp with offset and returns UTC.
//! - [`from_local_naive`] converts a naive local timestamp with an IANA time zone (e.g.,
//!   "America/New_York") to UTC, and returns an error for DST gaps (spring-forward) and
//!   ambiguous times (fall-back), aligning with `chrono_tz` behavior.

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

/// Naive local timestamp + IANA tz -> UTC.
///
/// Errors for nonexistent wall times during spring-forward and for ambiguous wall times
/// during fall-back. This lets callers decide policy for these edge cases.
pub fn from_local_naive(naive: NaiveDateTime, tz_name: &str) -> anyhow::Result<DateTime<Utc>> {
    let tz: Tz = tz_name
        .parse()
        .with_context(|| format!("bad tz: {tz_name}"))?;
    let local = tz
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| anyhow::anyhow!("ambiguous or nonexistent local time"))?;
    Ok(local.with_timezone(&Utc))
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
    fn ny_spring_forward_gap_is_error() {
        // America/New_York jumps from 02:00 to 03:00 on 2024-03-10.
        // 02:30 local does not exist and should error.
        let naive = NaiveDate::from_ymd_opt(2024, 3, 10)
            .unwrap()
            .and_hms_opt(2, 30, 0)
            .unwrap();
        let res = from_local_naive(naive, "America/New_York");
        assert!(res.is_err(), "expected error on spring-forward gap");
    }

    #[test]
    fn ny_fall_back_ambiguous_is_error() {
        // America/New_York repeats 01:xx on 2024-11-03.
        // 01:30 local is ambiguous and should error.
        let naive = NaiveDate::from_ymd_opt(2024, 11, 3)
            .unwrap()
            .and_hms_opt(1, 30, 0)
            .unwrap();
        let res = from_local_naive(naive, "America/New_York");
        assert!(res.is_err(), "expected error on fall-back ambiguity");
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
}
