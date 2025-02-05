#![doc(hidden)]

/// This dummy function references items from market_data_ingestor so that dead-code warnings are suppressed.
/// It is hidden from documentation.
pub fn init_dummy_usage() {
    // Reference items from market_data_ingestor to mark them as "used"
    let _ = crate::market_data_ingestor::requests::historical::fetch_historical_bars;
    let _ = crate::market_data_ingestor::requests::historical::fetch_bars_batch_partial;
}