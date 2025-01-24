# test_duckdb_vs_sqlite.py

from contextlib import contextmanager
from pathlib import Path
import shutil
import time
import random
import string

import duckdb
import sqlite3
import numpy as np
import pandas as pd
import polars as pl
import plotly.graph_objects as go
from plotly.subplots import make_subplots

TEMP_DIR = Path("temp") / "duckdb_vs_sqlite_benchmark"


@contextmanager
def timer():
    start = time.time()
    yield lambda: time.time() - start


class DBBenchmark:
    def __init__(self, num_symbols=100, num_rows=100000, num_repeats=5):
        self.num_symbols = num_symbols
        self.num_rows = num_rows
        self.num_repeats = num_repeats
        self.stock_symbols = self._generate_symbols(num_symbols)
        self.timeframes = ["1min", "5min", "1hour", "1day"]
        self.results = []

    def _generate_symbols(self, count):
        symbols = set()
        while len(symbols) < count:
            symbol = "".join(random.choices(string.ascii_uppercase, k=4))
            symbols.add(symbol)
        return list(symbols)

    def generate_synthetic_data(self):
        start_time = pl.datetime(2020, 1, 1)
        end_time = start_time + pl.duration(minutes=self.num_rows)

        timestamps = pl.datetime_range(start=start_time, end=end_time, interval="1m")
        size = pl.select(timestamps).height

        # data = pd.DataFrame({
        #     'timestamp': np.tile(timestamps, len(self.stock_symbols)),
        #     'open': np.random.rand(self.num_rows * len(self.stock_symbols)),
        #     'high': np.random.rand(self.num_rows * len(self.stock_symbols)),
        #     'low': np.random.rand(self.num_rows * len(self.stock_symbols)),
        #     'close': np.random.rand(self.num_rows * len(self.stock_symbols)),
        #     'volume': np.random.randint(1, 1000, self.num_rows * len(self.stock_symbols)),
        #     'symbol': np.repeat(self.stock_symbols, self.num_rows)
        # })
        data = pl.select(timestamps.alias("timestamp")).with_columns(
            pl.Series("open", np.random.rand(size).tolist()),
            pl.Series("high", np.random.rand(size).tolist()),
            pl.Series("low", np.random.rand(size).tolist()),
            pl.Series("close", np.random.rand(size).tolist()),
            pl.Series("volume", np.random.rand(size).tolist()),
        )

        self.data = data

    def cleanup(self):
        if TEMP_DIR.exists():
            shutil.rmtree(TEMP_DIR)

    def benchmark_duckdb(self):
        db_path = TEMP_DIR / "duckdb_benchmark.duckdb"
        conn = duckdb.connect(str(db_path))
        conn.execute("PRAGMA threads=4")

        # Create table
        with timer() as create_timer:
            conn.execute("""
                CREATE TABLE stock_data (
                    timestamp TIMESTAMP,
                    open DOUBLE,
                    high DOUBLE,
                    low DOUBLE,
                    close DOUBLE,
                    volume INTEGER
                )
            """)
        create_time = create_timer()

        # Insert data
        with timer() as insert_timer:
            conn.register("df", self.data)
            conn.execute("INSERT INTO stock_data SELECT * FROM df")
        insert_time = insert_timer()

        # Query data
        with timer() as query_timer:
            for symbol in self.stock_symbols[:10]:
                conn.execute("SELECT * FROM stock_data WHERE timestamp >= '2020-06-01'")
                conn.fetchall()
        query_time = query_timer()

        conn.close()
        return {
            "db": "DuckDB",
            "create_time": create_time,
            "insert_time": insert_time,
            "query_time": query_time,
        }

    def benchmark_sqlite(self):
        db_path = TEMP_DIR / "sqlite_benchmark.sqlite"
        conn = sqlite3.connect(str(db_path))
        conn_uri = f"sqlite:///{db_path}"

        # Create table
        with timer() as create_timer:
            conn.execute("""
                CREATE TABLE stock_data (
                    timestamp TEXT,
                    open REAL,
                    high REAL,
                    low REAL,
                    close REAL,
                    volume INTEGER
                )
            """)
        create_time = create_timer()

        # Insert data
        with timer() as insert_timer:
            self.data.write_database("stock_data", conn_uri, if_table_exists="append")
        insert_time = insert_timer()

        # Query data
        with timer() as query_timer:
            cursor = conn.cursor()
            for symbol in self.stock_symbols[:10]:
                cursor.execute(
                    "SELECT * FROM stock_data WHERE timestamp >= '2020-06-01'"
                )
                cursor.fetchall()
        query_time = query_timer()

        conn.close()
        return {
            "db": "SQLite",
            "create_time": create_time,
            "insert_time": insert_time,
            "query_time": query_time,
        }

    def run_benchmarks(self):
        self.cleanup()
        TEMP_DIR.mkdir(parents=True, exist_ok=True)
        self.generate_synthetic_data()

        for _ in range(self.num_repeats):
            duckdb_result = self.benchmark_duckdb()
            sqlite_result = self.benchmark_sqlite()
            self.results.extend([duckdb_result, sqlite_result])

            # Clean up databases after each run
            self.cleanup()
            TEMP_DIR.mkdir(parents=True, exist_ok=True)

    def analyze_results(self):
        df = pd.DataFrame(self.results)
        avg_results = df.groupby("db").mean().reset_index()

        print("\nBenchmark Results (Average over runs):")
        print(avg_results)

        self.avg_results = avg_results

    def visualize_results(self):
        # Visualize avg results
        df = pd.DataFrame(self.avg_results)
        fig = make_subplots(
            rows=1, cols=3, subplot_titles=("Create Time", "Insert Time", "Query Time")
        )

        metrics = ["create_time", "insert_time", "query_time"]

        for i, metric in enumerate(metrics, 1):
            fig.add_trace(
                go.Bar(x=df["db"], y=df[metric], name=metric.capitalize()), row=1, col=i
            )

        fig.update_layout(title="DuckDB vs SQLite Benchmark Results", showlegend=False)
        results_dir = Path("tests/benchmarks/results")
        results_dir.mkdir(parents=True, exist_ok=True)
        fig.write_html(results_dir / "duckdb_vs_sqlite_benchmark.html")
        print(
            f"Benchmark visualization saved to {results_dir}/duckdb_vs_sqlite_benchmark.html"
        )


if __name__ == "__main__":
    benchmark = DBBenchmark(num_symbols=100, num_rows=100000, num_repeats=3)
    benchmark.run_benchmarks()
    benchmark.analyze_results()
    benchmark.visualize_results()
