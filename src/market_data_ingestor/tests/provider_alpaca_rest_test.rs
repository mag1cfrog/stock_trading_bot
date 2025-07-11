#![cfg(all(test, feature = "alpaca-python-sdk"))]
use chrono::{DateTime, Duration, Utc};
use market_data_ingestor::{
    models::{
        asset::AssetClass,
        bar::Bar,
        bar_series::BarSeries,
        request_params::{BarsRequestParams, ProviderParams},
        stockbars::StockBarsParams as LegacyParams,
        timeframe::{TimeFrame, TimeFrameUnit},
    },
    providers::{
        alpaca_rest::{
            params::{AlpacaBarsParams, Sort},
            provider::AlpacaProvider,
        },
        DataProvider,
    },
    requests::historical::StockBarData,
};
use polars::prelude::*;
use serial_test::serial;
use std::io::Write;
use tempfile::NamedTempFile;
use std::{collections::HashMap, path::Path};

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

#[tokio::test]
#[serial]
#[ignore]
async fn test_alpaca_provider_pagination() {
    // This test requires APCA_API_KEY_ID and APCA_API_SECRET_KEY to be set in the environment.
    if std::env::var("APCA_API_KEY_ID").is_err() || std::env::var("APCA_API_SECRET_KEY").is_err() {
        println!("Skipping test_alpaca_provider_pagination: API keys not set.");
        return;
    }

    let provider = AlpacaProvider::new().expect("Failed to create AlpacaProvider");
    const PAGE_LIMIT: u32 = 200;

    let params = BarsRequestParams {
        symbols: vec!["SPY".to_string()], // Use a liquid asset
        timeframe: TimeFrame::new(1, TimeFrameUnit::Minute),
        start: Utc::now() - Duration::days(7), // Request a long period
        end: Utc::now() - Duration::days(1),
        asset_class: AssetClass::UsEquity,
        provider_specific: ProviderParams::Alpaca(AlpacaBarsParams {
            limit: Some(PAGE_LIMIT), // Set a small limit to force pagination
            ..Default::default()
        }),
    };

    let result = provider.fetch_bars(params).await;

    assert!(result.is_ok(), "fetch_bars returned an error: {:?}", result.err());

    let bar_series_vec = result.unwrap();
    assert_eq!(bar_series_vec.len(), 1, "Expected 1 BarSeries for SPY");

    let spy_series = &bar_series_vec[0];
    assert!(!spy_series.bars.is_empty(), "Expected to fetch bars for SPY");

    // The key assertion: if we got more bars than the page limit, pagination must have worked.
    assert!(
        spy_series.bars.len() > PAGE_LIMIT as usize,
        "Expected more bars than the page limit ({}), but got {}. Pagination may have failed.",
        PAGE_LIMIT,
        spy_series.bars.len()
    );
}

// Helper function to convert a DataFrame from the legacy client to Vec<BarSeries>
fn dataframe_to_bar_series(
    df: &DataFrame,
    timeframe: TimeFrame,
) -> Result<Vec<BarSeries>, Box<dyn std::error::Error>> {
    println!("DataFrame columns: {:?}", df.get_column_names());
    println!("DataFrame shape: {:?}", df.shape());
    
    let mut series_map: HashMap<String, Vec<Bar>> = HashMap::new();

    // After reset_index(), we should have both timestamp and symbol as columns
    let timestamp_col = df.column("timestamp")?.datetime()?;
    let symbol_col = df.column("symbol")?.str()?;
    let open_col = df.column("open")?.f64()?;
    let high_col = df.column("high")?.f64()?;
    let low_col = df.column("low")?.f64()?;
    let close_col = df.column("close")?.f64()?;
    let volume_col = df.column("volume")?.f64()?;
    
    // Handle trade_count - it might be f64 from Python, so we need to convert
    let trade_count_col = df.column("trade_count")?;
    let trade_count_values: Vec<Option<u64>> = if trade_count_col.dtype() == &DataType::Float64 {
        // Convert f64 to u64
        let f64_col = trade_count_col.f64()?;
        (0..df.height())
            .map(|i| f64_col.get(i).map(|v| v as u64))
            .collect()
    } else {
        // Try to extract as u64 directly
        let u64_col = trade_count_col.u64()?;
        (0..df.height())
            .map(|i| u64_col.get(i))
            .collect()
    };

    let vwap_col = df.column("vwap")?.f64()?;

    for i in 0..df.height() {
        let symbol = symbol_col.get(i).unwrap().to_string();
        let bar = Bar {
            timestamp: DateTime::from_timestamp_nanos(timestamp_col.get(i).unwrap()),
            open: open_col.get(i).unwrap(),
            high: high_col.get(i).unwrap(),
            low: low_col.get(i).unwrap(),
            close: close_col.get(i).unwrap(),
            volume: volume_col.get(i).unwrap(),
            trade_count: trade_count_values[i],
            vwap: vwap_col.get(i),
        };
        series_map.entry(symbol).or_default().push(bar);
    }

    let mut result: Vec<BarSeries> = series_map
        .into_iter()
        .map(|(symbol, mut bars)| {
            bars.sort_by_key(|b| b.timestamp); // Ensure consistent order
            BarSeries {
                symbol,
                timeframe: timeframe.clone(),
                bars,
            }
        })
        .collect();

    result.sort_by(|a, b| a.symbol.cmp(&b.symbol)); // Ensure consistent order
    Ok(result)
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_compare_rust_and_python_providers() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // This test requires APCA_API_KEY_ID and APCA_API_SECRET_KEY to be set.
    if std::env::var("APCA_API_KEY_ID").is_err() || std::env::var("APCA_API_SECRET_KEY").is_err() {
        println!("Skipping test_compare_rust_and_python_providers: API keys not set.");
        return;
    }

    // --- 1. Define Common Request Parameters ---
    let symbols = vec!["AAPL".to_string(), "MSFT".to_string()];
    let timeframe = TimeFrame::new(1, TimeFrameUnit::Day);
    let start = Utc::now() - Duration::days(20);
    let end = Utc::now() - Duration::days(1);

    // --- 2. Fetch with new Rust-native AlpacaProvider ---
    let rust_provider = AlpacaProvider::new().expect("Failed to create AlpacaProvider");
    let rust_params = BarsRequestParams {
        symbols: symbols.clone(),
        timeframe: timeframe.clone(),
        start,
        end,
        asset_class: AssetClass::UsEquity,
        provider_specific: ProviderParams::Alpaca(AlpacaBarsParams {
            sort: Some(Sort::Asc), // Ensure ascending order for comparison
            ..Default::default()
        }),
    };
    let mut rust_result = rust_provider
        .fetch_bars(rust_params)
        .await
        .expect("Rust provider failed to fetch bars");
    rust_result.sort_by(|a, b| a.symbol.cmp(&b.symbol)); // Ensure consistent order

    // --- 3. Fetch with legacy Python-based StockBarData client ---
    // --- 3. Fetch with legacy Python-based StockBarData client ---
    // Create a temporary config file to avoid relying on a hardcoded path.
    let mut temp_config = NamedTempFile::new().expect("Failed to create temp config file");
    let venv_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../python/.venv");
    let config_content = format!(
        "python_venv_path = \"{}\"",
        venv_path.to_str().unwrap().replace('\\', "\\\\") // Handle Windows paths
    );
    write!(temp_config, "{config_content}", ).expect("Failed to write to temp config file");
    let config_path = temp_config.path().to_str().unwrap();

    let python_client = StockBarData::new(config_path)
        .await
        .expect("Failed to create legacy Python client");
    let python_params = LegacyParams {
        symbols: symbols.clone(),
        timeframe: timeframe.clone(),
        start,
        end,
    };
    let python_df = python_client
        .fetch_historical_bars_to_memory(python_params)
        .expect("Python client failed to fetch bars");

    // --- 4. Convert legacy DataFrame to Vec<BarSeries> for comparison ---
    let python_result =
        dataframe_to_bar_series(&python_df, timeframe).expect("Failed to convert DataFrame");

    // --- 5. Assert that the results are identical ---
    assert_eq!(
        rust_result, python_result,
        "Data from Rust provider does not match data from Python provider"
    );
}