//! Canonical in-memory representation of a time-series bar (OHLCV).
//!
//! This struct is used as the standard output for all [`DataProvider`](crate::providers::DataProvider)
//! implementations, regardless of asset class (stocks, futures, crypto, etc.).

use chrono::{DateTime, Utc};

/// A single time-series bar (OHLCV) for a given timestamp.
///
/// This struct is vendor-agnostic and is used throughout the data ingestion pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    /// The timestamp for this bar (UTC).
    pub timestamp: DateTime<Utc>,

    /// Opening price.
    pub open: f64,

    /// Highest price during the bar interval.
    pub high: f64,

    /// Lowest price during the bar interval.
    pub low: f64,

    /// Closing price.
    pub close: f64,

    /// Volume traded during the bar interval.
    pub volume: f64,

    /// Trade count for the bar. Not all providers supply this.
    pub trade_count: Option<u64>,

    /// Volume-weighted average price. Not all providers supply this.
    pub vwap: Option<f64>,
}
