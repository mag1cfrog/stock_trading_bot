//! A collection of time-series bars for a specific symbol and timeframe.

use crate::models::{bar::Bar, timeframe::TimeFrame};

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
