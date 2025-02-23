use std::error::Error;
use std::ffi::CString;

use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::models::stockbars::StockBarsParams;
use crate::requests::historical::StockBarData;
use crate::requests::historical::errors::MarketDataError;

pub fn fetch_historical_bars(
    data: &StockBarData,
    params: StockBarsParams,
) -> Result<DataFrame, Box<dyn Error>> {
    // Initialize Python first without environment vars
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        // Set up virtual environment paths after Python is initialized

        // Add virtual environment's site-packages to Python's path
        let sys = py.import("sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("insert", (0, &data.site_packages_path))?;

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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serial_test::serial;
    use crate::models::stockbars::StockBarsParams;
    use crate::models::timeframe::TimeFrame;
    use std::path::Path;
    use crate::requests::historical::StockBarData;

    #[tokio::test]
    #[serial]
    async fn test_historical_data_fetch() {
        let market_data = StockBarData::new(Path::new("python/venv"))
            .await
            .expect("Can't initialize the data fetcher");

        let params = StockBarsParams {
            symbols: vec!["AAPL".into()],
            timeframe: TimeFrame::day().unwrap(),
            start: Utc.with_ymd_and_hms(2025, 1, 1, 9, 30, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2025, 1, 30, 16, 0, 0).unwrap(),
        };

        let df = market_data
            .fetch_historical_bars(params)
            .expect("Can't get dataframe from py to rs");
        println!("Test dataframe output: {}", df);
    }
}