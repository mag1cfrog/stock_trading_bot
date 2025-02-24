use std::error::Error;
use std::ffi::CString;

use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::models::stockbars::StockBarsParams;
use crate::requests::historical::StockBarData;
use crate::requests::historical::errors::MarketDataError;


pub fn fetch_bars_batch_partial(
    _data: &StockBarData,
    params_list: &[StockBarsParams],
    max_retries: u32,
    base_delay_ms: u64
) -> Result<Vec<Result<DataFrame, MarketDataError>>, Box<dyn Error>> {

    // Acquire GIL
    Python::with_gil(|py| {
        // Insert site-packages
        // let sys = py.import("sys")?;
        // let py_path = sys.getattr("path")?;
        // py_path.call_method1("insert", (0, &data.site_packages_path))?;

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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serial_test::serial;
    use crate::models::stockbars::StockBarsParams;
    use crate::models::timeframe::TimeFrame;
    use crate::requests::historical::StockBarData;

    #[tokio::test]
    #[serial]
    async fn test_batch_historical_data_fetch() {
        let market_data = StockBarData::new("/home/hanbo/repo/stock_trading_bot/src/configs/data_ingestor.toml")
            .await
            .expect("Can't initialize the data fetcher");

        // Create multiple parameter sets
        let params_list = [
            StockBarsParams {
                symbols: vec!["AAPL".into()],
                timeframe: TimeFrame::day().unwrap(),
                start: Utc.with_ymd_and_hms(2023, 1, 1, 9, 30, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2023, 1, 30, 16, 0, 0).unwrap(),
            },
            StockBarsParams {
                symbols: vec!["MSFT".into()],
                timeframe: TimeFrame::day().unwrap(),
                start: Utc.with_ymd_and_hms(2023, 1, 1, 9, 30, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2023, 1, 30, 16, 0, 0).unwrap(),
            }
        ];

        let results = market_data
            .fetch_bars_batch_partial(&params_list, 3, 1000)
            .expect("Failed to execute batch request");

        // Check results
        for (i, result) in results.iter().enumerate() {
            match result {
                Ok(df) => println!("Dataframe {} succeeded with shape: {:?}", i, df.shape()),
                Err(e) => println!("Request {} failed with error: {}", i, e),
            }
        }

        // Assert at least one success
        assert!(results.iter().any(|r| r.is_ok()), "At least one request should succeed");

}
}