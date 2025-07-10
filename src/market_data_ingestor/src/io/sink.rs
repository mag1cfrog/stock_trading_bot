use async_trait::async_trait;
use thiserror::Error;

use crate::models::bar_series::BarSeries;

#[derive(Debug, Error)]
pub enum SinkError {
    /// An error occurred while trying to write the data (e.g., file I/O error).
    #[error("Failed to write data: {0}")]
    WriteError(String),

    /// An error occurred while converting the canonical `BarSeries` model into the destination format (e.g. converting to a DataFrame).
    #[error("Data conversion error: {0}")]
    ConversionError(String),

    /// A generic I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
