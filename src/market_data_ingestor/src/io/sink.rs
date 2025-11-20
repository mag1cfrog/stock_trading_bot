use async_trait::async_trait;
use snafu::{Backtrace, Snafu};

use crate::models::bar::BarSeries;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum SinkError {
    /// An error occurred while trying to write the data (e.g., file I/O error).
    #[snafu(display("Failed to write data: {message}"))]
    WriteError {
        message: String,
        backtrace: Backtrace,
    },

    /// An error occurred while converting the canonical `BarSeries` model into the destination format (e.g. converting to a DataFrame).
    #[snafu(display("Data conversion error: {message}"))]
    ConversionError {
        message: String,
        backtrace: Backtrace,
    },

    /// A generic I/O error.
    #[snafu(display("I/O error: {source}"))]
    Io {
        source: std::io::Error,
        backtrace: Backtrace,
    },
}

#[async_trait]
pub trait DataSink {
    /// The type of output returned after a successful write operation.
    ///
    /// This makes the trait flexible. For example:
    /// - A file sink might return `Vec<PathBuf>`, the paths to the created files.
    /// - A database sink might return `usize`, the number of rows inserted.
    type Output;

    /// Writes a slice of `BarSeries` to the destination.
    ///
    /// # Arguments
    /// * `data` - A slice of `BarSeries` to be written.
    async fn write(&self, data: &[BarSeries]) -> Result<Self::Output, SinkError>;
}
