use crate::{io::legacy_errors::IOError, requests::historical::MarketDataError};
use std::fmt;

#[derive(Debug)]
pub enum IngestorError {
    Market(MarketDataError),
    IO(IOError),
    SystemError(String),
}

impl From<MarketDataError> for IngestorError {
    fn from(err: MarketDataError) -> Self {
        Self::Market(err)
    }
}

impl From<IOError> for IngestorError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

impl fmt::Display for IngestorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Market(err) => write!(f, "Market data error: {}", err),
            Self::IO(err) => write!(f, "I/O error: {}", err),
            Self::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for IngestorError {}
