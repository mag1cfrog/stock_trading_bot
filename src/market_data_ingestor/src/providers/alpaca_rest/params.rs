use serde::{Deserialize, Serialize};


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