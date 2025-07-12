# Market Data Ingestor

A Rust library for fetching and processing market data from various providers. This is an internal component of a stock trading system.

## Overview

The `market_data_ingestor` is a core component of the stock trading system that provides:

- **Multi-provider support**: Abstract interface for different market data vendors (currently supports Alpaca)
- **Async-first design**: Built with `tokio` for high-performance async operations
- **Rust-native implementation**: New efficient Rust-native provider implementations
- **Provider abstraction**: Unified interface for switching between different data sources
- **Rate limiting**: Built-in rate limiting to respect API constraints
- **Error handling**: Comprehensive error handling with detailed error types

## Architecture Evolution

This module has evolved from a Python SDK integration approach to a pure Rust-native implementation:

- **Legacy approach** (feature-gated): Python SDK integration via PyO3 (being phased out)
- **Current approach**: Rust-native provider implementations for better performance and scalability

## Current State

The module is currently in transition:

- **Active development**: New Rust-native Alpaca REST provider (`providers/alpaca_rest/`)
- **Legacy code**: Python SDK integration and CLI features are feature-gated and being phased out
- **Testing**: Both approaches have been tested and produce identical results
- **Future**: Moving towards pure Rust implementation for better performance and maintainability


### Feature Flags

- `cli`: Enable legacy command-line interface (being phased out)
- `alpaca-python-sdk`: Enable legacy Python SDK integration (being phased out)
- `default`: No features enabled by default - uses the new Rust-native approach

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

### Legacy Python SDK Integration (Feature-Gated)

*Note: This approach is being phased out in favor of the Rust-native implementation.*

```rust
// Only available with "alpaca-python-sdk" feature
use market_data_ingestor::{create_client, models::stockbars::StockBarsParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with config file
    let client = create_client("config.toml").await?;
    
    // Use the client for data fetching
    // ...
    
    Ok(())
}
```

## Configuration

### For Rust-Native Providers

Configuration is handled through environment variables:

```bash
export ALPACA_API_KEY="your_api_key_here"
export ALPACA_SECRET_KEY="your_secret_key_here"
export ALPACA_BASE_URL="https://paper-api.alpaca.markets"  # or live API URL
```

### For Legacy Python SDK Integration

Create a `config.toml` file (only needed when using legacy features):

```toml
[alpaca]
api_key = "your_api_key_here"
secret_key = "your_secret_key_here"
base_url = "https://paper-api.alpaca.markets"  # or live API URL
```

## Architecture

### Core Components

- **`providers/`**: Abstract provider interface and implementations
  - `alpaca_rest/`: New Rust-native Alpaca Markets REST API implementation
- **`models/`**: Data models for bars, assets, timeframes, and request parameters
- **`requests/`**: Legacy request handling (feature-gated)
- **`cli/`**: Legacy command-line interface (feature-gated)
- **`io/`**: Legacy I/O operations (feature-gated)
- **`utils/`**: Utility functions and legacy Python initialization

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
- **`IngestorError`**: General ingestion errors
- **`MarketDataError`**: Market data specific errors

### Migration Status

- ✅ **Rust-native Alpaca provider**: Complete and tested
- ✅ **Provider abstraction**: Implemented and working
- ✅ **Results validation**: Python SDK and Rust-native approaches produce identical results
- 🔄 **CLI interface**: Future work for Rust-native approach
- 🔄 **Legacy cleanup**: Gradual removal of Python SDK dependencies

## Dependencies

### Core Dependencies
- `tokio`: Async runtime
- `reqwest`: HTTP client for API requests
- `serde`: Serialization/deserialization
- `chrono`: Date and time handling
- `async-trait`: Async trait support
- `governor`: Rate limiting
- `thiserror`: Error handling

### Legacy Dependencies (Feature-Gated)
- `clap`: Command-line argument parsing
- `pyo3`: Python bindings
- `polars`: DataFrame operations
- `bincode`: Binary serialization

## Version

Current version: **1.3.4**
