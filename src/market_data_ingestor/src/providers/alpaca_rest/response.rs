use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AlpacaBar {
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "o")]
    pub open: f64,
    #[serde(rename = "h")]
    pub high: f64,
    #[serde(rename = "l")]
    pub low: f64,
    #[serde(rename = "c")]
    pub close: f64,
    #[serde(rename = "v")]
    pub volume: f64,
    #[serde(rename = "n")]
    pub trade_count: u64,
    #[serde(rename = "vw")]
    pub vwap: f64,
}

#[derive(Deserialize, Debug)]
pub struct AlpacaResponse {
    pub bars: IndexMap<String, Vec<AlpacaBar>>,
    pub next_page_token: Option<String>,
}
