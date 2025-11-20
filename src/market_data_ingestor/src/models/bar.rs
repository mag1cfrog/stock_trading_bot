//! Canonical in-memory representation of a time-series bar (OHLCV).
//!
//! This struct is used as the standard output for all [`DataProvider`](crate::providers::DataProvider)
//! implementations, regardless of asset class (stocks, futures, crypto, etc.).

use chrono::{DateTime, Utc};

use crate::models::timeframe::TimeFrame;

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

/// Represents a complete set of time-series data for a single symbol.
///
/// This struct groups a vector of [`Bar`]s with their corresponding symbol
/// and [`TimeFrame`], making the data set self-describing.
#[derive(Debug, Clone, PartialEq)]
pub struct BarSeries {
    /// The symbol this data represents (e.g., "AAPL", "ESU24").
    pub symbol: String,
    /// The time interval for each bar in the series.
    pub timeframe: TimeFrame,
    /// The collection of OHLCV bars.
    pub bars: Vec<Bar>,
}
