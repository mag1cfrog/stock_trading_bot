//! Provider registry that help runtime to map ProviderId to concrete providers
use market_data_ingestor::providers::{alpaca_rest::provider::AlpacaProvider, DataProvider, ProviderInitError};

use crate::spec::ProviderId;

/// Build and return a boxed data provider corresponding to the supplied ProviderId.
pub fn build_provider(id: ProviderId) -> Result<Box<dyn DataProvider + Send + Sync>, ProviderInitError> {
    match id {
        ProviderId::Alpaca => {
            let p = AlpacaProvider::new()?;
            Ok(Box::new(p))
        }
    }
}