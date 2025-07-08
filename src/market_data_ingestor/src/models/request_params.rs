use chrono::{DateTime, Utc};

use crate::models::{asset::AssetClass, timeframe::TimeFrame};

/// Universal parameters for requesting time-series bar data from any market data provider.
///
/// This struct is designed to be vendor-agnostic and supports multiple asset classes
/// (e.g., stocks, futures, crypto). It is intended as the standard input for all
/// [`DataProvider`](../requests/provider.rs) implementations.
#[derive(Clone, Debug)]
pub struct BarsRequestParams {
    /// List of symbols to request (e.g., `["AAPL"]`, `["ESU24"]`, `["BTC-USD"]`).
    pub symbols: Vec<String>,

    /// The time interval for each bar (e.g., 1 minute, 1 day).
    ///
    /// This uses the [`TimeFrame`](crate::models::timeframe::TimeFrame) struct, which is a universal
    /// representation of bar intervals (amount and unit). **Validation of allowed values is performed
    /// by each data provider implementation, according to their own API rules.**
    ///
    /// For example, some providers may only allow certain combinations (like 1-59 for minutes, or
    /// only 1 for days), while others may be more flexible.
    pub timeframe: TimeFrame,

    /// Start of the requested time range (inclusive, UTC).
    ///
    /// Providers should return bars starting at or after this timestamp.
    pub start: DateTime<Utc>,

    /// End of the requested time range (exclusive, UTC).
    ///
    /// Providers should return bars strictly before this timestamp.
    pub end: DateTime<Utc>,

    /// The asset class for the requested symbols (e.g., `UsEquity`, `Futures`, `Crypto`).
    ///
    /// This helps providers route the request to the correct API or endpoint.
    pub asset_class: AssetClass,
}