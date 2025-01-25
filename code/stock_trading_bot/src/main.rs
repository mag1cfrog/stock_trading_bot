use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::ffi::CString;
use std::path::Path;

fn main() -> PyResult<()> {
    
    // Initialize Python first without environment vars
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        // Set up virtual environment paths after Python is initialized
        let venv_path = Path::new("python/venv");
        
        // Add virtual environment's site-packages to Python's path
        let sys = py.import("sys")?;
        let path = sys.getattr("path")?;
        path.call_method1(
            "insert", 
            (0, venv_path.join("lib/python3.12/site-packages"))
        )?;

        let code = r#"
import os
from alpaca.data.historical import StockHistoricalDataClient
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame
from datetime import datetime

# Should use the virtual environment's packages
print("Alpaca version:", StockHistoricalDataClient.__module__)

alpaca_key = os.getenv('APCA_API_KEY_ID')
secret_key = os.getenv('APCA_API_SECRET_KEY')
# Initialize client with environment variables
client = StockHistoricalDataClient(api_key=alpaca_key, secret_key=secret_key)

# Build request
request_params = StockBarsRequest(
    symbol_or_symbols=["AAPL"],
    timeframe=TimeFrame.Day,
    start=datetime(2023, 1, 1),
    end=datetime(2023, 1, 10)
)

bars = client.get_stock_bars(request_params)
df = bars.df
print("Retrieved bars dataframe:")
print(df)
"#;

    let locals = PyDict::new(py);
    let globals = PyDict::new(py);

    // Convert the code string to CString
    let code = CString::new(code).unwrap();
    py.run(&code, Some(&globals), Some(&locals))?;
        Ok(())
    })
}