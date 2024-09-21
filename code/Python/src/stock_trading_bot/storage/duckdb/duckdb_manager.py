from datetime import datetime

import os
from pathlib import Path

import duckdb
from loguru import logger
import pandera.polars as pa
import polars as pl

from stock_trading_bot.utils import load_config_auto
from stock_trading_bot.storage import db_utils
from stock_trading_bot.storage.protocols import StorageManager



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
        """
        Use the utils function to retrieve the latest snapshot if available, and then connect to it.
        If no snapshot is available, create a new connection to the database.
        """
        self.connection = db_utils.prepare_and_connect_to_latest_snapshot()
        return self
    
    def __exit__(self, exc_type, exc_value, traceback):
        """
        Close the connection and snapshot the database.
        If the number of snapshots exceeds the maximum, clean up the oldest snapshot.
        """
        if self.connection:
            self.connection.close()
        db_utils.snapshot_database()
        db_utils.cleanup_snapshots()

    def initialize_base_level_table(self):
        """
        Create the lowest granularity table for storing stock data.
        """
        sql_query_create_table = f"""
            CREATE TABLE db.base_level_table (
                {db_utils.BASE_LEVEL_TABLE_SCHEMA}
            )
        """
        logger.trace(f"Creating base level table with schema: {db_utils.BASE_LEVEL_TABLE_SCHEMA}")
        self.connection.execute(sql_query_create_table)

    def add_stock_data(self, data:pl.DataFrame , table_name: str):
        """
        Add a polars DataFrame to the DuckDB database.

        """

        # First examine the polars DataFrame schema to ensure it matches the base level table schema

        non_negative_checks = [
            pa.Check.greater_than_or_equal_to(0)
        ]
        schema = pa.DataFrameSchema(
            columns={
                'symbol': pa.Column(pa.String),
                'timestamp': pa.Column(pl.Datetime(time_unit='ns', time_zone='UTC')),
                'open': pa.Column(pa.Float, checks=non_negative_checks),
                'high': pa.Column(pa.Float, checks=non_negative_checks),
                'low': pa.Column(pa.Float, checks=non_negative_checks),
                'close': pa.Column(pa.Float, checks=non_negative_checks),
                'volume': pa.Column(pa.Int, checks=non_negative_checks, coerce=True),
                'trade_count': pa.Column(pa.Int, checks=non_negative_checks, coerce=True),
                'vwap': pa.Column(pa.Float, checks=non_negative_checks),
            }
        )

        logger.trace(f"Validating DataFrame schema")
        logger.exception(schema.validate(data))


        sql_query_insert_data = f"""
            INSERT INTO db.{table_name}
                SELECT * FROM {data}
            WHERE NOT EXISTS (
                SELECT 1 FROM db.{table_name}
                WHERE 
                    db.{table_name}.timestamp = {data}.timestamp
                    AND db.{table_name}.symbol = {data}.symbol
            )

        """
        logger.trace(f"Inserting data into table {table_name}")
        logger.exception(self.connection.execute(sql_query_insert_data))
        

    # def calculate_higher_granularity(self, base_table_name, new_table_name, granularity):
    #     ...
    
