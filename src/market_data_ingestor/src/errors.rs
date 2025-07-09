use thiserror::Error;

use crate::{io::sink::SinkError, providers::ProviderError};

/// The unified error type for the `market_data_ingestor` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// An error originating from a data provider (e.g., API error, validation).
    #[error(transparent)]
    Provider(#[from] ProviderError),

    /// An error originating from a data sink (e.g., file I/O, database write).
    #[error(transparent)]
    Sink(#[from] SinkError),

    /// An error related to configuration.
    #[error("Configuration error: {0}")]
    Config(String),
}