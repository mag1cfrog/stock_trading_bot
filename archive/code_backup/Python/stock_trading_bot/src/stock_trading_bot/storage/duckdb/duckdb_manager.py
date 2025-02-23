from datetime import datetime

import os
from pathlib import Path

import duckdb
from loguru import logger

import polars as pl

from stock_trading_bot.utils import load_config_auto, validate_pl_df
from stock_trading_bot.storage.duckdb import db_utils
from stock_trading_bot.storage.protocols import StorageManager


class DuckDBManager(StorageManager):
    def __init__(self, config_path: Path = None):
        config = load_config_auto(config_path)
        self.base_granularity = (
            config["data_granularity"]["base_amount"],
            config["data_granularity"]["base_unit"],
        )
        self.data_directory = Path(config["data_directory"])
        self.db_directory = self.data_directory / "db"
        self.max_snapshots = config.get(
            "max_snapshots", 7
        )  # Default to 7 days of snapshots
        self.snapshot_directory = self.db_directory / "snapshots"

        self.snapshot_directory.mkdir(parents=True, exist_ok=True)

        # Delayed connection setup and snapshot preparation
        self.connection = None

    def __enter__(self):
        """
        Use the utils function to retrieve the latest snapshot if available, and then connect to it.
        If no snapshot is available, create a new connection to the database.
        """
        self.connection = db_utils.prepare_and_connect_to_latest_snapshot(
            self.db_directory, self.snapshot_directory
        )
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        """
        Close the connection and snapshot the database.
        If the number of snapshots exceeds the maximum, clean up the oldest snapshot.
        """
        if self.connection:
            self.connection.close()
        db_utils.snapshot_database(self.db_directory, self.snapshot_directory)
        db_utils.cleanup_snapshots(self.db_directory, self.max_snapshots)

    def initialize_base_level_table(self):
        """
        Create the lowest granularity table for storing stock data.
        """
        sql_query_create_table = f"""
            CREATE TABLE IF NOT EXISTS base_level_table (
                {db_utils.BASE_LEVEL_TABLE_SCHEMA}
            )
        """
        logger.trace(
            f"Creating base level table with schema: {db_utils.BASE_LEVEL_TABLE_SCHEMA}"
        )
        self.connection.execute(sql_query_create_table)

    def add_stock_data(self, data: pl.DataFrame, table_name: str):
        """
        Add a polars DataFrame to the DuckDB database.

        """

        # First examine the polars DataFrame schema to ensure it matches the base level table schema
        validate_pl_df(data)

        sql_query_insert_data = f"""
            INSERT INTO {table_name}
                SELECT * FROM data d 
            WHERE NOT EXISTS (
                SELECT 1 FROM {table_name} t
                WHERE 
                    t.timestamp = d.timestamp
                    AND t.symbol = d.symbol
            )

        """
        logger.trace(f"Inserting data into table {table_name}")
        logger.catch(self.connection.execute(sql_query_insert_data))

    def check_database_health(self) -> bool:
        """
        Check the health or connectivity of the database.
        """
        try:
            self.connection.execute("SELECT 1")
            logger.trace("Database connection is healthy")
            return True
        except Exception as e:
            logger.error(f"Database health check failed: {str(e)}")
            return False
