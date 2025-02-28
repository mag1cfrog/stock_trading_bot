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
