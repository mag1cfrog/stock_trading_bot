use snafu::prelude::*;

/// An environment variable required by the application is not set.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub struct MissingEnvVarError {
    pub name: String,
    source: std::env::VarError,
    backtrace: snafu::Backtrace,
}

/// Reads an environment variable, returning a structured error if it's missing.
///
/// This is a thin wrapper around `std::env::var` that provides a more
/// ergonomic and specific error type for missing variables.
///
/// # Arguments
/// * `name` - The name of the environment variable to read.
pub fn get_env_var(name: &str) -> Result<String, MissingEnvVarError> {
    std::env::var(name).context(MissingEnvVarSnafu {
        name: name.to_string(),
    })
}
