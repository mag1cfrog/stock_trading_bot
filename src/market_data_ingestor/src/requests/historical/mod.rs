/// Historical market data fetching functionality.
///
/// This module provides multiple ways to fetch market data:
/// - Memory-based methods: Return data directly as DataFrames
/// - File-based methods: Write data to temporary files and return paths
mod legacy_errors;
pub use legacy_errors::MarketDataError;

#[cfg(feature = "alpaca-python-sdk")]
mod single_request;
#[cfg(feature = "alpaca-python-sdk")]
pub use single_request::fetch_historical_bars;

mod batch_request;
#[cfg(feature = "alpaca-python-sdk")]
pub use batch_request::fetch_bars_batch_partial;

use std::path::PathBuf;

use polars::prelude::*;

#[cfg(feature = "alpaca-python-sdk")]
use crate::io::dataframe::write_dataframe_to_temp;
#[cfg(feature = "alpaca-python-sdk")]
use crate::io::legacy_errors::IOError;
use crate::legacy_errors::IngestorError;
#[cfg(feature = "alpaca-python-sdk")]
use crate::models::stockbars::StockBarsParams;
#[cfg(feature = "alpaca-python-sdk")]
use crate::utils::init_python;
#[cfg(feature = "alpaca-python-sdk")]
use crate::utils::python_init::{Config, init_python_with_config, read_config};

#[allow(unused)]
pub struct StockBarData {
    #[cfg(feature = "alpaca-python-sdk")]
    config: Config,
}

pub type InMemoryResult = Result<DataFrame, IngestorError>;
pub type FilePathResult = Result<PathBuf, IngestorError>;

impl StockBarData {
    #[cfg(feature = "alpaca-python-sdk")]
    pub async fn new(config_path: &str) -> Result<Self, IngestorError> {
        let config = read_config(config_path).unwrap();

        // Initialize Python environment using the utility
        init_python(config_path).unwrap();

        Ok(Self { config })
    }

    // New method that accepts Config directly
    #[cfg(feature = "alpaca-python-sdk")]
    pub async fn with_config(config: Config) -> Result<Self, IngestorError> {
        // Initialize Python environment with the provided config
        init_python_with_config(&config).map_err(|e| {
            IngestorError::SystemError(format!("Python initialization error: {}", e))
        })?;

        Ok(Self { config })
    }

    // Enhanced API: Direct memory methods

    /// Fetches historical bars data and returns it directly as a DataFrame
    #[cfg(feature = "alpaca-python-sdk")]
    pub fn fetch_historical_bars_to_memory(
        &self,
        params: StockBarsParams,
    ) -> Result<DataFrame, IngestorError> {
        fetch_historical_bars(self, params).map_err(Into::into)
    }

    /// Fetches batch historical data and returns results directly
    #[cfg(feature = "alpaca-python-sdk")]
    pub fn fetch_bars_batch_to_memory(
        &self,
        params_list: &[StockBarsParams],
        max_retries: u32,
        base_delay_ms: u64,
    ) -> Result<Vec<Result<DataFrame, IngestorError>>, IngestorError> {
        fetch_bars_batch_partial(self, params_list, max_retries, base_delay_ms)
            .map_err(Into::into)
            .map(|results| {
                results
                    .into_iter()
                    .map(|res| res.map_err(Into::into))
                    .collect()
            })
    }

    // File-based methods (for backward compatibility)

    /// Fetches historical bars and writes to a temporary file, returning the file path
    #[cfg(feature = "alpaca-python-sdk")]
    pub fn fetch_historical_bars_to_file(&self, params: StockBarsParams) -> FilePathResult {
        let symbol = params
            .symbols
            .first()
            .ok_or_else(|| IngestorError::SystemError("No symbols provided".to_string()))?;
        let mut df = fetch_historical_bars(self, params.clone())?;

        // Write to file (returns IOError)
        let path = write_dataframe_to_temp(&mut df, symbol)?;

        // Both errors automatically convert to IngestorError
        Ok(path)
    }

    /// Batch fetches historical data and writes successful results to temporary files
    #[cfg(feature = "alpaca-python-sdk")]
    pub fn fetch_batch_to_files(
        &self,
        params_list: &[StockBarsParams],
        max_retries: u32,
        base_delay_ms: u64,
    ) -> Result<Vec<FilePathResult>, IngestorError> {
        let results = fetch_bars_batch_partial(self, params_list, max_retries, base_delay_ms)?;

        let mut file_results: Vec<Result<PathBuf, IngestorError>> =
            Vec::with_capacity(results.len());

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(mut df) => {
                    if let Some(symbol) = params_list.get(i).and_then(|p| p.symbols.first()) {
                        match write_dataframe_to_temp(&mut df, symbol) {
                            Ok(path) => file_results.push(Ok(path)),
                            Err(e) => file_results.push(Err(IngestorError::from(e))),
                        }
                    } else {
                        file_results.push(Err(IngestorError::IO(IOError::InvalidSymbol(
                            "Missing symbol for batch item".to_string(),
                        ))));
                    }
                }
                Err(e) => file_results.push(Err(IngestorError::from(e))),
            }
        }

        Ok(file_results)
    }

    // Original methods (for backward compatibility)
    #[cfg(feature = "alpaca-python-sdk")]
    pub fn fetch_historical_bars(
        &self,
        params: StockBarsParams,
    ) -> Result<DataFrame, IngestorError> {
        fetch_historical_bars(self, params).map_err(Into::into)
    }

    #[cfg(feature = "alpaca-python-sdk")]
    pub fn fetch_bars_batch_partial(
        &self,
        params_list: &[StockBarsParams],
        max_retries: u32,
        base_delay_ms: u64,
    ) -> Result<Vec<Result<DataFrame, IngestorError>>, IngestorError> {
        fetch_bars_batch_partial(self, params_list, max_retries, base_delay_ms)
            .map_err(Into::into)
            .map(|results| {
                results
                    .into_iter()
                    .map(|res| res.map_err(Into::into))
                    .collect()
            })
    }
}
