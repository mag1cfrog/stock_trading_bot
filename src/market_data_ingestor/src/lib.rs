use errors::IngestorError;
use requests::historical::StockBarData;

pub mod cli;
pub mod errors;
pub mod io;
pub mod models;
pub mod requests;
pub mod utils;

pub async fn create_client(config_path: &str) -> Result<StockBarData, IngestorError> {
    StockBarData::new(config_path).await
}
