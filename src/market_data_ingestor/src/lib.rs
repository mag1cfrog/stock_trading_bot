#[cfg(feature = "alpaca-python-sdk")]
use legacy_errors::IngestorError;
#[cfg(feature = "alpaca-python-sdk")]
use requests::historical::StockBarData;
#[cfg(feature = "alpaca-python-sdk")]
use utils::python_init::Config;

#[cfg(feature = "cli")]
pub mod cli;
pub mod errors;
pub mod io;
#[cfg(feature = "alpaca-python-sdk")]
pub mod legacy_errors;
pub mod models;
pub mod providers;
pub mod requests;
pub mod utils;

#[cfg(feature = "alpaca-python-sdk")]
pub async fn create_client(config_path: &str) -> Result<StockBarData, IngestorError> {
    StockBarData::new(config_path).await
}

// New function - create client with direct config
#[cfg(feature = "alpaca-python-sdk")]
pub async fn create_client_with_config(config: Config) -> Result<StockBarData, IngestorError> {
    StockBarData::with_config(config).await
}
