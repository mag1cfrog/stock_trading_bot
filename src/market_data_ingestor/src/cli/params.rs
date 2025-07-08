use std::io::Read;
use std::{error::Error, fs, io};

use serde_json::Value;

use crate::models::timeframe::TimeFrameUnit;
use crate::models::{
    stockbars::StockBarsParams,
    timeframe::{TimeFrame, TimeFrameError},
};

use super::commands::BatchParamItem;

pub fn parse_timeframe(amount: u32, unit: &str) -> Result<TimeFrame, Box<dyn Error>> {
    let unit = match unit.trim().to_lowercase().as_str() {
        "m" | "min" | "minute" => TimeFrame::new(amount, TimeFrameUnit::Minute),
        "h" | "hr" | "hour" => TimeFrame::new(amount, TimeFrameUnit::Hour),
        "d" | "day" => TimeFrame::new(amount, TimeFrameUnit::Day),
        "w" | "wk" | "week" => TimeFrame::new(amount, TimeFrameUnit::Week),
        "mo" | "month" => TimeFrame::new(amount, TimeFrameUnit::Month),
        _ => return Box::new(TimeFrameError::InvalidInput { message: format!("Invalid timeframe unit: {}", unit)}),
    };
    Ok(TimeFrame::new(amount, unit))
}

#[cfg(feature = "alpaca-python-sdk")]
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

#[cfg(feature = "alpaca-python-sdk")]
pub fn parse_batch_params_from_json_string(
    json_str: &str,
) -> Result<Vec<StockBarsParams>, Box<dyn Error>> {
    let json_value: Value = serde_json::from_str(json_str)?;
    parse_batch_params_from_json_value(json_value)
}

#[cfg(feature = "alpaca-python-sdk")]
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

#[cfg(feature = "alpaca-python-sdk")]
pub fn parse_batch_params_from_file(
    file_path: &str,
) -> Result<Vec<StockBarsParams>, Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;
    let json_value = serde_json::from_str(&content)?;
    parse_batch_params_from_json_value(json_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::timeframe::TimeFrameUnit;

    #[test]
    fn test_parse_timeframe() {
        // Test various valid timeframe formats
        let minute_tf = parse_timeframe(5, "m").unwrap();
        assert!(matches!(
            minute_tf,
            TimeFrame {
                amount: 5,
                unit: TimeFrameUnit::Minute
            }
        ));

        let hour_tf = parse_timeframe(2, "h").unwrap();
        assert!(matches!(
            hour_tf,
            TimeFrame {
                amount: 2,
                unit: TimeFrameUnit::Hour
            }
        ));

        let day_tf = parse_timeframe(1, "d").unwrap();
        assert!(matches!(
            day_tf,
            TimeFrame {
                amount: 1,
                unit: TimeFrameUnit::Day
            }
        ));

        // Test error cases
        assert!(parse_timeframe(2, "d").is_err()); // Day only supports amount=1
        assert!(parse_timeframe(60, "m").is_err()); // Minutes only up to 59
        assert!(parse_timeframe(5, "invalid").is_err()); // Invalid unit
    }

    #[cfg(feature = "alpaca-python-sdk")]
    #[test]
    fn test_parse_batch_params_from_json_string() {
        let json_str = r#"[
            {
                "symbols": "AAPL",
                "amount": 5,
                "unit": "m",
                "start": "2023-01-01T00:00:00Z",
                "end": "2023-01-31T00:00:00Z"
            },
            {
                "symbols": "MSFT,GOOGL",
                "amount": 1,
                "unit": "d",
                "start": "2023-01-01T00:00:00Z",
                "end": "2023-01-31T00:00:00Z"
            }
        ]"#;

        let params_list = parse_batch_params_from_json_string(json_str).unwrap();

        assert_eq!(params_list.len(), 2);
        assert_eq!(params_list[0].symbols, vec!["AAPL"]);
        assert_eq!(params_list[1].symbols, vec!["MSFT", "GOOGL"]);
    }

    #[cfg(feature = "alpaca-python-sdk")]
    #[test]
    fn test_parse_batch_params_from_json_value() {
        let json_value: serde_json::Value = serde_json::from_str(
            r#"[
            {
                "symbols": "AAPL",
                "amount": 5,
                "unit": "m",
                "start": "2023-01-01T00:00:00Z",
                "end": "2023-01-31T00:00:00Z"
            }
        ]"#,
        )
        .unwrap();

        let params_list = parse_batch_params_from_json_value(json_value).unwrap();

        assert_eq!(params_list.len(), 1);
        assert_eq!(params_list[0].symbols, vec!["AAPL"]);
        assert!(matches!(
            params_list[0].timeframe,
            TimeFrame {
                amount: 5,
                unit: TimeFrameUnit::Minute
            }
        ));
    }
}
