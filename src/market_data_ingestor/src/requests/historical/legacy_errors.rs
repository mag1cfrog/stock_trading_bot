use std::fmt;

use polars::error::PolarsError;
#[cfg(feature = "alpaca-python-sdk")]
use pyo3::PyErr;

#[derive(Debug)]
pub enum MarketDataError {
    InvalidPath(String),
    MissingSitePackages(String),
    MissingAlpacaPackage(String),
    NoPythonVersionFound(String),
    AlpacaAPIError { py_type: String, message: String },
    PythonExecutionError(String),
    EnvError(String),
    PyInterfaceError(String),
    DataFrameError(String),
}

#[cfg(feature = "alpaca-python-sdk")]
// Add From implementations for automatic conversions
impl From<PyErr> for MarketDataError {
    fn from(err: PyErr) -> Self {
        Self::PyInterfaceError(err.to_string())
    }
}

impl From<PolarsError> for MarketDataError {
    fn from(err: PolarsError) -> Self {
        Self::DataFrameError(err.to_string())
    }
}

impl fmt::Display for MarketDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(path) => write!(f, "Invalid path: {path}"),
            Self::MissingSitePackages(path) => {
                write!(f, "Missing site-packages directory: {path}")
            }
            Self::MissingAlpacaPackage(path) => write!(f, "Missing Alpaca package: {path}"),
            Self::NoPythonVersionFound(msg) => write!(f, "No Python version found: {msg}"),
            Self::AlpacaAPIError { py_type, message } => {
                write!(f, "Alpaca API error({py_type}): {message}")
            }
            Self::PythonExecutionError(msg) => write!(f, "Python execution error: {msg}"),
            Self::EnvError(msg) => write!(f, "Environment error: {msg}"),
            Self::PyInterfaceError(msg) => write!(f, "Python interface error: {msg}"),
            Self::DataFrameError(msg) => write!(f, "DataFrame processing error: {msg}"),
        }
    }
}

impl std::error::Error for MarketDataError {}
