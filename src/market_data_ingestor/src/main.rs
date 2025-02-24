use std::{env, fs::{self, File}, path::PathBuf};
use std::error::Error;

use chrono::Utc;
use clap::{Parser, Subcommand};
use market_data_ingestor::{models::{stockbars::StockBarsParams, timeframe::TimeFrame}, requests::historical::{fetch_historical_bars, StockBarData}};
use polars::frame::DataFrame;
use polars_io::ipc::IpcWriter;
use polars_io::SerWriter;
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

#[derive(Subcommand)]
enum Commands {
    /// Execute a single data fetch request
    Single {
        /// Comma-separated list of symbols (e.g. "AAPL,MSFT")
        #[arg(long)]
        symbols: String,

        /// Timeframe: either a number (for minutes) or a preset like "day" or "week"
        #[arg(short, long)]
        timeframe: String,

        /// Start datetime in ISO8601 format (e.g. "2025-01-01T09:30:00Z")
        #[arg(long)]
        start: String,

        /// End datetime in ISO8601 format (e.g. "2025-01-30T16:00:00Z")
        #[arg(short, long)]
        end: String,
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

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let cli = Cli::parse();

    // Initialize the Python environment using the config
    // This calls [init_python](src/utils/python_init.rs) and sets up the interpreter.
    let market_data = futures::executor::block_on(StockBarData::new(&cli.config))?;

    // Process subcommands
    match &cli.command {
        Commands::Single { 
            symbols, 
            timeframe, 
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
            let tf = if let Ok(n) = timeframe.parse::<u32>() {
                TimeFrame::minutes(n)?
            } else {
                match timeframe.as_str() {
                    "day" => TimeFrame::day()?,
                    "week" => TimeFrame::week()?,
                    _ => TimeFrame::minutes(5)?,
                }
            };
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
        }
    }
    Ok(())
}
