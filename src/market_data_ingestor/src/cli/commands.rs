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

        #[arg(long, default_value = "300")]
        base_delay_ms: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parsing() {
        // Test basic CLI parsing
        let args = vec![
            "program",
            "--config",
            "config.toml",
            "single",
            "--symbols",
            "AAPL",
            "--amount",
            "5",
            "--start",
            "2023-01-01T00:00:00Z",
            "--end",
            "2023-01-31T00:00:00Z",
        ];

        let cli = Cli::parse_from(args);

        assert_eq!(cli.config, "config.toml");
        match cli.command {
            Commands::Single {
                symbols,
                amount,
                unit,
                start,
                end,
            } => {
                assert_eq!(symbols, "AAPL");
                assert_eq!(amount, 5);
                assert_eq!(unit, "m"); // Default value
                assert_eq!(start, "2023-01-01T00:00:00Z");
                assert_eq!(end, "2023-01-31T00:00:00Z");
            }
            _ => panic!("Expected Single command"),
        }
    }

    #[test]
    fn test_batch_command_parsing() {
        // Test batch CLI parsing
        let args = vec![
            "program",
            "--config",
            "config.toml",
            "batch",
            "--source",
            "file",
            "--input",
            "batch_params.json",
            "--max-retries",
            "5",
        ];

        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Batch {
                source,
                input,
                max_retries,
                base_delay_ms,
            } => {
                assert_eq!(source, "file");
                assert_eq!(input, Some("batch_params.json".to_string()));
                assert_eq!(max_retries, 5);
                assert_eq!(base_delay_ms, 300); // Default value
            }
            _ => panic!("Expected Batch command"),
        }
    }

    #[test]
    fn test_batch_param_item_serialization() {
        // Test BatchParamItem serialization/deserialization
        let item = BatchParamItem {
            symbols: "AAPL,MSFT".to_string(),
            amount: 5,
            unit: "m".to_string(),
            start: "2023-01-01T00:00:00Z".to_string(),
            end: "2023-01-31T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: BatchParamItem = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbols, "AAPL,MSFT");
        assert_eq!(deserialized.amount, 5);
        assert_eq!(deserialized.unit, "m");
        assert_eq!(deserialized.start, "2023-01-01T00:00:00Z");
        assert_eq!(deserialized.end, "2023-01-31T00:00:00Z");
    }

    #[test]
    fn test_cli_validation() {
        // Verify required arguments are enforced
        let cmd = Cli::command();

        // Test that config is required
        cmd.clone()
            .try_get_matches_from(vec!["program", "single", "--symbols", "AAPL"])
            .expect_err("Should fail without config");

        // Test that symbols is required for Single command
        cmd.clone()
            .try_get_matches_from(vec!["program", "--config", "config.toml", "single"])
            .expect_err("Should fail without symbols");
    }
}
