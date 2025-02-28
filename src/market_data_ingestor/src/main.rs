use clap::Parser;
use market_data_ingestor::{
    cli::{
        commands::{Cli, Commands},
        params::*,
    },
    io::dataframe::write_dataframe_to_temp,
    models::stockbars::StockBarsParams,
    requests::historical::{StockBarData, fetch_historical_bars},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            end,
        } => {
            // Parse symbols (comma-separated)
            let symbol_list: Vec<String> =
                symbols.split(',').map(|s| s.trim().to_string()).collect();

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
        }

        Commands::Batch {
            source,
            input,
            max_retries,
            base_delay_ms,
        } => {
            // Parse parameters based on source
            let params_list = match source.as_str() {
                "file" => {
                    let file_path = input.as_ref().ok_or("File path required for source=file")?;
                    parse_batch_params_from_file(file_path)?
                }
                "stdin" => parse_batch_params_from_stdin()?,
                "json" => {
                    let json_str = input
                        .as_ref()
                        .ok_or("JSON string required for source=json")?;
                    parse_batch_params_from_json_string(json_str)?
                }

                _ => return Err("Invalid source. Use 'file', 'stdin', or 'json'".into()),
            };

            // Execute batch request
            let results =
                market_data.fetch_bars_batch_partial(&params_list, *max_retries, *base_delay_ms)?;

            // Process results and save successful ones
            let mut success_count = 0;
            let mut error_count = 0;

            for (i, result) in results.into_iter().enumerate() {
                match result {
                    Ok(mut df) => {
                        // For each successful request, save the DataFrame
                        if let Some(symbol) = params_list[i].symbols.first() {
                            let output_path = write_dataframe_to_temp(&mut df, symbol)?;
                            println!("{}", output_path.display());
                            success_count += 1;
                        }
                    }
                    Err(e) => {
                        if let Some(symbol) = params_list[i].symbols.first() {
                            eprintln!("ERROR: {} - {}", symbol, e);
                        }
                        error_count += 1;
                    }
                }
            }

            // Print summary to stderr so it doesn't intefere with machine parisng of paths
            eprintln!(
                "SUMMARY: {} succeeded, {} failed",
                success_count, error_count
            );
        }
    }
    Ok(())
}
