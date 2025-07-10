#![cfg(test)]
use chrono::{Duration, Utc};
use market_data_ingestor::{
    models::{
        asset::AssetClass,
        request_params::{BarsRequestParams, ProviderParams},
        timeframe::{TimeFrame, TimeFrameUnit},
    },
    providers::{
        alpaca_rest::{params::{AlpacaBarsParams, Sort}, provider::AlpacaProvider},
        DataProvider,
    },
};
use serial_test::serial;

#[tokio::test]
#[serial]
#[ignore]
async fn test_alpaca_provider_fetch_bars() {
    // This test requires APCA_API_KEY_ID and APCA_API_SECRET_KEY to be set in the environment.
    if std::env::var("APCA_API_KEY_ID").is_err() || std::env::var("APCA_API_SECRET_KEY").is_err() {
        println!("Skipping test_alpaca_provider_fetch_bars: API keys not set.");
        return;
    }

    let provider = AlpacaProvider::new().expect("Failed to create AlpacaProvider");

    let params = BarsRequestParams {
        symbols: vec!["AAPL".to_string()],
        timeframe: TimeFrame::new(1, TimeFrameUnit::Day),
        start: Utc::now() - Duration::days(10),
        end: Utc::now() - Duration::days(1),
        asset_class: AssetClass::UsEquity,
        provider_specific: ProviderParams::Alpaca(AlpacaBarsParams {
            sort: Some(Sort::Desc),
            limit: Some(5),
            ..Default::default()
        }),
    };

    let result = provider.fetch_bars(params).await;

    assert!(result.is_ok(), "fetch_bars returned an error: {:?}", result.err());

    let bar_series_vec = result.unwrap();
    assert_eq!(bar_series_vec.len(), 1, "Expected 1 BarSeries for AAPL");

    let aapl_series = &bar_series_vec[0];
    assert_eq!(aapl_series.symbol, "AAPL");
    assert!(!aapl_series.bars.is_empty(), "Expected to fetch at least one bar for AAPL");
    assert!(aapl_series.bars.len() <= 5, "Expected at most 5 bars due to limit");

    // Check that bars are sorted descending
    if aapl_series.bars.len() > 1 {
        assert!(aapl_series.bars[0].timestamp > aapl_series.bars[1].timestamp);
    }
}