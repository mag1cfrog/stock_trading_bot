# Market Data Ingestor

A pure Rust library for fetching and processing market data from various providers. This is an internal component of a stock trading system.

## Overview

The `market_data_ingestor` is a core component of the stock trading system that provides:

- **Multi-provider support**: Abstract interface for different market data vendors (currently supports Alpaca)
- **Async-first design**: Built with `tokio` for high-performance async operations
- **Pure Rust implementation**: Efficient Rust-native provider implementations
- **Provider abstraction**: Unified interface for switching between different data sources
- **Rate limiting**: Built-in rate limiting to respect API constraints
- **Error handling**: Comprehensive error handling with detailed error types

## Usage

### Rust-Native Provider (Current Implementation)

```rust
use market_data_ingestor::{
    providers::alpaca_rest::AlpacaRestProvider,
    models::request_params::BarsRequestParams,
    models::timeframe::Timeframe,
    providers::DataProvider,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Alpaca provider
    let provider = AlpacaRestProvider::new().await?;
    
    // Create request parameters
    let params = BarsRequestParams {
        symbols: vec!["AAPL".to_string(), "MSFT".to_string()],
        timeframe: Timeframe::OneDay,
        start: "2024-01-01T00:00:00Z".parse()?,
        end: "2024-01-31T23:59:59Z".parse()?,
    };
    
    // Fetch bar data
    let bars = provider.fetch_bars(params).await?;
    
    println!("Fetched {} bar series", bars.len());
    Ok(())
}
```

## Configuration

Configuration is handled through environment variables:

```bash
export ALPACA_API_KEY="your_api_key_here"
export ALPACA_SECRET_KEY="your_secret_key_here"
export ALPACA_BASE_URL="https://paper-api.alpaca.markets"  # or live API URL
```

## Architecture

### Core Components

- **`providers/`**: Abstract provider interface and implementations
  - `alpaca_rest/`: Rust-native Alpaca Markets REST API implementation
- **`models/`**: Data models for bars, assets, timeframes, and request parameters
- **`requests/`**: Request handling infrastructure
- **`io/`**: I/O operations and sinks
- **`utils/`**: Utility functions
- **`errors/`**: Error types and handling

### Provider System

The library uses a trait-based system for supporting multiple data providers:

```rust
#[async_trait]
pub trait DataProvider {
    async fn fetch_bars(&self, params: BarsRequestParams) -> Result<Vec<BarSeries>, ProviderError>;
}
```

This allows easy extension to support additional market data vendors while maintaining a consistent interface.

## Error Handling

The library provides comprehensive error handling with specific error types:

- **`ProviderError`**: Errors from data provider operations
- **`ProviderInitError`**: Errors during provider initialization
- **`SinkError`**: Errors during data sink operations

## Dependencies

- `tokio`: Async runtime
- `reqwest`: HTTP client for API requests
- `serde`: Serialization/deserialization
- `chrono`: Date and time handling
- `async-trait`: Async trait support
- `governor`: Rate limiting
- `thiserror`: Error handling
- `secrecy`: Secure handling of sensitive data

## Version

Current version: **1.3.5**
