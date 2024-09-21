from datetime import datetime
from pathlib import Path
import shutil

import duckdb
from loguru import logger


BASE_LEVEL_TABLE_SCHEMA = """
symbol STRING,
timestamp TIMESTAMP,
open FLOAT64,
high FLOAT64,
low FLOAT64,
close FLOAT64,
volume INT64,
trade_count INT64,
vwap FLOAT64,
"""


def prepare_and_connect_to_latest_snapshot(db_directory: Path, snapshot_directory: Path) -> duckdb.DuckDBPyConnection:
    """
    Prepare the working environment by selecting and connecting to the latest snapshot.
    
    Args:
        db_directory (Path): The directory where the database is stored.
        snapshot_directory (Path): The directory where snapshots are stored.
    
    Returns:
        duckdb.DuckDBPyConnection: The connection to the DuckDB database.

    """
    latest_snapshot = get_latest_snapshot(snapshot_directory)
    working_db_path = db_directory / "temp_dir" / "stock_data.duckdb"

    if latest_snapshot:
        logger.debug(f"Found the latest snapshot: {latest_snapshot}")
        logger.trace(f"Copying {latest_snapshot} to {working_db_path}")
        shutil.copy2(latest_snapshot, working_db_path)
        logger.info(f"Using latest snapshot: {latest_snapshot}")
    else:
        logger.warning("No snapshot found, initializing a new database.")
    
    logger.trace(f"Connecting to {working_db_path}")
    return duckdb.connect(str(working_db_path))


def get_latest_snapshot(snapshot_directory) -> Path | None:
    """
    Retrieve the path to the most recent snapshot file from the snapshot directory based on the last modified time, if any.

    Args:
        snapshot_directory (Path): The directory where snapshots are stored.
    
    Returns:
        Path | None: The path to the latest snapshot file, or None if no snapshots are found.
    """
    snapshots = list(snapshot_directory.glob("*.duckdb"))
    if snapshots:
        latest_snapshot = max(snapshots, key=lambda x: x.stat().st_mtime)
        return latest_snapshot
    return None


def snapshot_database(db_directory: Path, snapshot_directory: Path) -> None:
    """Create a snapshot of the current DuckDB file."""
    current_time = datetime.now().strftime("%Y%m%d_%H%M%S")
    snapshot_path = snapshot_directory / f"stock_data_{current_time}.duckdb"
    current_db_path = db_directory / "temp_dir"/ "stock_data.duckdb"
    try:
        shutil.copy2(current_db_path, snapshot_path)
        logger.debug(f"Snapshot saved to {snapshot_path}")
    except Exception as e:
        logger.error(f"Error creating snapshot: {e}")


def cleanup_snapshots(snapshot_directory: Path, max_snapshots: int) -> None:
    """Remove old snapshots beyond the maximum retention number."""
    snapshots = list(snapshot_directory.glob("*.duckdb"))
    snapshots.sort(key=lambda x: x.stat().st_mtime)
    while len(snapshots) > max_snapshots:
        try:
            snapshots[0].unlink()
            logger.debug(f"Removed old snapshot: {snapshots[0]}")
            snapshots.pop(0)
        except Exception as e:
            logger.error(f"Failed to remove snapshot {snapshots[0]}: {e}")