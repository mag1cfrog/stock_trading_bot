use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::{env, fs};

use chrono::Utc;
use polars::frame::DataFrame;
use polars_io::SerWriter;
use polars_io::ipc::IpcWriter;
use uuid::Uuid;

pub fn write_dataframe_to_temp(
    df: &mut DataFrame,
    symbol: &str,
) -> Result<PathBuf, Box<dyn Error>> {
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
    let filename = format!("{}_{}_{}.feather", symbol, timestamp, Uuid::new_v4());
    let mut output_path = base_temp.clone();
    output_path.push(filename);

    // Create file and write the DataFrame using Polars' IPC writer (compatible with Arrow/Feather).
    let mut file = File::create(&output_path)?;
    let mut writer = IpcWriter::new(&mut file);
    writer.finish(df)?;

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use std::fs;

    #[test]
    fn test_write_dataframe_to_temp() {
        // Create a simple DataFrame
        let df = DataFrame::new(vec![
            Series::new("symbol".into(), &["AAPL"]).into(),
            Series::new("close".into(), &[150.0f64]).into(),
            Series::new("volume".into(), &[1000000i64]).into(),
        ])
        .unwrap();

        let mut df_mut = df.clone();

        // Test writing to temp file
        let output_path = write_dataframe_to_temp(&mut df_mut, "AAPL").unwrap();

        // Verify file exists
        assert!(output_path.exists());

        // Verify file can be read back
        let read_df = IpcReader::new(File::open(output_path.clone()).unwrap())
            .finish()
            .unwrap();

        // Check that dimensions match
        assert_eq!(read_df.shape(), df.shape());

        // Clean up
        fs::remove_file(output_path).unwrap();
    }
}
