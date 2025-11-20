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
//!     bar::BarSeries,
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
use snafu::{Backtrace, Snafu};

use crate::models::{bar::BarSeries, request_params::BarsRequestParams};

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
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ProviderInitError {
    /// missed environment variable.
    #[snafu(display("Missing environment variable: {source}"))]
    MissingEnvVar {
        source: MissingEnvVarError,
        backtrace: Backtrace,
    },

    /// failed to init reqwest client
    #[snafu(display("Failed to build HTTP client: {source}"))]
    ClientBuild {
        source: reqwest::Error,
        backtrace: Backtrace,
    },

    /// API key contains invalid characters.
    #[snafu(display("Invalid API key format: {source}"))]
    InvalidApiKey {
        source: reqwest::header::InvalidHeaderValue,
        backtrace: Backtrace,
    },
}

/// Errors that can occur within a `DataProvider` implementation.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ProviderError {
    /// An error during an API request (e.g., network failure, timeout).
    #[snafu(display("API request failed: {source}"))]
    Reqwest {
        source: reqwest::Error,
        backtrace: Backtrace,
    },

    /// The provider's API returned a specific error message (e.g., invalid API key).
    #[snafu(display("API error: {message}"))]
    Api {
        message: String,
        backtrace: Backtrace,
    },

    /// The request parameters were invalid for this specific provider.
    #[snafu(display("Invalid parameters for provider: {message}"))]
    Validation {
        message: String,
        backtrace: Backtrace,
    },

    /// An internal error occurred while processing data within the provider.
    #[snafu(display("Internal provider error: {message}"))]
    Internal {
        message: String,
        backtrace: Backtrace,
    },

    /// An error during provider configuration or initialization.
    #[snafu(display("Provider initialization error: {source}"))]
    Init {
        #[snafu(backtrace)]
        source: ProviderInitError,
    },
}
