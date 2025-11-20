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
//! use async_trait::async_trait;
//! use market_data_ingestor::models::{
//!     bar_series::BarSeries,
//!     request_params::BarsRequestParams,
//! };
//! use market_data_ingestor::providers::{DataProvider, ProviderError};
//!
//! struct MyProvider;
//!
//! #[async_trait]
//! impl DataProvider for MyProvider {
//!     async fn fetch_bars(
//!         &self,
//!         _params: BarsRequestParams,
//!     ) -> Result<Vec<BarSeries>, ProviderError> {
//!         Ok(vec![])
//!     }
//! }
//! ```
//!

pub mod alpaca_rest;

use async_trait::async_trait;
use shared_utils::env::MissingEnvVarError;
use thiserror::Error;

use crate::models::{bar_series::BarSeries, request_params::BarsRequestParams};

/// Trait for fetching time-series bar data from a market data provider.
///
/// Implement this trait for each concrete data vendor (e.g., Alpaca, Polygon).
/// The trait is designed for async usage and supports dynamic dispatch (`dyn DataProvider`)
/// for runtime selection of providers.
#[async_trait]
pub trait DataProvider {
    /// Fetches time-series bar data for the given request parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters specifying symbols, timeframe, and date range.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<BarSeries>)` - A vector of bar series, one per symbol.
    /// * `Err(Error)` - If the request fails, returns a unified error type.
    async fn fetch_bars(&self, params: BarsRequestParams) -> Result<Vec<BarSeries>, ProviderError>;
}

/// Errors that can occur during the creation of a provider instance
#[derive(Debug, Error)]
pub enum ProviderInitError {
    /// missed environment variable.
    #[error(transparent)]
    MissingEnvVar(#[from] MissingEnvVarError),

    /// failed to init reqwest client
    #[error(transparent)]
    ClientBuild(#[from] reqwest::Error),

    /// API key contains invalid characters.
    #[error("Invalid API key format: {0}")]
    InvalidApiKey(#[from] reqwest::header::InvalidHeaderValue),
}

/// Errors that can occur within a `DataProvider` implementation.
#[derive(Debug, Error)]
pub enum ProviderError {
    /// An error during an API request (e.g., network failure, timeout).
    #[error("API request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// The provider's API returned a specific error message (e.g., invalid API key).
    #[error("API error: {0}")]
    Api(String),

    /// The request parameters were invalid for this specific provider.
    #[error("Invalid parameters for provider: {0}")]
    Validation(String),

    /// An internal error occurred while processing data within the provider.
    #[error("Internal provider error: {0}")]
    Internal(String),

    /// An error during provider configuration or initialization.
    #[error(transparent)]
    Init(#[from] ProviderInitError),
}
