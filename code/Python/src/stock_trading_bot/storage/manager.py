from datetime import datetime
from typing import Protocol
import os
from pathlib import Path

import duckdb
from loguru import logger

from stock_trading_bot.utils import load_config_auto
from stock_trading_bot.storage import db_utils

class StorageManager(Protocol):

    def add_data(self, data, table_name: str) -> None:
        """Add new data to the storage system."""
        pass

    def calculate_higher_granularity(self, base_table_name: str, new_table_name: str, granularity: str) -> None:
        """Calculate higher granularity data from base data."""
        pass

    def __enter__(self):
        """Enter the storage manager context."""
        pass

    def __exit__(self, exc_type, exc_value, traceback):
        """Exit the storage manager context."""
        pass


class DuckDBManager(StorageManager):
    def __init__(self):
        config = load_config_auto()
        self.data_directory = Path(config['data_directory'])
        self.db_directory = self.data_directory / 'db'
        self.max_snapshots = config.get('max_snapshots', 7)  # Default to 7 days of snapshots
        self.snapshot_directory = self.db_directory / 'snapshots'

        self.snapshot_directory.mkdir(parents=True, exist_ok=True)

        # Delayed connection setup and snapshot preparation
        self.connection = None

    def __enter__(self):
        db_utils.prepare_and_connect_to_latest_snapshot()
        return self
    
    def __exit__(self, exc_type, exc_value, traceback):
        if self.connection:
            self.connection.close()
        db_utils.snapshot_database()
        db_utils.cleanup_snapshots()

    def add_data(self, data, table_name):
        ...

    def calculate_higher_granularity(self, base_table_name, new_table_name, granularity):
        ...

    
