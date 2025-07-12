# Stock Trading Bot

[![Rust](https://img.shields.io/badge/Developed%20in-Rust-orange?logo=rust)](https://www.rust-lang.org)
[![Project Status: Active](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)

This repository contains a high-performance, Rust-native stock trading bot designed for algorithmic trading. The system is architected for reliability, speed, and modularity, leveraging modern data engineering tools.

**Evolution Notice**: This project has evolved from Python/Go prototypes into a fully Rust-native system to leverage its safety, performance, and concurrency features. For more details, see our [Architecture Evolution Blog Post](docs/blog/2025-01-rust-architecture-pivot.md).

## Core Components

### 1. Market Data Ingestor
The `market_data_ingestor` is a key component responsible for fetching financial market data from various sources.

**Key Features**:
- **High-Performance**: Built with asynchronous Rust (`tokio`) for efficient, non-blocking I/O.
- **Extensible Provider API**: Easily add new data sources. Currently supports Alpaca.
- **Data Serialization**: Uses `serde` for robust JSON parsing and `polars` for efficient in-memory data representation.
- **Flexible Output**: Can be used as a standalone CLI tool or as a Python-callable module via `PyO3`.
- **Rate Limiting**: Integrated request throttling to respect provider API limits.

### 2. Storage Service
The storage layer is built on **Delta Lake**, providing ACID transactions, time travel, and scalable metadata for large-scale financial datasets. This ensures data integrity and reproducibility for backtesting and live trading.

### 3. Trading Logic
The core trading strategies and backtesting engine are implemented in Rust, ensuring memory safety and computational efficiency.

## Why Rust?
- **Performance**: Achieves C-level speed without sacrificing safety.
- **Fearless Concurrency**: Safely parallelize backtesting and data processing.
- **Type Safety**: Catches data-related errors at compile time, crucial for financial applications.
- **Modern Ecosystem**: Access to cutting-edge crates like `tokio`, `polars`, and `delta-rs`.
- **WASM Future**: Potential to build interactive web-based dashboards with WebAssembly.

## Development Progress
- **Q1 2025**: Migrated core trading engine and data ingestion services to Rust.
- **2024**: Developed initial prototypes in Python and Go. Legacy code is available in the `archive/` directory.

## Getting Started
*Instructions on how to build and run the project will be added here soon.*