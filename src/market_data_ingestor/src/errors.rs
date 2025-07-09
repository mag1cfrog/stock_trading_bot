use thiserror::Error;

/// The unified error type for the `market_data_ingestor` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// An error originating from a data provider (e.g., API error, validation).
    #[error("Provider error: {0}")]
    Provider(String),

    /// An error originating from a data sink (e.g., file I/O, database write).
    #[error("Sink error: {0}")]
    Sink(String),

    /// An error related to configuration.
    #[error("Configuration error: {0}")]
    Config(String),

    /// A generic I/O error.
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    /// An error from the Polars library.
    #[error("Polars operation failed")]
    Polars(#[from] polars::prelude::PolarsError),
}