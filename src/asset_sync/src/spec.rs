//! Declarative specification of *what data to keep fresh*.
//!
use serde::{Deserialize, Serialize};


/// Which upstream to use (serde snake_case).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    /// Alpaca trading API provider.
    Alpaca,
}