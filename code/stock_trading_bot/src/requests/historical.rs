use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::error::Error;
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
    ) -> Result<DataFrame, Box<dyn Error>> {
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
                        name.to_string().to_lowercase().contains("apierror")
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

    pub fn fetch_bars_batch_partial(
        &self,
        params_list: &[StockBarsParams],
        max_retries: u32,
        base_delay_ms: u64
    ) -> Result<Vec<Result<DataFrame, MarketDataError>>, Box<dyn Error>> {

        // Acquire GIL
        Python::with_gil(|py| {
            // Insert site-packages
            let sys = py.import("sys")?;
            let py_path = sys.getattr("path")?;
            py_path.call_method1("insert", (0, &self.site_packages_path))?;

            // Convert params to Python list
            let py_list = PyList::empty(py);
            for param in params_list {
                let py_request = param.clone().into_pyobject(py)?; 
                py_list.append(py_request)?;
            }

            // Python code that attempts each request with internal retry.
            //
            // We'll store results in `results_list`, which will be a list of dicts:
            //   [ {"ok": <IPC bytes>}, {"err": "some error msg"}, ... ]
            // same length as request_params_list.
            // That way we can do partial success in one pass.
            //
            // We do a simple "while attempt < max_retries" approach + some (fixed) time.sleep.
            // You could do exponential backoff or anything else you want in Python here.
            let code = r#"
import os, time
from alpaca.data.historical import StockHistoricalDataClient
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame
import polars as pl

max_r = int(max_retries)
base_delay = float(base_delay_ms) / 1000.0  # convert ms -> seconds

alpaca_key = os.getenv('APCA_API_KEY_ID')
secret_key = os.getenv('APCA_API_SECRET_KEY')
client = StockHistoricalDataClient(api_key=alpaca_key, secret_key=secret_key)

results_list = []

for request_params in request_params_list:
    attempt = 0
    success = False
    last_exception = None
    while attempt < max_r:
        try:
            bars = client.get_stock_bars(request_params)
            df = bars.df
            pl_df = pl.from_pandas(df)
            arrow_data = pl_df.write_ipc(
                file=None,
                compression='uncompressed',
                compat_level=pl.CompatLevel.newest()
            ).getvalue()
            results_list.append({"ok": arrow_data})
            success = True
            break
        except Exception as e:
            # here you can check type of e to decide if it's retryable
            # e.g. if "rate limit", "503", etc.
            # We'll do a crude check
            msg = str(e).lower()
            if ("internal" in msg or "5xx" in msg or "503" in msg or "timeout" in msg or "rate limit" in msg):
                # retryable
                time.sleep(base_delay * (2 ** attempt))  # exponential backoff
                attempt += 1
                last_exception = e
            else:
                # not retryable
                last_exception = e
                break

    if not success:
        # We used up attempts or we had a non-retryable error
        if last_exception is not None:
            results_list.append({"err": str(last_exception)})
        else:
            results_list.append({"err": "Unknown error"})
"#;

            let locals = PyDict::new(py);
            locals.set_item("request_params_list", py_list)?;
            locals.set_item("max_retries", max_retries)?;
            locals.set_item("base_delay_ms", base_delay_ms)?;

            // Run the Python snippet

            // Convert the code string to CString
            let code = CString::new(code).unwrap();
            match py.run(&code, None, Some(&locals)) {
                Ok(_) => {
                    // Extract results_list from locals
                    let py_results_list = locals
                        .get_item("results_list")
                        .map_err(|e| MarketDataError::PythonExecutionError(
                            format!("Could not find `results_list` in Python locals: {}", e)
                        ))?
                        .ok_or_else(|| MarketDataError::PythonExecutionError(
                            "Python variable `results_list` was not set".into()
                        ))?; // Convert Option to Result;
                    
                    let py_results_list: &Bound<'_, PyList> = py_results_list
                        .downcast()
                        .map_err(|e| MarketDataError::PythonExecutionError(
                            format!("results_list is not a Python list: {}", e)
                        ))?;

                    // We'll parse each entry in results_list into a Rust `Result<DataFrame, MarketDataError>`.
                    let mut out = Vec::with_capacity(params_list.len());
                    for item_result in py_results_list.try_iter()? {
                        let item = item_result?;
                        // item should be a Python dict with either "ok" -> bytes or "err" -> string
                        // With proper dict downcasting:
                        let dict = item.downcast::<PyDict>().map_err(|e| {
                            MarketDataError::PythonExecutionError(format!("Expected dict: {}", e))
                        })?;

                        let keys = dict.keys();
                        if keys.contains("ok")? {
                            // success
                            let ipc_bytes: Vec<u8> = item.get_item("ok").unwrap().extract()?;
                            let df = IpcReader::new(std::io::Cursor::new(ipc_bytes)).finish()?;
                            out.push(Ok(df));
                        } else if keys.contains("err")? {
                            let err_str: String = item.get_item("err").unwrap().extract()?;
                            // Decide if we want to treat it as AlpacaAPIError or PythonExecutionError, etc.
                            // For demonstration, let's assume it's an Alpaca error if it says 'api' in it:
                            let lower = err_str.to_lowercase();
                            if lower.contains("api") {
                                out.push(Err(MarketDataError::AlpacaAPIError {
                                    py_type: "APIError".to_string(),
                                    message: err_str,
                                }));
                            } else {
                                out.push(Err(MarketDataError::PythonExecutionError(err_str)));
                            }
                        } else {
                            out.push(Err(MarketDataError::PythonExecutionError(
                                "Malformed result: no 'ok' or 'err'".to_string(),
                            )));
                        }
                    }

                    Ok(out)
                },
                Err(e) => {
                    // Means the entire snippet crashed unexpectedly (like missing package, bad Python, etc.)
                    let py_err_str = e.to_string();
                    let name_result = e.get_type(py).name();
                    let type_name = name_result
                        .unwrap();

                    let is_api_error = type_name.contains("APIError")?
                        || py_err_str.to_lowercase().contains("apierror");

                    let err = if is_api_error {
                        MarketDataError::AlpacaAPIError {
                            py_type: type_name.to_string(),
                            message: py_err_str,
                        }
                    } else {
                        MarketDataError::PythonExecutionError(py_err_str)
                    };
                    Err(Box::new(err).into())
                }
            }
        })
    }
}
