//! Provider abstraction for market data sources.
//!
//! This module defines the [`DataProvider`] trait, which serves as a unified interface
//! for fetching time-series bar data from any market data vendor (e.g., Alpaca, Polygon.io).
//!
//! Each concrete provider implementation (such as Alpaca or Polygon) should implement
//! [`DataProvider`] to handle vendor-specific API logic and validation.
//!
//! The trait is designed for async usage and supports dynamic dispatch (`dyn DataProvider`)
//! for runtime selection of providers.
//!
//! # Example
//!
//! ```rust
//! # use market_data_ingestor::models::{bar::Bar, request_params::BarsRequestParams};
//! # use market_data_ingestor::providers::DataProvider;
//! # use async_trait::async_trait;
//! struct MyProvider;
//! #[async_trait]
//! impl DataProvider for MyProvider {
//!     async fn fetch_bars(&self, params: BarsRequestParams) -> Result<Vec<BarSeries>, anyhow::Error> {
//!         // Implementation here...
//!         Ok(vec![])
//!     }
//! }
//! ```
//! 
pub mod errors;

use async_trait::async_trait;

use crate::{errors::Error, models::{bar_series::BarSeries, request_params::BarsRequestParams}};

#[async_trait]
pub trait DataProvider {
    async fn fetch_bars(&self, params: BarsRequestParams) -> Result<Vec<BarSeries>, Error>;
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::Utc;

    use crate::models::{asset::AssetClass, timeframe::{TimeFrame, TimeFrameUnit}};

    use super::*;
    
    struct AlpacaProvider;
    struct PolygonProvider;

    #[async_trait]
    impl DataProvider for AlpacaProvider {
        async fn fetch_bars(&self, params: BarsRequestParams) -> Result<Vec<BarSeries>, Error> {
            println!("Fetching from alpaca for symbols: {:?}", params.symbols);
            Ok(vec![])
        }
    }

    #[async_trait]
    impl DataProvider for PolygonProvider {
        async fn fetch_bars(&self, params: BarsRequestParams) -> Result<Vec<BarSeries>, Error> {
            println!("Fetching from Polygon.io for symbols: {:?}", params.symbols);
            Ok(vec![]) // Return dummy data
        }
    }

    // This function decides AT RUNTIME which provider to give back.
    // It can only work because it returns a `Box<dyn DataProvider>`.
    fn get_provider(name: &str) -> Box<dyn DataProvider> {
        if name == "alpaca" {
            Box::new(AlpacaProvider)
        } else {
            Box::new(PolygonProvider)
        }
    }

    #[tokio::test]
    async fn test_dynamic_provider() {
        // We get a provider, but we don't know or care which one it is.
        // We just know it implements `DataProvider`.
        let provider = get_provider("polygon");

        let params = BarsRequestParams {
            symbols: vec!["ESU24".to_string()],
            timeframe: TimeFrame::new(1, TimeFrameUnit::Day),
            start: Utc::now(),
            end: Utc::now(),
            asset_class: AssetClass::Futures,
        };

        // We can call `fetch_bars` because it's part of the trait contract.
        let result = provider.fetch_bars(params).await;
        assert!(result.is_ok());
    }
}