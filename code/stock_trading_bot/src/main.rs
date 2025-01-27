use std::path::Path;

use stock_trading_bot::historical::MarketData;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let market_data = MarketData::new(Path::new("python/venv"))
        .await
        .expect("Can't initialize the data fetcher");
    let df = market_data.fetch_historical_bars().expect("Can't get dataframe from py to rs");
    println!("dataframe: {}", df);
    Ok(())
}
