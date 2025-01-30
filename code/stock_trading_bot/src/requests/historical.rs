use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::ffi::CString;
use std::fmt;
use std::path::Path;
use tokio::fs;

use crate::models::stockbars::StockBarsParams;

#[derive(Debug)]
pub enum MarketDataError {
    InvalidPath(String),
    MissingSitePackages(String),
    MissingAlpacaPackage(String),
    NoPythonVersionFound(String),
    AlpacaAPIError { py_type: String, message: String },
    PythonExecutionError(String),
}

impl fmt::Display for MarketDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            Self::MissingSitePackages(path) => {
                write!(f, "Missing site-packages directory: {}", path)
            }
            Self::MissingAlpacaPackage(path) => write!(f, "Missing Alpaca package: {}", path),
            Self::NoPythonVersionFound(msg) => write!(f, "No Python version found: {}", msg),
            Self::AlpacaAPIError { py_type, message } => {
                write!(f, "Alpaca API error({}): {}", py_type, message)
            }
            Self::PythonExecutionError(msg) => write!(f, "Python execution error: {}", msg),
        }
    }
}

impl std::error::Error for MarketDataError {}

pub struct StockBarData {
    site_packages_path: String,
}

impl StockBarData {
    async fn validate_python_env(path: &Path) -> Result<String, MarketDataError> {
        let site_packages = path.join("site-packages");
        let alpaca_path = site_packages.join("alpaca");

        if !site_packages.exists() {
            return Err(MarketDataError::MissingSitePackages(
                site_packages.display().to_string(),
            ));
        }

        // Explicit check for alpaca package
        if !alpaca_path.exists() {
            return Err(MarketDataError::MissingAlpacaPackage(
                alpaca_path.display().to_string(),
            ));
        }

        Ok(site_packages.to_string_lossy().into_owned())
    }

    pub async fn new(venv_path: &Path) -> Result<Self, MarketDataError> {
        // Validate path exists
        if !venv_path.exists() {
            return Err(MarketDataError::InvalidPath(
                venv_path.display().to_string(),
            ));
        }

        // Find Python lib path with alpaca package
        let lib_dir = venv_path.join("lib");
        let mut entries = fs::read_dir(&lib_dir)
            .await
            .map_err(|_| MarketDataError::InvalidPath(lib_dir.display().to_string()))?;

        // Find Python site-package directory
        let mut site_packages_path = None;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| MarketDataError::NoPythonVersionFound(e.to_string()))?
        {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.starts_with("python"))
                .unwrap_or(false)
            {
                match Self::validate_python_env(&path).await {
                    Ok(path) => {
                        site_packages_path = Some(path);
                        break;
                    }
                    Err(MarketDataError::MissingAlpacaPackage(_)) => continue, // Try next Python version
                    Err(e) => return Err(e),
                }
            }
        }

        let site_packages_path = site_packages_path
            .ok_or_else(|| MarketDataError::NoPythonVersionFound(lib_dir.display().to_string()))?;

        Ok(Self { site_packages_path })
    }

    pub fn fetch_historical_bars(
        &self,
        params: StockBarsParams,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        // Initialize Python first without environment vars
        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| {
            // Set up virtual environment paths after Python is initialized

            // Add virtual environment's site-packages to Python's path
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;
            path.call_method1("insert", (0, &self.site_packages_path))?;

            // Convert parameters to Python object
            let py_request = params.into_pyobject(py)?;

            let code = r#"
from datetime import datetime
import os

from alpaca.data.historical import StockHistoricalDataClient
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame
import polars as pl

# Should use the virtual environment's packages
print("Alpaca version:", StockHistoricalDataClient.__module__)

alpaca_key = os.getenv('APCA_API_KEY_ID')
secret_key = os.getenv('APCA_API_SECRET_KEY')
# Initialize client with environment variables
client = StockHistoricalDataClient(api_key=alpaca_key, secret_key=secret_key)

bars = client.get_stock_bars(request_params) # Use injected params
df = bars.df

# Convert to Polars DataFrame
pl_df = pl.from_pandas(df)

# Write to in-memory Arrow IPC file (Feather format)
arrow_data = pl_df.write_ipc(
    file=None,  # Return BytesIO
    compression='uncompressed',
    compat_level=pl.CompatLevel.newest()  # Ensures Rust compatibility
).getvalue()
"#;

            let locals = PyDict::new(py);
            locals.set_item("request_params", py_request)?;

            // let globals = PyDict::new(py);

            // Convert the code string to CString
            let code = CString::new(code).unwrap();
            match py.run(&code, None, Some(&locals)) {
                Ok(_) => {
                    // Get IPC bytes from Python
                    let ipc_bytes: Vec<u8> = locals
                        .get_item("arrow_data")
                        .unwrap()
                        .expect("Can't get Python arrow data.")
                        .extract()?;

                    // Read directly into Polars DataFrame
                    let df = IpcReader::new(std::io::Cursor::new(ipc_bytes)).finish()?;

                    Ok(df)
                }
                Err(e) => {
                    let py_err_str = e.to_string();
                    let name_result = e.get_type(py).name();
                    let is_api_error = if let Ok(name) = name_result {
                        name.contains("APIError").unwrap_or(false)
                    } else {
                        false
                    };
                    // Get error type name with proper conversion
                    let type_name = e
                        .get_type(py)
                        .name()
                        .map(|name| name.to_string())
                        .unwrap_or_else(|_| "UnknownError".to_string());

                    let err = if is_api_error {
                        MarketDataError::AlpacaAPIError {
                            py_type: type_name,
                            message: py_err_str,
                        }
                    } else {
                        MarketDataError::PythonExecutionError(py_err_str)
                    };

                    Err(Box::new(err) as Box<dyn std::error::Error>)
                }
            }
        })
    }
}
