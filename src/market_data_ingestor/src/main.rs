use std::{env, fs::{self, File}, io::{self, Read}, path::PathBuf};
use std::error::Error;

use chrono::Utc;
use clap::{Parser, Subcommand};
use market_data_ingestor::{models::{stockbars::StockBarsParams, timeframe::{TimeFrame, TimeFrameError}}, requests::historical::{fetch_historical_bars, StockBarData}};
use polars::frame::DataFrame;
use polars_io::ipc::IpcWriter;
use polars_io::SerWriter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Path to the config file(data_ingestor.toml)
    #[arg(short, long)]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Serialize, Deserialize)]
struct BatchParamItem {
    symbols: String,
    amount: u32,
    unit: String,
    start: String,
    end: String,
}

#[derive(Subcommand)]
enum Commands {
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
    }
}

fn write_dataframe_to_temp(df: &mut DataFrame, symbol: &str) -> Result<PathBuf, Box<dyn Error>> {
    // Determine the base temporary directory. By default, this is /tmp in Debian.
    let mut base_temp = env::temp_dir();
    // Create a subfolder for our application, e.g. /tmp/market_data_ingestor
    base_temp.push("market_data_ingestor");
    if !base_temp.exists() {
        fs::create_dir_all(&base_temp)?;
    }

    // Create a filename that includes the stock symbol and a UUID.
    // Also, you can include a timestamp if desired.
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let filename = format!(
        "{}_{}_{}.feather",
        symbol,
        timestamp,
        Uuid::new_v4()
    );
    let mut output_path = base_temp.clone();
    output_path.push(filename);

    // Create file and write the DataFrame using Polars' IPC writer (compatible with Arrow/Feather).
    let mut file = File::create(&output_path)?;
    let mut writer = IpcWriter::new(&mut file);
    writer.finish(df)?;

    Ok(output_path)
}

fn parse_timeframe(amount: u32, unit: &str) -> Result<TimeFrame, Box<dyn Error>> {
    match unit.trim().to_lowercase().as_str() {
        "m" | "min" | "minute" => TimeFrame::minutes(amount),
        "h" | "hr" | "hour" => TimeFrame::hours(amount),
        "d" | "day" => TimeFrame::day(),
        "w" | "wk" | "week" => TimeFrame::week(),
        "M" | "mo" | "month" => TimeFrame::months(amount),
        _ => Err(TimeFrameError::InvalidInput { message: format!("Invalid timeframe unit: {}", unit) })
    }.map_err(|e| e.into())
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let cli = Cli::parse();

    // Initialize the Python environment using the config
    // This calls [init_python](src/utils/python_init.rs) and sets up the interpreter.
    let market_data = futures::executor::block_on(StockBarData::new(&cli.config))?;

    // Process subcommands
    match &cli.command {
        Commands::Single { 
            symbols, 
            amount,
            unit,
            start, 
            end 
        } => {
            // Parse symbols (comma-separated)
            let symbol_list: Vec<String> = symbols
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            let first_symbol = symbol_list.first().unwrap().clone();

            // Parse timeframe; if it parses as a number we'll assume minutes,
            // otherwise interpret some presets.
            let tf = parse_timeframe(*amount, unit)?;
            let start_dt = start.parse::<chrono::DateTime<chrono::Utc>>()?;
            let end_dt = end.parse::<chrono::DateTime<chrono::Utc>>()?;

            let params = StockBarsParams {
                symbols: symbol_list,
                timeframe: tf,
                start: start_dt,
                end: end_dt,
            };
            let mut df = fetch_historical_bars(&market_data, params)?;
            let output_path = write_dataframe_to_temp(&mut df, &first_symbol)?;
            println!("{}", output_path.display())
        },

        Commands::Batch { 
            source, 
            input, 
            max_retries, 
            base_delay_ms 
        } => {
            // Parse parameters based on source
            let params_list = match source.as_str() {
                "file" => {
                    let file_path = input.as_ref().ok_or("File path required for source=file")?;
                    parse_batch_params_from_file(file_path)?
                },
                "stdin" => {
                    parse_batch_params_from_stdin()?
                },
                "json" => {
                    let json_str = input.as_ref().ok_or("JSON string required for source=json")?;
                    parse_batch_params_from_json_string(json_str)?
                }

                _ => return Err("Invalid source. Use 'file', 'stdin', or 'json'".into())
            };

            // Execute batch request
            let results = market_data.fetch_bars_batch_partial(&params_list, *max_retries, *base_delay_ms)?;

            // Process results and save successful ones
            let mut succes_count = 0;
            let mut error_count = 0;

            for (i, result) in results.into_iter().enumerate() {
                match result {
                    Ok(mut df) => {
                        // For each successful request, save the DataFrame
                        if let Some(symbol) = params_list[i].symbols.first() {
                            let output_path = write_dataframe_to_temp(&mut df, symbol)?;
                            println!("{}", output_path.display());
                            succes_count += 1;
                        }
                    },
                    Err(e) => {
                        if let Some(symbol) = params_list[i].symbols.first() {
                            eprintln!("ERROR: {} - {}", symbol, e);
                        }
                        error_count += 1;
                    }
                }
            }

            // Print summary to stderr so it doesn't intefere with machine parisng of paths
            eprintln!("SUMMARY: {} succeeded, {} failed", succes_count, error_count);
        }
    }
    Ok(())
}

fn parse_batch_params_from_stdin() -> Result<Vec<StockBarsParams>, Box<dyn Error>> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;

    // Try to parse as binary format first(more efficient)
    let json_value: Result<Value, _> = bincode::deserialize(&buffer)
        .or_else(|_| {
            // If binary foramt fails, try as JSON
            serde_json::from_slice(&buffer)
        });

    match json_value {
        Ok(value) => parse_batch_params_from_json_value(value),
        Err(e) => Err(format!("Failed to parse stdin data: {}", e).into())
    }
}

fn parse_batch_params_from_json_string(json_str: &str) -> Result<Vec<StockBarsParams>, Box<dyn Error>> {
    let json_value: Value = serde_json::from_str(json_str)?;
    parse_batch_params_from_json_value(json_value)
}

fn parse_batch_params_from_json_value(json_value: Value) -> Result<Vec<StockBarsParams>, Box<dyn Error>> {
    let items: Vec<BatchParamItem> = serde_json::from_value(json_value)?;

    let mut params_list = Vec::with_capacity(items.len());

    for item in items {
        // Parse symbols (comma-separated)
        let symbols: Vec<String> = item.symbols
            .split(",")
            .map(|s| s.trim().to_string())
            .collect();

        // Parse timeframe
        let timeframe = parse_timeframe(item.amount, &item.unit)?;

        // Parse date
        let start = item.start.parse::<chrono::DateTime<chrono::Utc>>()?;
        let end = item.end.parse::<chrono::DateTime<chrono::Utc>>()?;

        params_list.push(StockBarsParams { 
            symbols, 
            timeframe, 
            start, 
            end
        });
    }

    Ok(params_list)
}

fn parse_batch_params_from_file(file_path: &str) -> Result<Vec<StockBarsParams>, Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;
    let json_value = serde_json::from_str(&content)?;
    parse_batch_params_from_json_value(json_value)
}

