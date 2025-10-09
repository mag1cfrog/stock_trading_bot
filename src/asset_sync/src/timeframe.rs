//! Timeframe utilities for expressing uniform bar intervals.
//!
//! A [`Timeframe`] pairs a non-zero amount with a [`TimeframeUnit`], covering
//! minute, hour, day, week (Monday-based), and month buckets in UTC. These types
//! give a typed alternative to ad-hoc `(i32, &str)` tuples when scheduling jobs,
//! generating bar requests, or seeding catalog metadata.
//!
//! Typical usage:
//! ```
//! use std::num::NonZeroU32;
//! use asset_sync::timeframe::{Timeframe, TimeframeUnit};
//!
//! let tf = Timeframe::new(NonZeroU32::new(5).unwrap(), TimeframeUnit::Minute);
//! assert_eq!(tf.amount().get(), 5);
//! assert_eq!(tf.unit(), TimeframeUnit::Minute);
//! ```

use std::{fmt, num::NonZeroU32, str::FromStr};

use anyhow::{anyhow, bail};

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
    pub amount: NonZeroU32,
    pub unit: TimeframeUnit,
}

impl Timeframe {
    /// Create a new TimeFrame
    pub const fn new(amount: NonZeroU32, unit: TimeframeUnit) -> Self {
        Self { amount, unit }
    }
    pub const fn amount(&self) -> NonZeroU32 {
        self.amount
    }
    pub const fn unit(&self) -> TimeframeUnit {
        self.unit
    }
}

/// DB round-trip helpers for your schema (amount: INTEGER, unit: TEXT).
pub mod db {
    use anyhow::bail;

    use super::*;

    pub fn to_db_strings(tf: Timeframe) -> (i32, &'static str) {
        let amt = tf.amount().get() as i32;
        let unit = match tf.unit {
            TimeframeUnit::Minute => "Minute",
            TimeframeUnit::Hour => "Hour",
            TimeframeUnit::Day => "Day",
            TimeframeUnit::Week => "Week",
            TimeframeUnit::Month => "Month",
        };
        (amt, unit)
    }

    pub fn from_db_row(amount_i32: i32, unit_str: &str) -> anyhow::Result<Timeframe> {
        if amount_i32 <= 0 {
            bail!("timeframe_amount must be > 0");
        }
        let amount = NonZeroU32::new(amount_i32 as u32).unwrap();
        let unit = match unit_str {
            "Minute" => TimeframeUnit::Minute,
            "Hour" => TimeframeUnit::Hour,
            "Day" => TimeframeUnit::Day,
            "Week" => TimeframeUnit::Week,
            "Month" => TimeframeUnit::Month,
            _ => bail!("unknown timeframe_unit: {unit_str}"),
        };
        Ok(Timeframe::new(amount, unit))
    }
}

/// Display/parse for CLI ergonomics (`"5m"`, `"1D"`, `"6M"`)
impl fmt::Display for Timeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let a = self.amount.get();
        let u = match self.unit {
            TimeframeUnit::Minute => "m",
            TimeframeUnit::Hour => "h",
            TimeframeUnit::Day => "D",
            TimeframeUnit::Week => "W",
            TimeframeUnit::Month => "M",
        };
        write!(f, "{a}{u}")
    }
}
impl FromStr for Timeframe {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // very small parser: 5m / 3h / 1D / 1W / 6M
        if s.is_empty() {
            bail!("empty timeframe");
        }
        let (digits, unit) = s.split_at(s.len() - 1);
        let amount_num: u32 = digits.parse()?;
        let amount = NonZeroU32::new(amount_num).ok_or_else(|| anyhow!("amount must be > 0"))?;
        let unit = match unit {
            "m" => TimeframeUnit::Minute,
            "h" => TimeframeUnit::Hour,
            "D" => TimeframeUnit::Day,
            "W" => TimeframeUnit::Week,
            "M" => TimeframeUnit::Month,
            _ => bail!("unknown unit: {unit}"),
        };
        Ok(Timeframe::new(amount, unit))
    }
}
