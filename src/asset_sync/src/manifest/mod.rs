//! Manifest + coverage + gaps repository (SQLite).

#[derive(thiserror::Error, Debug)]
/// Errors that can occur while interacting with the manifest repository.
pub enum RepoError {
    #[error("coverage version conflict (expected {expected})")]
    /// Raised when the coverage version does not match the expected value.
    CoverageConflict {
        /// The expected coverage version.
        expected: i32,
    },
}
