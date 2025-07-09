#![cfg(all(test, feature = "cli", feature = "alpaca-python-sdk"))]
use std::process::Command;
use std::str;

use serial_test::serial;

fn count_feather_files(output: &str) -> usize {
    output.matches(".feather").count()
}

#[test]
#[serial]
#[ignore]
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

    // Verify exactly one feather file is returned
    let feather_count = count_feather_files(stdout);
    assert_eq!(
        feather_count, 1,
        "Expected 1 feather file, found {}",
        feather_count
    );

    Ok(())
}

#[test]
#[serial]
#[ignore]
fn test_batch_fetch_cli_with_file_input() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a temporary JSON file with batch parameters
    let mut temp_file = NamedTempFile::new()?;
    let json_content = r#"[
        {
            "symbols": "AAPL",
            "amount": 5,
            "unit": "m",
            "start": "2025-01-01T09:30:00Z",
            "end": "2025-01-01T16:00:00Z"
        },
        {
            "symbols": "MSFT",
            "amount": 1,
            "unit": "d",
            "start": "2025-01-01T09:30:00Z",
            "end": "2025-01-05T16:00:00Z"
        }
    ]"#;

    write!(temp_file, "{}", json_content)?;
    let file_path = temp_file.path().to_str().unwrap();

    // Run CLI with file input
    let config_path = "/home/hanbo/repo/stock_trading_bot/src/configs/data_ingestor.toml";

    let output =
        Command::new("/home/hanbo/repo/stock_trading_bot/src/target/debug/market_data_ingestor")
            .args([
                "--config",
                config_path,
                "batch",
                "--source",
                "file",
                "--input",
                file_path,
            ])
            .output()?;

    if !output.status.success() {
        eprintln!("stderr: {}", str::from_utf8(&output.stderr)?);
    }
    assert!(output.status.success(), "Binary did not exit successfully");

    let stdout = str::from_utf8(&output.stdout)?;
    println!("App output: {}", stdout);

    // Check that output contains feather file names
    assert!(
        stdout.contains(".feather"),
        "Expected feather file names in output, got: {}",
        stdout
    );

    // Check summary in stderr
    let stderr = str::from_utf8(&output.stderr)?;
    assert!(
        stderr.contains("SUMMARY:"),
        "Expected summary in stderr, got: {}",
        stderr
    );

    // Verify we got exactly two feather files
    let feather_count = count_feather_files(stdout);
    assert_eq!(
        feather_count, 2,
        "Expected 2 feather files, found {}",
        feather_count
    );

    Ok(())
}

#[test]
#[serial]
#[ignore]
fn test_batch_fetch_cli_with_json_input() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "/home/hanbo/repo/stock_trading_bot/src/configs/data_ingestor.toml";
    let json_input = r#"[{"symbols":"AAPL","amount":5,"unit":"m","start":"2025-01-01T09:30:00Z","end":"2025-01-01T16:00:00Z"}, {"symbols":"AAPL","amount":5,"unit":"m","start":"2025-01-01T09:30:00Z","end":"2025-01-01T16:00:00Z"}]"#;

    let output =
        Command::new("/home/hanbo/repo/stock_trading_bot/src/target/debug/market_data_ingestor")
            .args([
                "--config",
                config_path,
                "batch",
                "--source",
                "json",
                "--input",
                json_input,
            ])
            .output()?;

    if !output.status.success() {
        eprintln!("stderr: {}", str::from_utf8(&output.stderr)?);
    }
    assert!(output.status.success(), "Binary did not exit successfully");

    let stdout = str::from_utf8(&output.stdout)?;
    println!("App output: {}", stdout);

    assert!(
        stdout.contains(".feather"),
        "Expected a feather file name in output, got: {}",
        stdout
    );

    let stderr = str::from_utf8(&output.stderr)?;
    assert!(
        stderr.contains("SUMMARY:"),
        "Expected summary in stderr, got: {}",
        stderr
    );

    // Verify we got exactly two feather files
    let feather_count = count_feather_files(stdout);
    assert_eq!(
        feather_count, 2,
        "Expected 2 feather files, found {}",
        feather_count
    );

    Ok(())
}

#[test]
#[serial]
#[ignore]
fn test_batch_fetch_cli_with_stdin_input() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let config_path = "/home/hanbo/repo/stock_trading_bot/src/configs/data_ingestor.toml";
    let json_input = r#"[{"symbols":"AAPL","amount":5,"unit":"m","start":"2025-01-01T09:30:00Z","end":"2025-01-01T16:00:00Z"}, {"symbols":"AAPL","amount":5,"unit":"m","start":"2025-01-01T09:30:00Z","end":"2025-01-01T16:00:00Z"}]"#;

    // Create a command that will read from stdin
    let mut cmd =
        Command::new("/home/hanbo/repo/stock_trading_bot/src/target/debug/market_data_ingestor")
            .args(["--config", config_path, "batch", "--source", "stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

    // Write to stdin
    if let Some(stdin) = cmd.stdin.as_mut() {
        stdin.write_all(json_input.as_bytes())?;
    }

    // Close stdin to avoid deadlock waiting for more input
    drop(cmd.stdin.take());

    // Wait for the command to complete and capture output
    let output = cmd.wait_with_output()?;

    if !output.status.success() {
        eprintln!("stderr: {}", str::from_utf8(&output.stderr)?);
    }
    assert!(output.status.success(), "Binary did not exit successfully");

    let stdout = str::from_utf8(&output.stdout)?;
    println!("App output: {}", stdout);

    assert!(
        stdout.contains(".feather"),
        "Expected a feather file name in output, got: {}",
        stdout
    );

    let stderr = str::from_utf8(&output.stderr)?;
    assert!(
        stderr.contains("SUMMARY:"),
        "Expected summary in stderr, got: {}",
        stderr
    );

    // Verify we got exactly two feather files
    let feather_count = count_feather_files(stdout);
    assert_eq!(
        feather_count, 2,
        "Expected 2 feather files, found {}",
        feather_count
    );

    Ok(())
}

#[test]
#[ignore]
fn test_batch_fetch_cli_with_invalid_source() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "/home/hanbo/repo/stock_trading_bot/src/configs/data_ingestor.toml";

    let output =
        Command::new("/home/hanbo/repo/stock_trading_bot/src/target/debug/market_data_ingestor")
            .args([
                "--config",
                config_path,
                "batch",
                "--source",
                "invalid_source", // This is an invalid source
            ])
            .output()?;

    // This should fail
    assert!(
        !output.status.success(),
        "Binary should have failed with invalid source"
    );

    // Check error message
    let stderr = str::from_utf8(&output.stderr)?;
    assert!(
        stderr.contains("Invalid source"),
        "Expected 'Invalid source' error message, got: {}",
        stderr
    );

    Ok(())
}

#[test]
#[ignore]
fn test_batch_fetch_cli_missing_required_input() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "/home/hanbo/repo/stock_trading_bot/src/configs/data_ingestor.toml";

    let output =
        Command::new("/home/hanbo/repo/stock_trading_bot/src/target/debug/market_data_ingestor")
            .args([
                "--config",
                config_path,
                "batch",
                "--source",
                "file",
                // Missing --input argument
            ])
            .output()?;

    // This should fail
    assert!(
        !output.status.success(),
        "Binary should have failed with missing input"
    );

    // Check error message
    let stderr = str::from_utf8(&output.stderr)?;
    assert!(
        stderr.contains("File path required for source=file"),
        "Expected 'File path required' error message, got: {}",
        stderr
    );

    Ok(())
}
