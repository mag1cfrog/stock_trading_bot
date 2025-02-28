use std::process::Command;
use std::str;

#[test]
fn test_single_fetch_cli() -> Result<(), Box<dyn std::error::Error>> {
    // Set up arguments.
    // Adjust the config path below as needed (for instance, to a test config file).
    let config_path = "/home/hanbo/repo/stock_trading_bot/src/configs/data_ingestor.toml";
    let symbols = "AAPL,MSFT";
    let timeframe_amount = "5"; // 5 minutes
    let timeframe_unit = "min";
    let start = "2025-01-01T09:30:00Z";
    let end = "2025-01-01T16:00:00Z";

    let output =
        Command::new("/home/hanbo/repo/stock_trading_bot/src/target/debug/market_data_ingestor")
            .args([
                "--config",
                config_path,
                "single",
                "--symbols",
                symbols,
                "--amount",
                timeframe_amount,
                "--unit",
                timeframe_unit,
                "--start",
                start,
                "--end",
                end,
            ])
            .output()?;

    // If the process failed, print its stderr.
    if !output.status.success() {
        eprintln!("stderr: {}", str::from_utf8(&output.stderr)?);
    }
    assert!(output.status.success(), "Binary did not exit successfully");

    let stdout = str::from_utf8(&output.stdout)?;
    println!("App output: {}", stdout);

    // Check that output contains a feather file name.
    assert!(
        stdout.contains(".feather"),
        "Expected a feather file name in output, got: {}",
        stdout
    );

    Ok(())
}
