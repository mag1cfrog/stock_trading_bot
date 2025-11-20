use snafu::{Backtrace, Snafu};

use crate::{io::sink::SinkError, providers::ProviderError};

/// The unified error type for the `market_data_ingestor` crate.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// An error originating from a data provider (e.g., API error, validation).
    #[snafu(display("Provider error: {source}"))]
    Provider {
        #[snafu(backtrace)]
        source: ProviderError,
    },

    /// An error originating from a data sink (e.g., file I/O, database write).
    #[snafu(display("Sink error: {source}"))]
    Sink {
        #[snafu(backtrace)]
        source: SinkError,
    },

    /// An error related to configuration.
    #[snafu(display("Configuration error: {message}"))]
    Config {
        message: String,
        backtrace: Backtrace,
    },
}
