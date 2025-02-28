use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Path to the config file(data_ingestor.toml)
    #[arg(short, long)]
    pub config: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Serialize, Deserialize)]
pub struct BatchParamItem {
    pub symbols: String,
    pub amount: u32,
    pub unit: String,
    pub start: String,
    pub end: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a single data fetch request
    Single {
        /// Comma-separated list of symbols (e.g. "AAPL,MSFT")
        #[arg(long)]
        symbols: String,

        /// Timeframe amount (numeric value)
        #[arg(long)]
        amount: u32,

        /// Timeframe unit: m (minute), h (hour), d (day), w (week), M (month)
        #[arg(long, default_value = "m")]
        unit: String,

        /// Start datetime in ISO8601 format (e.g. "2025-01-01T09:30:00Z")
        #[arg(long)]
        start: String,

        /// End datetime in ISO8601 format (e.g. "2025-01-30T16:00:00Z")
        #[arg(short, long)]
        end: String,
    },

    /// Execute batch data fetch requests
    Batch {
        /// Source of batch parameters: file, stdin, or json
        #[arg(long, default_value = "stdin")]
        source: String,

        /// Path to JSON file (when source=file) or inline JSON string (when source=json)
        #[arg(long)]
        input: Option<String>,

        /// Maximum number of retries for failed requests
        #[arg(long, default_value = "3")]
        max_retries: u32,

        #[arg(long, default_value = "1000")]
        base_delay_ms: u64,
    },
}
