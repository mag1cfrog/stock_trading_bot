use async_trait::async_trait;
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use reqwest::{header, Client};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use shared_utils::env::get_env_var;

use crate::{models::{bar::Bar, bar_series::BarSeries, request_params::BarsRequestParams, timeframe::{TimeFrame, TimeFrameUnit}}, providers::{DataProvider, ProviderError, ProviderInitError}};

const BASE_URL: &str = "https://data.alpaca.markets/v2/stocks/bars";

#[derive(Deserialize, Debug)]
struct AlpacaBar {
    #[serde(rename = "t")]
    timestamp: DateTime<Utc>,
    #[serde(rename = "o")]
    open: f64,
    #[serde(rename = "h")]
    high: f64,
    #[serde(rename = "l")]
    low: f64,
    #[serde(rename = "c")]
    close: f64,
    #[serde(rename = "v")]
    volume: f64,
    #[serde(rename = "n")]
    trade_count: u64,
    #[serde(rename = "vw")]
    vwap: f64,
}

#[derive(Deserialize, Debug)]
struct AlpacaResponse {
    bars: IndexMap<String, Vec<AlpacaBar>>,
    next_page_token: Option<String>,
}

pub struct AlpacaProvider {
    client: Client,
    _api_key: SecretString,
    _secret_key: SecretString,
}

impl AlpacaProvider {
    /// Creates a new Alpaca provider.
    ///
    /// Reads API keys from the `APCA_API_KEY_ID` and `APCA_API_SECRET_KEY`
    /// environment variables.
    pub fn new() -> Result<Self, ProviderInitError> {
        let api_key = SecretString::new(get_env_var("APCA_API_KEY_ID")?.into());
        let secret_key = SecretString::new(get_env_var("APCA_API_SECRET_KEY")?.into());

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "APCA-API-KEY-ID",
            header::HeaderValue::from_str(api_key.expose_secret())?,
        );
        headers.insert(
            "APCA_API_SECRET_KEY",
            header::HeaderValue::from_str(secret_key.expose_secret())?,
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            _api_key: api_key,
            _secret_key: secret_key,
        })
    }
}

fn format_timeframe(tf: &TimeFrame) -> String {
    let unit_str = match tf.unit {
        TimeFrameUnit::Minute => "Min",
        TimeFrameUnit::Hour => "Hour",
        TimeFrameUnit::Day => "Day",
        TimeFrameUnit::Week => "Week",
        TimeFrameUnit::Month => "Month",
    };
    format!("{}{}", tf.amount, unit_str)
}

#[async_trait]
impl DataProvider for AlpacaProvider {
    async fn fetch_bars(&self, params: BarsRequestParams) -> Result<Vec<BarSeries>, ProviderError>{
        // TODO: Implement Alpaca-specific timeframe validation here.

        let symbols = params.symbols.join(",");
        let timeframe = format_timeframe(&params.timeframe);

        // TODO: Implement pagination using `next_page_token`.
        // This initial implementation only fetches the first page.
        let response = self
            .client
            .get(BASE_URL)
            .query(&[
                ("symbols", symbols.as_str()),
                ("timeframe", &timeframe),
                ("start", &params.start.to_rfc3339()),
                ("end", &params.end.to_rfc3339()),
                ("limit", "10000"),
                ("adjustment", "raw"),
                ("feed", "sip"),
                ("sort", "asc"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown API error".to_string());
            return Err(ProviderError::Api(error_msg));
        }

        let alpaca_response = response
            .json::<AlpacaResponse>()
            .await?;

        // Convert the HashMap into a Vec<BarSeries>
        let result: Vec<BarSeries> = alpaca_response
            .bars
            .into_iter()
            .map(|(symbol, alpaca_bars)| {
                let bars = alpaca_bars
                    .into_iter()
                    .map(|ab| Bar {
                        timestamp: ab.timestamp,
                        open: ab.open,
                        high: ab.high,
                        low: ab.low,
                        close: ab.close,
                        volume: ab.volume,
                        trade_count: Some(ab.trade_count),
                        vwap: Some(ab.vwap),
                    })
                    .collect();

                BarSeries {
                    symbol,
                    timeframe: params.timeframe.clone(),
                    bars,
                }
            })
            .collect();

        Ok(result)
    }
}