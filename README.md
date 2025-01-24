# stock_trading_bot [![Rust](https://img.shields.io/badge/Maintained%20in-Rust-orange?logo=rust)](https://www.rust-lang.org)

**Evolution Notice**: This project has evolved into a Rust-native system. 
See [Why Rust?](#why-rust) and our [Architecture Evolution Blog](docs/blog/2025-01-rust-architecture-pivot.md).

## Key Features (Updated)
- **Delta Lake Storage**: Reliable ACID-compliant data layer via delta-rs  
- **Rust Core**: Memory-safe foundation for trading logic  
- **Arrow Flight**: High-speed data transport between components  

## Why Rust?
- **Fearless Concurrency**: Safe parallelism for backtesting  
- **Type Safety**: Catch errors at compile time  
- **WASM Future**: Web-based dashboards via WebAssembly  
- **Ecosystem**: Growing data engineering crates (delta-rs, polars, etc)

## Development Progress
- **2025 Q1**: Core trading engine migration to Rust  
- **2024**: Python/Go prototypes ([see legacy code](archive/))  