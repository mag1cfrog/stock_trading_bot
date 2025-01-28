---
title: "Evolution of Stock Trading Bot: Embracing Rust for Robust Financial Systems"
date: 2025-01-15
author: Hanbo Wang
---

# Evolution of Stock Trading Bot: Embracing Rust for Robust Financial Systems

## From Prototyping to Production-Grade Systems

### Lessons from Our Python/Go Journey (2024)
Our initial development phase taught us valuable lessons:

- **The API Benchmarking Trap**
  - Achieved 400+ calls/min with custom Go/Python implementations
  - Learned: Raw speed means little without system-level reliability
  - Reality check: Got IP-banned during stress tests

- **Storage System Rabbit Holes**
  - Built custom [DuckDB managers](archive/code_backup/Python/STB_storage_manager_py/src/stb_storage_manager_py/duckdb/duckdb_manager.py)
  - Benchmarking obsession: Tested multiple storage patterns, transport protocols, pedantic details
  - Wasted effort: Spent months optimizing what we'd eventually replace

- **Dashboard Distractions**
  - Created `CryptoQuotesDashVisualizer` [(legacy code)](archive/code_backup/Python/stock_trading_bot/src/stock_trading_bot/visualization/visualizers/crypto_quotes_dash_visualizer.py)
  - Hard truth: Pretty graphs ≠ trading profitability

## Why Rust? Professional-Grade Requirements

### Python's Hidden Costs
```python
# The type safety nightmare we leave behind
def calculate_position_size(account_balance, risk_percent):
    # What if risk_percent is string "1.5"? 
    return account_balance * (risk_percent / 100)
```

- **Runtime Surprises**: Type errors crashing live trading
- **GC Uncertainties**: Pauses during high-frequency operations
- **Debugging Debt**: 40% time spent tracing abstraction leaks

### Go's Limitations Revealed
- **Data Wrangling Friction**: Lack of expressiveness for data-heavy workflows
- **Error Handling**: Error handling too simplistic for financial system needs


### Rust's Financial System Advantages
- **Memory Safety**: No segfaults in 24/7 trading processes
- **Zero-Cost Abstractions**: 
  ```rust
  // Compile-time validated order types
  enum Order {
      Market {
          symbol: String,
          quantity: f64,
          direction: Direction,
      },
      Limit {
          symbol: String,
          quantity: f64,
          limit_price: f64,
          expiry: DateTime<Utc>,
      },
  }
  ```
- **Async Simplicity**: Tokio runtime for market data streams
- **WASM Future**: Same code for backend and web dashboards


## New Development Philosophy: Learning Through Doing

### 1. Start With Actual Jobs, Not Abstract Layers
**Old Way**: We fell into the trap of building infrastructure first - creating storage managers and dashboards before having working trading logic. This led to:

- Components that didn't connect to real needs
- Wasted time on "optimizations" for non-existent workflows

**New Approach**

```rust
// Start simple: NVDA data pipeline
#[tokio::main]
async fn main() -> Result<(), TradingError> {
    // Phase 1: Just fetch and log
    let nvda_data = AlpacaFetcher::new().get_historical("NVDA").await?;
    println!("Fetched {} NVDA bars", nvda_data.len());
    
    // Phase 2: Add basic storage (we'll use delta-rs here later)
    let mut storage = FileStorage::new("data/nvda.json");
    storage.save(&nvda_data).await?;
    
    // Phase 3: Simple moving average display
    let sma_20 = calculate_sma(&nvda_data, 20);
    plot_data(&nvda_data, &sma_20);
    
    Ok(())
}
```


### 2. Build Outward From Working Code

Instead of theoretical layer diagrams, we now:

1. **Find the Smallest Working Scenario**(e.g., "Show NVDA's SMA crossover points")


2. **Extract Natural Components**

- Data fetching emerges when we add a second stock

- Error handling becomes critical on first API failure


3. **Refactor Only When Pain Demands**

- No premature "generic storage managers"

- Let the code tell us what abstractions it needs

### 3. Rust's Explicitness as a Design Tool

What we initially saw as Rust's "hassle" became our secret weapon:
```rust
// Compiler-guided development
struct Trade {
    symbol: String,
    // Forgot timestamp? Compiler error when saving to DB!
    // timestamp: DateTime<Utc>, 
    quantity: f64,
    price: f64,
}

impl Trade {
    // Must handle Result early
    fn validate(&self) -> Result<(), TradeError> {
        if self.quantity <= 0.0 {
            return Err(TradeError::InvalidQuantity);
        }
        // Forced to consider price validity
        if self.price <= 0.0 || self.price > 1_000_000.0 {
            return Err(TradeError::PriceOutOfRange);
        }
        Ok(())
    }
}
```

### 4. Avoiding Optimization Rabbit Holes

- **Storage**: Using delta-rs instead of custom DuckDB
- **APIs**: Start with official SDKs, optimize only when proven needed
- **Architecture**: Let performance metrics guide rewrites, not speculation


## Why This Matters

Financial systems demand:
- **💰 Predictability**: No GC pauses during market opens
- **🔒 Auditability**: Immutable Delta Lake transactions
- **🛡️ Safety**: Compile-time checks > runtime exceptions

Rust enables this from day one - no costly retrofits.

---

## Modern Architecture with Delta Lake

### Why We Chose delta-rs
| Feature               | DuckDB Custom Solution | delta-rs (Delta Lake) |
|-----------------------|------------------------|-----------------------|
| ACID Compliance       | ❌ Manual              | ✅ Built-in           |
| Time Travel           | ❌ Complex             | ✅ 1-line query       |
| Schema Evolution      | ❌ Destructive         | ✅ Safe migrations    |
| Cross-Language        | ❌ Python-only         | ✅ Python/Rust/Java   |

```rust
// Example: Creating Delta Lake table with Rust
use deltalake::DeltaTableBuilder;

async fn create_trade_table() -> Result<(), deltalake::errors::DeltaError> {
    let table = DeltaTableBuilder::new()
        .with_location("s3://trading-data/nvda")
        .with_table_name("nvda_trades")
        .with_comment("NVDA stock trades")
        .with_columns(get_trade_schema()) // Arrow schema
        .await?;

    table.create().await?;
    Ok(())
}
```