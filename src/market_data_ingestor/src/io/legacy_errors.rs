use std::fmt;
use std::io;
use std::path::PathBuf;

use polars::error::PolarsError;

#[derive(Debug)]
pub enum IOError {
    FileCreationError { path: PathBuf, error: String },
    DirectoryCreationError { path: PathBuf, error: String },
    FileWriteError { path: PathBuf, error: String },
    InvalidSymbol(String),
    DataFrameError(String),
}

impl From<io::Error> for IOError {
    fn from(err: io::Error) -> Self {
        // Default path handling without context
        Self::FileWriteError {
            path: PathBuf::from("<unknown>"),
            error: err.to_string(),
        }
    }
}

impl From<PolarsError> for IOError {
    fn from(err: PolarsError) -> Self {
        Self::DataFrameError(err.to_string())
    }
}

impl fmt::Display for IOError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileCreationError { path, error } => {
                write!(f, "Failed to create file at {}: {}", path.display(), error)
            }
            Self::DirectoryCreationError { path, error } => write!(
                f,
                "Failed to create directory at {}: {}",
                path.display(),
                error
            ),
            Self::FileWriteError { path, error } => write!(
                f,
                "Failed to write to file at {}: {}",
                path.display(),
                error
            ),
            Self::InvalidSymbol(msg) => write!(f, "Invalid symbol: {msg}",),
            Self::DataFrameError(msg) => write!(f, "DataFrame processing error: {msg}",),
        }
    }
}

impl std::error::Error for IOError {}
