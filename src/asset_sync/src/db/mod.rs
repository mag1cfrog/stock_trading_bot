//! Database utilities for connections and schema migrations.
//!
//! This module provides:
//! - SQLite connection helpers: [`connection::connect_sqlite`] applies WAL, foreign_keys=ON, and a 5000ms busy_timeout.
//! - Embedded Diesel migrations and runners: [`migrate::run_sqlite`], [`migrate::run_postgres`], and [`migrate::run_all`]
//!   which dispatches based on the URL (postgres://, postgresql://, or a SQLite path).
//!
//! Example:
//! ```no_run
//! use asset_sync::db::{migrate, connection};
//!
//! // Run embedded migrations (treats non-postgres URLs as SQLite, supports bare file paths)
//! let db_path = std::env::temp_dir().join("asset_sync_example.db");
//! migrate::run_all(db_path.to_str().unwrap()).expect("migrations");
//!
//! // Open a tuned SQLite connection
//! let _conn = connection::connect_sqlite(db_path.to_str().unwrap()).expect("connect");
//! ```
//!
//! Note: Building with PostgreSQL support requires the system libpq (e.g., libpq-dev on Debian/Ubuntu).

pub mod connection;
pub mod migrate;
