use pyo3::{
    types::{PyAnyMethods, PyDict},
    Bound, IntoPyObject, PyAny, PyErr,
};

use crate::models::timeframe::TimeFrame;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct StockBarsParams {
    pub symbols: Vec<String>,
    pub timeframe: TimeFrame,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl<'py> IntoPyObject<'py> for StockBarsParams {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: pyo3::Python<'py>) -> Result<Self::Output, Self::Error> {
        let request_cls = py
            .import("alpaca.data.requests")?
            .getattr("StockBarsRequest")?;

        // Convert timeframe to Python
        let py_timeframe = self.timeframe.into_pyobject(py)?;

        // Build kwargs
        let kwargs = PyDict::new(py);
        // "symbol_or_symbols" is a required named arg for StockBarsRequest
        kwargs.set_item("symbol_or_symbols", self.symbols)?;
        // "timeframe" is also required
        kwargs.set_item("timeframe", py_timeframe)?;
        // The base classes also define "start" and "end" as optional, but we have them, so
        kwargs.set_item("start", self.start)?;
        kwargs.set_item("end", self.end)?;

        // Any additional fields (like limit, feed, etc.) can be set here if you want:
        // kwargs.set_item("limit", 1000)?;

        // Call StockBarsRequest(...) with **kwargs
        request_cls.call((), Some(&kwargs))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::models::timeframe::TimeFrame;
    use chrono::TimeZone;
    use pyo3::Python;

    fn init_python() {
        // Initialize Python with venv
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let venv_path = Path::new("python/venv");
            let sys = py.import("sys").unwrap();
            let path = sys.getattr("path").unwrap();
            path.call_method1(
                "insert",
                (0, venv_path.join("lib/python3.12/site-packages")),
            )
            .unwrap();
        });
    }

    #[test]
    fn test_stockbars_params_to_python() {
        init_python();
        Python::with_gil(|py| {
            // Prevent deadlock by importing modules upfront
            py.import("alpaca.data.timeframe").unwrap();
            py.import("alpaca.data.requests").unwrap();

            let params = StockBarsParams {
                symbols: vec!["AAPL".to_string(), "MSFT".to_string()],
                timeframe: TimeFrame::minutes(5).unwrap(),
                start: Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap(),
            };

            let py_request = params.into_pyobject(py).unwrap();

            // Verify the Python object properties
            assert_eq!(
                py_request
                    .getattr("symbol_or_symbols")
                    .unwrap()
                    .extract::<Vec<String>>()
                    .unwrap(),
                vec!["AAPL".to_string(), "MSFT".to_string()]
            );

            // Verify timeframe conversion
            let timeframe = py_request.getattr("timeframe").unwrap();
            assert_eq!(
                timeframe
                    .getattr("amount_value")
                    .unwrap()
                    .extract::<u32>()
                    .unwrap(),
                5
            );

            // Verify dates
            let start = py_request.getattr("start").unwrap();
            let end = py_request.getattr("end").unwrap();
            assert!(start.call_method0("__str__").is_ok());
            assert!(end.call_method0("__str__").is_ok());
        });
    }

    #[test]
    fn test_stockbars_params_creation() {
        let params = StockBarsParams {
            symbols: vec!["AAPL".to_string()],
            timeframe: TimeFrame::minutes(1).unwrap(),
            start: Utc::now(),
            end: Utc::now(),
        };

        assert_eq!(params.symbols.len(), 1);
        assert_eq!(params.symbols[0], "AAPL");
    }
}
