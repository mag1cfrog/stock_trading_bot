use errors::IngestorError;
use requests::historical::StockBarData;
use utils::python_init::Config;

pub mod cli;
pub mod errors;
pub mod io;
pub mod models;
pub mod requests;
pub mod utils;

pub async fn create_client(config_path: &str) -> Result<StockBarData, IngestorError> {
    StockBarData::new(config_path).await
}

// New function - create client with direct config
pub async fn create_client_with_config(config: Config) -> Result<StockBarData, IngestorError> {
    StockBarData::with_config(config).await
}