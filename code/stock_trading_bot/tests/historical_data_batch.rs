use std::path::Path;
use chrono::{TimeZone, Utc};
use stock_trading_bot::models::stockbars::StockBarsParams;
use stock_trading_bot::models::timeframe::TimeFrame;
use stock_trading_bot::requests::historical::StockBarData;

#[tokio::test]
async fn test_batch_historical_data_fetch() {
    let market_data = StockBarData::new(Path::new("python/venv"))
        .await
        .expect("Can't initialize the data fetcher");

    // Create multiple parameter sets
    let params_list = vec![
        StockBarsParams {
            symbols: vec!["AAPL".into()],
            timeframe: TimeFrame::day().unwrap(),
            start: Utc.with_ymd_and_hms(2023, 1, 1, 9, 30, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2023, 1, 30, 16, 0, 0).unwrap(),
        },
        StockBarsParams {
            symbols: vec!["MSFT".into()],
            timeframe: TimeFrame::day().unwrap(),
            start: Utc.with_ymd_and_hms(2023, 1, 1, 9, 30, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2023, 1, 30, 16, 0, 0).unwrap(),
        }
    ];

    let results = market_data
        .fetch_bars_batch_partial(&params_list, 3, 1000)
        .expect("Failed to execute batch request");

    // Check results
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(df) => println!("Dataframe {} succeeded with shape: {:?}", i, df.shape()),
            Err(e) => println!("Request {} failed with error: {}", i, e),
        }
    }

    // Assert at least one success
    assert!(results.iter().any(|r| r.is_ok()), "At least one request should succeed");
}