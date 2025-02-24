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
use crate::utils::python_init::{read_config, Config};


#[allow(unused)]
pub struct StockBarData {
    config: Config,
}

impl StockBarData {
    // async fn validate_python_env(path: &Path) -> Result<String, MarketDataError> {
    //     let site_packages = path.join("site-packages");
    //     let alpaca_path = site_packages.join("alpaca");

    //     if !site_packages.exists() {
    //         return Err(MarketDataError::MissingSitePackages(
    //             site_packages.display().to_string(),
    //         ));
    //     }

    //     // Explicit check for alpaca package
    //     if !alpaca_path.exists() {
    //         return Err(MarketDataError::MissingAlpacaPackage(
    //             alpaca_path.display().to_string(),
    //         ));
    //     }

    //     Ok(site_packages.to_string_lossy().into_owned())
    // }

    pub async fn new(config_path: &str) -> Result<Self, MarketDataError> {

        let config = read_config(config_path).unwrap();

        // let venv_path = Path::new(&config.python_venv_path);
        // // Validate path exists
        // if !venv_path.exists() {
        //     return Err(MarketDataError::InvalidPath(
        //         venv_path.display().to_string(),
        //     ));
        // }

        crate::utils::python_init::verify_shell_environment()
            .map_err(|e| MarketDataError::EnvError(e.to_string()))?;

        // Initialize Python environment using the utility
        init_python(config_path).unwrap();

        // // Find Python lib path with alpaca package
        // let lib_dir = venv_path.join("lib");
        // let mut entries = fs::read_dir(&lib_dir)
        //     .await
        //     .map_err(|_| MarketDataError::InvalidPath(lib_dir.display().to_string()))?;

        // // Find Python site-package directory
        // let mut site_packages_path = None;
        // while let Some(entry) = entries
        //     .next_entry()
        //     .await
        //     .map_err(|e| MarketDataError::NoPythonVersionFound(e.to_string()))?
        // {
        //     let path = entry.path();
        //     if path
        //         .file_name()
        //         .and_then(|n| n.to_str())
        //         .map(|s| s.starts_with("python"))
        //         .unwrap_or(false)
        //     {
        //         match Self::validate_python_env(&path).await {
        //             Ok(path) => {
        //                 site_packages_path = Some(path);
        //                 break;
        //             }
        //             Err(MarketDataError::MissingAlpacaPackage(_)) => continue, // Try next Python version
        //             Err(e) => return Err(e),
        //         }
        //     }
        // }

        // let site_packages_path = site_packages_path
        //     .ok_or_else(|| MarketDataError::NoPythonVersionFound(lib_dir.display().to_string()))?;

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
        base_delay_ms: u64
    ) -> Result<Vec<Result<DataFrame, MarketDataError>>, Box<dyn Error>> {
        fetch_bars_batch_partial(self, params_list, max_retries, base_delay_ms)
    }
}
