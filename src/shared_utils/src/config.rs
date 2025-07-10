use thiserror::Error;

/// Errors related to application configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// An environment variable required by the application is not set.
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
}

/// Reads an environment variable, returning a structured error if it's missing.
///
/// This is a thin wrapper around `std::env::var` that provides a more
/// ergonomic and specific error type for missing variables.
///
/// # Arguments
/// * `name` - The name of the environment variable to read.
pub fn get_env_var(name: &str) -> Result<String, ConfigError> {
    std::env::var(name).map_err(|_| ConfigError::MissingEnvVar(name.to_string()))
}