use std::path::Path;
use chrono::{TimeZone, Utc};
use stock_trading_bot::models::stockbars::StockBarsParams;
use stock_trading_bot::models::timeframe::TimeFrame;
use stock_trading_bot::requests::historical::StockBarData;

#[tokio::test]
async fn test_historical_data_fetch() {
    let market_data = StockBarData::new(Path::new("python/venv"))
        .await
        .expect("Can't initialize the data fetcher");

    let params = StockBarsParams {
        symbols: vec!["AAPL".into()],
        timeframe: TimeFrame::day().unwrap(),
        start: Utc.with_ymd_and_hms(2025, 1, 1, 9, 30, 0).unwrap(),
        end: Utc.with_ymd_and_hms(2025, 1, 30, 16, 0, 0).unwrap(),
    };

    let df = market_data
        .fetch_historical_bars(params)
        .expect("Can't get dataframe from py to rs");
    println!("Test dataframe output: {}", df);
}