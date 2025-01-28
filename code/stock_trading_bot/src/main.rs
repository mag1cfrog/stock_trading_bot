use std::path::Path;

use chrono::{Utc, TimeZone};

use stock_trading_bot::requests::historical::StockBarData;
use stock_trading_bot::models::stockbars::StockBarsParams;
use stock_trading_bot::models::timeframe::TimeFrame;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let market_data = StockBarData::new(Path::new("python/venv"))
        .await
        .expect("Can't initialize the data fetcher");
    
    let params = StockBarsParams {
        symbols: vec!["AAPL".into()],
        timeframe: TimeFrame::day().unwrap(),
        start: Utc.with_ymd_and_hms(2025, 1, 1, 9, 30, 0).unwrap(),
        end: Utc.with_ymd_and_hms(2025, 1, 30, 16, 0, 0).unwrap(),
    };

    let df = market_data.fetch_historical_bars(params).expect("Can't get dataframe from py to rs");
    println!("dataframe: {}", df);
    Ok(())
}
