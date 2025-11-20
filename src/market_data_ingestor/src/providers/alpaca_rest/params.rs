use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    models::{
        request_params::{BarsRequestParams, ProviderParams},
        timeframe::{TimeFrame, TimeFrameUnit},
    },
    providers::ProviderError,
};

/// Alpaca subscription plans with different rate limits
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlpacaSubscriptionPlan {
    #[default]
    Basic, // 200 reqs/min
    AlgoTrader, // 10,000 reqs/min
}

impl AlpacaSubscriptionPlan {
    /// Returns the rate limit in requests per minute for this subscription plan
    pub fn rate_limit_per_minute(&self) -> u32 {
        match self {
            AlpacaSubscriptionPlan::Basic => 200,
            AlpacaSubscriptionPlan::AlgoTrader => 10_000,
        }
    }
}

/// Specifies the corporate action adjustment for stock data.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Adjustment {
    #[default]
    Raw,
    Split,
    Dividend,
    All,
}

/// Specifies the source feed for stock data.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Feed {
    #[default]
    Sip,
    Iex,
    Otc,
}

/// Specifies the sort order for the bars.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    #[default]
    Asc,
    Desc,
}

/// Alpaca-specific parameters for a bars request.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AlpacaBarsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment: Option<Adjustment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed: Option<Feed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    /// Subscription plan for validation (defaults to Basic)
    #[serde(default)]
    pub subscription_plan: AlpacaSubscriptionPlan,
}

fn format_timeframe_str(tf: &TimeFrame) -> String {
    let unit_str = match tf.unit {
        TimeFrameUnit::Minute => "Min",
        TimeFrameUnit::Hour => "Hour",
        TimeFrameUnit::Day => "Day",
        TimeFrameUnit::Week => "Week",
        TimeFrameUnit::Month => "Month",
    };
    format!("{}{}", tf.amount, unit_str)
}

pub fn validate_timeframe(tf: &TimeFrame) -> Result<(), ProviderError> {
    match tf.unit {
        TimeFrameUnit::Minute if !(1..=59).contains(&tf.amount) => {
            Err(ProviderError::Validation(format!(
                "Alpaca supports 1-59 for Minute timeframes, but got {}",
                tf.amount
            )))
        }
        TimeFrameUnit::Hour if !(1..=23).contains(&tf.amount) => {
            Err(ProviderError::Validation(format!(
                "Alpaca supports 1-23 for Hour timeframes, but got {}",
                tf.amount
            )))
        }
        TimeFrameUnit::Day | TimeFrameUnit::Week if tf.amount != 1 => {
            Err(ProviderError::Validation(format!(
                "Alpaca only supports an amount of 1 for Day and Week timeframes, but got {}",
                tf.amount
            )))
        }
        TimeFrameUnit::Month if ![1, 2, 3, 4, 6, 12].contains(&tf.amount) => {
            Err(ProviderError::Validation(format!(
                "Alpaca only supports amounts of 1, 2, 3, 4, 6, or 12 for Month timeframes, but got {}",
                tf.amount
            )))
        }
        _ => Ok(()),
    }
}

pub fn construct_params(params: &BarsRequestParams) -> Vec<(String, String)> {
    let symbols = params.symbols.join(",");
    let timeframe = format_timeframe_str(&params.timeframe);

    let alpaca_params = match &params.provider_specific {
        ProviderParams::Alpaca(p) => p.clone(),
        _ => AlpacaBarsParams::default(),
    };

    let mut query_params = vec![
        ("symbols".to_string(), symbols),
        ("timeframe".to_string(), timeframe),
        ("start".to_string(), params.start.to_rfc3339()),
        ("end".to_string(), params.end.to_rfc3339()),
    ];

    // Add optional parameters if they exist
    if let Some(adjustment) = alpaca_params.adjustment {
        query_params.push((
            "adjustment".to_string(),
            serde_json::to_string(&adjustment)
                .expect("Serializing Adjustment enum should never fail")
                .replace('"', ""),
        ));
    }
    if let Some(feed) = alpaca_params.feed {
        query_params.push((
            "feed".to_string(),
            serde_json::to_string(&feed)
                .expect("Serializing Feed enum should never fail")
                .replace('"', ""),
        ));
    }
    if let Some(currency) = alpaca_params.currency {
        query_params.push(("currency".to_string(), currency));
    }
    if let Some(limit) = alpaca_params.limit {
        query_params.push(("limit".to_string(), limit.to_string()));
    }
    if let Some(sort) = alpaca_params.sort {
        query_params.push((
            "sort".to_string(),
            serde_json::to_string(&sort)
                .expect("Serializing Sort enum should never fail")
                .replace('"', ""),
        ));
    }

    query_params
}

/// Validates the date range based on Alpaca's historical data limitations
pub fn validate_date_range(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    plan: &AlpacaSubscriptionPlan,
) -> Result<(), ProviderError> {
    let now = Utc::now();

    // Both plans support data since 2016
    let earliest_date = DateTime::parse_from_rfc3339("2016-01-01T00:00:00Z")
        .expect("Hardcoded RFC3339 date string is valid")
        .with_timezone(&Utc);

    if start < earliest_date {
        return Err(ProviderError::Validation(format!(
            "Alpaca historical data is only available since 2016-01-01, but start date is {}",
            start.format("%Y-%m-%d")
        )));
    }

    // Basic plan has 15-minute delay limitation
    match plan {
        AlpacaSubscriptionPlan::Basic => {
            let delay_threshold = now - Duration::minutes(15);
            if end > delay_threshold {
                return Err(ProviderError::Validation(format!(
                    "Basic plan has a 15-minute delay. Latest available data is at {}. Requested end time: {}",
                    delay_threshold.format("%Y-%m-%d %H:%M:%S UTC"),
                    end.format("%Y-%m-%d %H:%M:%S UTC")
                )));
            }
        }
        AlpacaSubscriptionPlan::AlgoTrader => {
            // No restriction for Algo Trader Plus
        }
    }

    // Basic date range validation
    if start >= end {
        return Err(ProviderError::Validation(
            "Start date must be before end date".to_string(),
        ));
    }

    Ok(())
}

/// Combined validation function that checks both timeframe and date range
pub fn validate_request(params: &BarsRequestParams) -> Result<(), ProviderError> {
    // Validate timeframe
    validate_timeframe(&params.timeframe)?;

    // Extract subscription plan from provider-specific params
    let plan = match &params.provider_specific {
        ProviderParams::Alpaca(alpaca_params) => &alpaca_params.subscription_plan,
        _ => &AlpacaSubscriptionPlan::Basic, // Default to Basic for safety
    };

    // Validate date range
    validate_date_range(params.start, params.end, plan)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{asset::AssetClass, timeframe::TimeFrame};
    use chrono::Utc;

    #[test]
    fn test_validate_timeframe_valid() {
        assert!(validate_timeframe(&TimeFrame::new(5, TimeFrameUnit::Minute)).is_ok());
        assert!(validate_timeframe(&TimeFrame::new(1, TimeFrameUnit::Hour)).is_ok());
        assert!(validate_timeframe(&TimeFrame::new(1, TimeFrameUnit::Day)).is_ok());
        assert!(validate_timeframe(&TimeFrame::new(1, TimeFrameUnit::Week)).is_ok());
        assert!(validate_timeframe(&TimeFrame::new(3, TimeFrameUnit::Month)).is_ok());
    }

    #[test]
    fn test_validate_timeframe_invalid() {
        assert!(validate_timeframe(&TimeFrame::new(60, TimeFrameUnit::Minute)).is_err());
        assert!(validate_timeframe(&TimeFrame::new(0, TimeFrameUnit::Minute)).is_err());
        assert!(validate_timeframe(&TimeFrame::new(24, TimeFrameUnit::Hour)).is_err());
        assert!(validate_timeframe(&TimeFrame::new(2, TimeFrameUnit::Day)).is_err());
        assert!(validate_timeframe(&TimeFrame::new(5, TimeFrameUnit::Month)).is_err());
    }

    #[test]
    fn test_construct_params_basic() {
        let params = BarsRequestParams {
            symbols: vec!["AAPL".to_string(), "MSFT".to_string()],
            timeframe: TimeFrame::new(1, TimeFrameUnit::Day),
            start: Utc::now(),
            end: Utc::now(),
            asset_class: AssetClass::UsEquity,
            provider_specific: ProviderParams::None,
        };

        let query = construct_params(&params);
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("symbols").unwrap(), "AAPL,MSFT");
        assert_eq!(query_map.get("timeframe").unwrap(), "1Day");
    }

    #[test]
    fn test_construct_params_with_specifics() {
        let params = BarsRequestParams {
            symbols: vec!["SPY".to_string()],
            timeframe: TimeFrame::new(15, TimeFrameUnit::Minute),
            start: Utc::now(),
            end: Utc::now(),
            asset_class: AssetClass::UsEquity,
            provider_specific: ProviderParams::Alpaca(AlpacaBarsParams {
                limit: Some(100),
                sort: Some(Sort::Desc),
                ..Default::default()
            }),
        };

        let query = construct_params(&params);
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("limit").unwrap(), "100");
        assert_eq!(query_map.get("sort").unwrap(), "desc");
    }
    #[test]
    fn test_validate_date_range_basic_plan() {
        let now = Utc::now();
        let plan = AlpacaSubscriptionPlan::Basic;

        // Valid date range (more than 15 minutes ago)
        let start = now - Duration::hours(2);
        let end = now - Duration::minutes(20);
        assert!(validate_date_range(start, end, &plan).is_ok());

        // Invalid: too recent for Basic plan
        let start = now - Duration::hours(1);
        let end = now - Duration::minutes(5);
        assert!(validate_date_range(start, end, &plan).is_err());

        // Invalid: before 2016
        let start = DateTime::parse_from_rfc3339("2015-12-31T00:00:00Z")
            .expect("Test date string is valid")
            .with_timezone(&Utc);
        let end = now - Duration::hours(1);
        assert!(validate_date_range(start, end, &plan).is_err());
    }

    #[test]
    fn test_validate_date_range_algo_trader() {
        let now = Utc::now();
        let plan = AlpacaSubscriptionPlan::AlgoTrader;

        // Valid: recent data (allowed for Algo Trader)
        let start = now - Duration::minutes(30);
        let end = now - Duration::minutes(1);
        assert!(validate_date_range(start, end, &plan).is_ok());

        // Invalid: before 2016 (applies to all plans)
        let start = DateTime::parse_from_rfc3339("2015-12-31T00:00:00Z")
            .expect("Test date string is valid")
            .with_timezone(&Utc);
        let end = now - Duration::hours(1);
        assert!(validate_date_range(start, end, &plan).is_err());
    }

    #[test]
    fn test_validate_request_integration() {
        let now = Utc::now();

        // Valid request for Basic plan
        let params = BarsRequestParams {
            symbols: vec!["AAPL".to_string()],
            timeframe: TimeFrame::new(1, TimeFrameUnit::Day),
            start: now - Duration::days(30),
            end: now - Duration::hours(1),
            asset_class: AssetClass::UsEquity,
            provider_specific: ProviderParams::Alpaca(AlpacaBarsParams {
                subscription_plan: AlpacaSubscriptionPlan::Basic,
                ..Default::default()
            }),
        };
        assert!(validate_request(&params).is_ok());

        // Invalid request for Basic plan (too recent)
        let params = BarsRequestParams {
            symbols: vec!["AAPL".to_string()],
            timeframe: TimeFrame::new(1, TimeFrameUnit::Day),
            start: now - Duration::minutes(30),
            end: now - Duration::minutes(5),
            asset_class: AssetClass::UsEquity,
            provider_specific: ProviderParams::Alpaca(AlpacaBarsParams {
                subscription_plan: AlpacaSubscriptionPlan::Basic,
                ..Default::default()
            }),
        };
        assert!(validate_request(&params).is_err());
    }
}
