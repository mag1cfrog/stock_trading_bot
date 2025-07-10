use async_trait::async_trait;
use indexmap::IndexMap;
use reqwest::{header, Client};
use secrecy::{ExposeSecret, SecretString};
use shared_utils::env::get_env_var;

use crate::{models::{bar::Bar, bar_series::BarSeries, request_params::BarsRequestParams}, providers::{alpaca_rest::{params::{construct_params, validate_timeframe}, response::{AlpacaBar, AlpacaResponse}}, DataProvider, ProviderError, ProviderInitError}};



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



#[async_trait]
impl DataProvider for AlpacaProvider {
    async fn fetch_bars(&self, params: BarsRequestParams) -> Result<Vec<BarSeries>, ProviderError>{
        // Validate the timeframe before proceeding.
        validate_timeframe(&params.timeframe)?;

        let mut all_bars: IndexMap<String, Vec<AlpacaBar>> = IndexMap::new();
        let mut next_page_token: Option<String> = None;

        loop {
            let mut query_params = construct_params(&params);
            if let Some(token) = &next_page_token {
                query_params.push(("page_token".to_string(), token.clone()));
            }

            let response = self.client.get(BASE_URL).query(&query_params).send().await?;

            if !response.status().is_success() {
                let error_msg = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown API error".to_string());
                return Err(ProviderError::Api(error_msg));
            }

            let alpaca_response = response.json::<AlpacaResponse>().await?;

            // Merge the bars from the current page into our collection.
            for (symbol, bars) in alpaca_response.bars {
                all_bars.entry(symbol).or_default().extend(bars);
            }

            // If there's a next page token, use it for the next iteration. Otherwise, we're done.
            if let Some(token) = alpaca_response.next_page_token {
                next_page_token = Some(token);
            } else {
                break;
            }
        }

        // Convert the accumulated bars into the final Vec<BarSeries>
        let result = all_bars
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