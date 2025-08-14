//! Declarative specification of *what data to keep fresh*.
//!
use chrono::{DateTime, Utc};
use market_data_ingestor::models::{asset::AssetClass, timeframe::{TimeFrame, TimeFrameUnit}};
use serde::{Deserialize, Serialize};


/// Which upstream to use (serde snake_case).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    /// Alpaca trading API provider.
    Alpaca,
}

/// Open/closed time range for desired data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Range {
    /// Inclusive start, open end (keep fresh).
    Open {
        /// Inclusive start timestamp (UTC).
        start: DateTime<Utc>,
    },

    /// Inclusive start..end (backfill only).
    Closed {
        /// Inclusive start timestamp (UTC).
        start: DateTime<Utc>,
        /// Inclusive end timestamp (UTC).
        end: DateTime<Utc>,
    }
}

impl Range {
    /// Returns the inclusive start timestamp (UTC) of the range, regardless of whether it is open or closed.
    pub fn start(&self) -> DateTime<Utc> {
        match *self {
            Range::Open { start } | Range::Closed { start, .. } => start,
        }
    }

    /// Returns the inclusive end timestamp (UTC) if the range is closed; None if the range is open.
    pub fn end(&self) -> Option<DateTime<Utc>> {
        match *self {
            Range::Open { .. } => None,
            Range::Closed { end, .. } => Some(end),
        }
    }
}

/// Declarative spec for one symbol/timeframe on one provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetSpec {
    /// e.g., "AAPL", "ESU25"
    pub symbol: String,

    /// Upstream market data/trading provider identifier.
    pub provider: ProviderId,

    /// Asset class (e.g., US equity, futures, crypto).
    pub asset_class: AssetClass,

    /// Bar timeframe to keep current.
    pub timeframe: TimeFrame,

    /// Time range to backfill (closed) or keep fresh (open).
    pub range: Range,

}

impl Default for AssetSpec {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            provider: ProviderId::Alpaca,
            asset_class: AssetClass::UsEquity,
            timeframe: TimeFrame::new(1, TimeFrameUnit::Minute),
            range: Range::Open {
                // epoch; you can choose a more sensible default
                start: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
            },
        }
    }
}