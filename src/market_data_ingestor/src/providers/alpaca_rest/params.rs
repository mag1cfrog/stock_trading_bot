use serde::{Deserialize, Serialize};

use crate::{
    models::{
        request_params::{BarsRequestParams, ProviderParams},
        timeframe::{TimeFrame, TimeFrameUnit},
    },
    providers::ProviderError,
};

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
            serde_json::to_string(&adjustment).unwrap().replace('"', ""),
        ));
    }
    if let Some(feed) = alpaca_params.feed {
        query_params.push((
            "feed".to_string(),
            serde_json::to_string(&feed).unwrap().replace('"', ""),
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
            serde_json::to_string(&sort).unwrap().replace('"', ""),
        ));
    }

    query_params
}
