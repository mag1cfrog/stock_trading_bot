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

/// Loader + validation helpers.
pub mod load {
    use super::*;
    use std::{fs, path::Path};
    use thiserror::Error;

    #[derive(Debug, Error)]
    /// Errors that can occur while loading or validating an AssetSpec.
    pub enum SpecError {
        /// Underlying filesystem I/O error when reading a spec file.
        #[error("I/O: {0}")]
        Io(#[from] std::io::Error),
        /// Error while parsing TOML into an AssetSpec.
        #[error("TOML parse: {0}")]
        Toml(#[from] toml::de::Error),
        /// Range validation failed: start must be strictly before end.
        #[error("invalid range: start must be < end")]
        BadRange,
        /// Validation failed: symbol was empty or whitespace.
        #[error("symbol must be non-empty")]
        EmptySymbol,
    }

    /// Parse a single spec file (.toml).
    pub fn from_file(path: &Path) -> Result<AssetSpec, SpecError> {
        let s = fs::read_to_string(path)?;
        let spec: AssetSpec = toml::from_str(&s)?;
        validate(&spec)?;
        Ok(spec)
    }

    /// Validate the provided AssetSpec (non-empty symbol; for closed ranges, start < end).
    pub fn validate(spec: &AssetSpec) -> Result<(), SpecError> {
        if spec.symbol.trim().is_empty() {
            return Err(SpecError::EmptySymbol);
        }

        if let Range::Closed { start, end } = spec.range {
            if start >= end {
                return Err(SpecError::BadRange);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::load::{from_file, validate, SpecError};
    use chrono::{TimeZone, Utc};
    use toml::Value;
    use std::{fs, path::PathBuf};

    fn tmp_file(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        // Keep filename deterministic for debugging; include pid to reduce collision risk.
        p.push(format!("asset_spec_test_{}_{}.toml", name, std::process::id()));
        p
    }

    #[test]
    fn test_range_start_end_open() {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let r = Range::Open { start: ts };
        assert_eq!(r.start(), ts);
        assert_eq!(r.end(), None);
    }

    #[test]
    fn test_range_start_end_closed() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 9, 30, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 10, 16, 0, 0).unwrap();
        let r = Range::Closed { start, end };
        assert_eq!(r.start(), start);
        assert_eq!(r.end(), Some(end));
    }

    #[test]
    fn test_asset_spec_default() {
        let d = AssetSpec::default();
        assert!(d.symbol.is_empty());
        assert!(matches!(d.provider, ProviderId::Alpaca));
        assert!(matches!(d.asset_class, AssetClass::UsEquity));
        assert!(matches!(d.timeframe.amount, 1));
        assert!(matches!(d.range, Range::Open { .. }));
    }

    #[test]
    fn test_validate_ok_closed_range() {
        let spec = AssetSpec {
            symbol: "AAPL".into(),
            provider: ProviderId::Alpaca,
            asset_class: AssetClass::UsEquity,
            timeframe: TimeFrame::new(5, TimeFrameUnit::Minute),
            range: Range::Closed {
                start: Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2024, 2, 2, 0, 0, 0).unwrap(),
            },
        };
        assert!(validate(&spec).is_ok());
    }

    #[test]
    fn test_validate_error_empty_symbol() {
        let spec = AssetSpec {
            symbol: "   ".into(),
            ..AssetSpec::default()
        };
        let err = validate(&spec).unwrap_err();
        assert!(matches!(err, SpecError::EmptySymbol));
    }

    #[test]
    fn test_validate_error_bad_range() {
        let t = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let spec = AssetSpec {
            symbol: "MSFT".into(),
            range: Range::Closed { start: t, end: t }, // equal -> bad
            ..AssetSpec::default()
        };
        let err = validate(&spec).unwrap_err();
        assert!(matches!(err, SpecError::BadRange));
    }

    #[test]
    fn test_from_file_success_round_trip() {
        let spec = AssetSpec {
            symbol: "GOOGL".into(),
            provider: ProviderId::Alpaca,
            asset_class: AssetClass::UsEquity,
            timeframe: TimeFrame::new(15, TimeFrameUnit::Minute),
            range: Range::Open {
                start: Utc.with_ymd_and_hms(2023, 12, 31, 0, 0, 0).unwrap(),
            },
        };
        let toml_str = toml::to_string(&spec).expect("serialize");
        let path = tmp_file("ok");
        fs::write(&path, toml_str).unwrap();

        let loaded = from_file(&path).expect("from_file");
        assert_eq!(loaded, spec);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_from_file_invalid_symbol() {
        // Start from a valid spec TOML then blank out symbol.
        let spec = AssetSpec {
            symbol: "AAPL".into(),
            ..AssetSpec::default()
        };
        let mut toml_str = toml::to_string(&spec).unwrap();
        toml_str = toml_str.replace("symbol = \"AAPL\"", "symbol = \"\"");

        let path = tmp_file("bad_symbol");
        fs::write(&path, toml_str).unwrap();
        let err = from_file(&path).unwrap_err();
        assert!(matches!(err, SpecError::EmptySymbol));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_from_file_invalid_closed_range() {
            let t = r#"
symbol = "AAPL"
provider = "alpaca"
asset_class = "UsEquity"

[timeframe]
amount = 1
unit   = "Minute"

[range.closed]
start = "2024-01-01T00:00:00Z"
end   = "2024-01-01T00:00:00Z"
"#;
    let path = tmp_file("bad_range_literal");
    std::fs::write(&path, t).unwrap();

    let err = from_file(&path).unwrap_err();
    assert!(matches!(err, SpecError::BadRange));

    let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_serialization_snake_case_fields() {
        let spec = AssetSpec {
            symbol: "AAPL".into(),
            ..AssetSpec::default()
        };
        let toml_str = toml::to_string(&spec).unwrap();
        
        let v: Value = toml::from_str(&toml_str).unwrap();
        assert_eq!(v.get("provider").and_then(Value::as_str), Some("alpaca"));

        // Externally tagged enum: range table containing an 'open' table.
        let range_tbl = v.get("range").and_then(Value::as_table).expect("range table");
        assert!(range_tbl.contains_key("open"), "expected 'open' variant key in range");
    }
}