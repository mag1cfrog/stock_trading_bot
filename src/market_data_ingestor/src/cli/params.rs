use std::io::Read;
use std::{error::Error, fs, io};

use serde_json::Value;

use crate::models::{
    stockbars::StockBarsParams,
    timeframe::{TimeFrame, TimeFrameError},
};

use super::commands::BatchParamItem;

pub fn parse_timeframe(amount: u32, unit: &str) -> Result<TimeFrame, Box<dyn Error>> {
    match unit.trim().to_lowercase().as_str() {
        "m" | "min" | "minute" => TimeFrame::minutes(amount),
        "h" | "hr" | "hour" => TimeFrame::hours(amount),
        "d" | "day" => TimeFrame::day(),
        "w" | "wk" | "week" => TimeFrame::week(),
        "M" | "mo" | "month" => TimeFrame::months(amount),
        _ => Err(TimeFrameError::InvalidInput {
            message: format!("Invalid timeframe unit: {}", unit),
        }),
    }
    .map_err(|e| e.into())
}

pub fn parse_batch_params_from_stdin() -> Result<Vec<StockBarsParams>, Box<dyn Error>> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;

    // Try to parse as binary format first(more efficient)
    let json_value: Result<Value, _> = bincode::deserialize(&buffer).or_else(|_| {
        // If binary foramt fails, try as JSON
        serde_json::from_slice(&buffer)
    });

    match json_value {
        Ok(value) => parse_batch_params_from_json_value(value),
        Err(e) => Err(format!("Failed to parse stdin data: {}", e).into()),
    }
}

pub fn parse_batch_params_from_json_string(
    json_str: &str,
) -> Result<Vec<StockBarsParams>, Box<dyn Error>> {
    let json_value: Value = serde_json::from_str(json_str)?;
    parse_batch_params_from_json_value(json_value)
}

pub fn parse_batch_params_from_json_value(
    json_value: Value,
) -> Result<Vec<StockBarsParams>, Box<dyn Error>> {
    let items: Vec<BatchParamItem> = serde_json::from_value(json_value)?;

    let mut params_list = Vec::with_capacity(items.len());

    for item in items {
        // Parse symbols (comma-separated)
        let symbols: Vec<String> = item
            .symbols
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
            end,
        });
    }

    Ok(params_list)
}

pub fn parse_batch_params_from_file(
    file_path: &str,
) -> Result<Vec<StockBarsParams>, Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;
    let json_value = serde_json::from_str(&content)?;
    parse_batch_params_from_json_value(json_value)
}
