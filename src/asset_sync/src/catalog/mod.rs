//! Catalog subsystem.
//!
//! This module groups configuration and normalization utilities for the provider
//! catalog, which describes providers, supported asset classes, and symbol
//! mappings. See [`crate::catalog::config`] for the TOML model and helpers.

mod cache;
pub mod config;
pub mod repo;
pub mod sync;

pub use cache::{clear_allowed_cache, is_allowed_provider_class, refresh_allowed, snapshot};
