use serde::{Deserialize, Serialize};

/// Represents the class of financial asset for a data request.
///
/// This enum is used in [`BarsRequestParams`](crate::models::request_params::BarsRequestParams)
/// to specify the type of asset being queried (e.g., stocks, futures, etc.).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetClass {
    /// U.S. equities (stocks traded on U.S. exchanges).
    UsEquity,
    /// Exchange-traded futures contracts (e.g., ES, NQ).
    Futures,
    // Add more asset classes (e.g., Crypto, Options) as needed.
}