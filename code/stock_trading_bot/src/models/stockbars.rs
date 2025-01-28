use chrono::{DateTime, Utc};
use pyo3::{types::PyAnyMethods, Bound, IntoPyObject, PyAny, PyErr};

use crate::models::timeframe::TimeFrame;

pub struct StockBarsParams {
    pub symbols: Vec<String>,
    pub timeframe: TimeFrame,
    pub start:DateTime<Utc>,
    pub end:DateTime<Utc>,
}

impl<'py> IntoPyObject<'py> for StockBarsParams {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: pyo3::Python<'py>) -> Result<Self::Output, Self::Error> {
        let request_cls = py
            .import("alpaca.data.requests")?
            .getattr("STockBarsRequest")?;

        // Convert timeframe to Python
        let py_timeframe = self.timeframe.into_pyobject(py)?;

        request_cls.call1((
            self.symbols,
            py_timeframe,
            self.start,
            self.end,
        ))
    }
}