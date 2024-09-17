from typing import Protocol
import os

import duckdb

from stock_trading_bot.utils import load_config_auto

class StorageManager(Protocol):
    def initialize(self) -> None:
        """Initialize the storage system, setting up any necessary tables or files."""
        pass

    def add_data(self, data, table_name: str) -> None:
        """Add new data to the storage system."""
        pass

    def calculate_higher_granularity(self, base_table_name: str, new_table_name: str, granularity: str) -> None:
        """Calculate higher granularity data from base data."""
        pass

class DuckDBManager:
    def __init__(self):
        config = load_config_auto()
        self.data_directory = config['data_directory']
        self.db_directory = os.path.join(self.data_directory, 'db')


    def add_data(self, data, table_name):
        ...

    def calculate_higher_granularity(self, base_table_name, new_table_name, granularity):
        ...