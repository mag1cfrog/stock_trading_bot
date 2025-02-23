use std::fmt;

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