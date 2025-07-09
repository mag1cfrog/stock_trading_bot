use thiserror::Error;

/// Errors that can occur within a `DataProvider` implementation.
#[derive(Debug, Error)]
pub enum ProviderError {
    /// An error during an API request (e.g., network failure, timeout).
    #[error("API request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// The provider's API returned a specific error message (e.g., invalid API key).
    #[error("API error: {0}")]
    Api(String),

    /// The request parameters were invalid for this specific provider.
    #[error("Invalid parameters for provider: {0}")]
    Validation(String),

    /// An internal error occurred while processing data within the provider.
    #[error("Internal provider error: {0}")]
    Internal(String),
}