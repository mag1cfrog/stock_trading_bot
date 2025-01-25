use std::path::Path;

use tokio;

use stock_trading_bot::market_data::MarketData;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let market_data = MarketData::new(Path::new("python/venv"))
        .await
        .expect("Can't initialize the data fetcher");
    market_data.fetch_historical_bars()?;
    Ok(())
}
