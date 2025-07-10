use async_trait::async_trait;
use reqwest::{header, Client};
use secrecy::{ExposeSecret, SecretString};
use shared_utils::env::get_env_var;

use crate::{models::{bar::Bar, bar_series::BarSeries, request_params::{BarsRequestParams, ProviderParams}, timeframe::{TimeFrame, TimeFrameUnit}}, providers::{alpaca_rest::{params::AlpacaBarsParams, response::AlpacaResponse}, DataProvider, ProviderError, ProviderInitError}};



const BASE_URL: &str = "https://data.alpaca.markets/v2/stocks/bars";

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
            query_params.push(("adjustment".to_string(), serde_json::to_string(&adjustment).unwrap().replace('"', "")));
        }
        if let Some(feed) = alpaca_params.feed {
            query_params.push(("feed".to_string(), serde_json::to_string(&feed).unwrap().replace('"', "")));
        }
        if let Some(currency) = alpaca_params.currency {
            query_params.push(("currency".to_string(), currency));
        }
        if let Some(limit) = alpaca_params.limit {
            query_params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(sort) = alpaca_params.sort {
            query_params.push(("sort".to_string(), serde_json::to_string(&sort).unwrap().replace('"', "")));
        }

        let response = self
            .client
            .get(BASE_URL)
            .query(&query_params)
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