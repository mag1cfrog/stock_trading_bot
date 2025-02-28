mod errors;
pub use errors::MarketDataError;

mod single_request;
pub use single_request::fetch_historical_bars;

mod batch_request;
pub use batch_request::fetch_bars_batch_partial;

use std::error::Error;

use polars::prelude::*;

use crate::models::stockbars::StockBarsParams;
use crate::utils::init_python;
use crate::utils::python_init::{Config, read_config};

#[allow(unused)]
pub struct StockBarData {
    config: Config,
}

impl StockBarData {
    pub async fn new(config_path: &str) -> Result<Self, MarketDataError> {
        let config = read_config(config_path).unwrap();

        // crate::utils::python_init::verify_shell_environment()
        //     .map_err(|e| MarketDataError::EnvError(e.to_string()))?;

        // Initialize Python environment using the utility
        init_python(config_path).unwrap();

        Ok(Self { config })
    }

    pub fn fetch_historical_bars(
        &self,
        params: StockBarsParams,
    ) -> Result<DataFrame, Box<dyn Error>> {
        fetch_historical_bars(self, params)
    }

    pub fn fetch_bars_batch_partial(
        &self,
        params_list: &[StockBarsParams],
        max_retries: u32,
        base_delay_ms: u64,
    ) -> Result<Vec<Result<DataFrame, MarketDataError>>, Box<dyn Error>> {
        fetch_bars_batch_partial(self, params_list, max_retries, base_delay_ms)
    }
}
