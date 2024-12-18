from contextlib import contextmanager
from itertools import product
from pathlib import Path
import shutil
import time
import random
import string

import duckdb
import numpy as np
import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots
import polars as pl
# from viztracer import VizTracer
import yaml

TEMP_DIR = Path("temp") / Path("storage_design_benchmark")

config_path = Path("tests/benchmarks/bm_sdp.yaml")

with open(config_path, 'r') as file:
    config = yaml.safe_load(file)

SYMBOL_COUNTS = config.get('SYMBOL_COUNTS', [1, 100])
ROW_COUNTS = config.get('ROW_COUNTS', [10000])
APPEND_BATCH_NUMBER = config.get('APPEND_BATCH_NUMBER', 100)
NUM_REPEATS = config.get('NUM_REPEATS', 5)


@contextmanager
def timer():
    start = time.perf_counter()
    yield lambda: time.perf_counter() - start

class StorageDesignBenchmark:
    def __init__(self, symbol_counts: list=SYMBOL_COUNTS, row_counts: list=ROW_COUNTS, append_batch_number: int=APPEND_BATCH_NUMBER, num_repeats: int=NUM_REPEATS):
        self.symbol_counts = symbol_counts
        self.row_counts = row_counts
        self.timeframes = ['1min', '5min', '1hour', '1day']
        self.initial_data = None
        self.append_data = None
        self.append_batch_number = append_batch_number
        self.results = {}
        self.num_repeats = num_repeats

    def _generate_symbols(self, count):
        """Generate random unique symbol names of 4 characters each."""
        symbols = set()
        while len(symbols) < count:
            symbol = ''.join(random.choices(string.ascii_uppercase, k=4))
            symbols.add(symbol)
        return list(symbols)

    def generate_synthetic_data(self):
        # Initial data (smaller dataset for table creation)
        initial_timestamps = pl.datetime_range(start=pl.datetime(2020, 1, 1), end=pl.datetime(2020, 1, 2), interval='1m', time_unit='ms',
            closed='left')
        self.initial_data = self._create_dataframe(initial_timestamps)
        
        # Create multiple smaller append batches
        self.append_batches = []
        batch_size = self.num_rows // self.append_batch_number
        for i in range(self.append_batch_number):
            start_date = pl.datetime(2020, 4, 1) + pl.duration(days=i*7)
            end_date = start_date + pl.duration(minutes=batch_size)
            batch_timestamps = pl.datetime_range(
                start=start_date,
                end=end_date,
                interval='1m',
                time_unit='ms',closed='left'
            )
            batch_data = self._create_dataframe(batch_timestamps)
            self.append_batches.append(batch_data)

    def _create_dataframe(self, timestamps):
        size = pl.select(timestamps).height
        return pl.select(timestamps.alias('timestamp')).with_columns(
            pl.Series('open', np.random.rand(size).tolist()),
            pl.Series('high', np.random.rand(size).tolist()),
            pl.Series('low', np.random.rand(size).tolist()),
            pl.Series('close', np.random.rand(size).tolist()),
            pl.Series('volume', np.random.rand(size).tolist())
        )

    def cleanup(self):
        if TEMP_DIR.exists():
            shutil.rmtree(TEMP_DIR)

    def benchmark_option_1(self):

        db_path = TEMP_DIR / 'option1_all_data.duckdb'

        conn = duckdb.connect(str(db_path))
        
        # Initial table creation
        with timer() as create_timer:
            for symbol in self.stock_symbols:
                for timeframe in self.timeframes:
                    temp_data = self.initial_data
                    conn.execute(f"CREATE TABLE {symbol}_{timeframe} AS SELECT * FROM temp_data")
        create_time = create_timer()

        # Test multiple append operations
        append_times = []
        with timer() as total_append_timer:
            for batch_idx, batch_data in enumerate(self.append_batches):
                with timer() as batch_timer:
                    for symbol in self.stock_symbols:
                        for timeframe in self.timeframes:
                            append_data = batch_data
                            conn.execute(f"INSERT INTO {symbol}_{timeframe} SELECT * FROM append_data")
                append_times.append(batch_timer())
        total_append_time = total_append_timer()

        # Test query performance after appends
        with timer() as query_timer:
            for symbol in self.stock_symbols:
                for timeframe in self.timeframes:
                    conn.execute(f"SELECT * FROM {symbol}_{timeframe} WHERE timestamp >= '2020-06-01'")
        query_time = query_timer()
        
        conn.close()
        return {
            'create_time': create_time,
            'append_time': total_append_time,
            'append_time_per_batch': append_times,
            'query_time': query_time
        }

    def benchmark_option_2(self):
        # Initial table creation
        with timer() as create_timer:
            for symbol in self.stock_symbols:
                for timeframe in self.timeframes:
                    db_path = TEMP_DIR / f"option2_data/{symbol}/{timeframe}"
                    db_path.mkdir(parents=True, exist_ok=True)
                    conn = duckdb.connect(str(db_path / 'data.duckdb'))
                    temp_data = self.initial_data
                    conn.execute("CREATE TABLE data AS SELECT * FROM temp_data")
                    conn.close()
        create_time = create_timer()

        # Test multiple append operations
        append_times = []
        with timer() as total_append_timer:
            for batch_idx, batch_data in enumerate(self.append_batches):
                with timer() as batch_timer:
                    for symbol in self.stock_symbols:
                        for timeframe in self.timeframes:
                            conn = duckdb.connect(str(db_path / 'data.duckdb'))
                            append_data = batch_data
                            conn.execute("INSERT INTO data SELECT * FROM append_data")
                            conn.close()
                append_times.append(batch_timer())
        total_append_time = total_append_timer()

        # Test query performance after appends
        with timer() as query_timer:
            for symbol in self.stock_symbols:
                for timeframe in self.timeframes:
                    conn = duckdb.connect(str(db_path / 'data.duckdb'))
                    conn.execute("SELECT * FROM data WHERE timestamp >= '2020-06-01'")
                    conn.close()
        query_time = query_timer()

        return {
            'create_time': create_time,
            'append_time': total_append_time,
            'append_time_per_batch': append_times,
            'query_time': query_time
        }

    def benchmark_option_3(self):
        # Initial table creation
        with timer() as create_timer:
            for symbol in self.stock_symbols:
                db_path = TEMP_DIR / f"option3_{symbol}.duckdb"
                conn = duckdb.connect(str(db_path))
                temp_data = self.initial_data
                for timeframe in self.timeframes:
                    conn.execute(f"CREATE TABLE data_{timeframe} AS SELECT * FROM temp_data")
                conn.close()
        create_time = create_timer()

        # Test multiple append operations
        append_times = []
        with timer() as total_append_timer:
            for batch_idx, batch_data in enumerate(self.append_batches):
                with timer() as batch_timer:
                    for symbol in self.stock_symbols:
                        db_path = TEMP_DIR / f"option3_{symbol}.duckdb"
                        conn = duckdb.connect(str(db_path))
                        append_data = batch_data
                        for timeframe in self.timeframes:
                            conn.execute(f"INSERT INTO data_{timeframe} SELECT * FROM append_data")
                        conn.close()
                append_times.append(batch_timer())
        total_append_time = total_append_timer()

        # Test query performance after appends
        with timer() as query_timer:
            for symbol in self.stock_symbols:
                conn = duckdb.connect(str(db_path))
                for timeframe in self.timeframes:
                    conn.execute(f"SELECT * FROM data_{timeframe} WHERE timestamp >= '2020-06-01'")
                conn.close()
        query_time = query_timer()

        return {
            'create_time': create_time,
            'append_time': total_append_time,
            'append_time_per_batch': append_times,
            'query_time': query_time
        }

    def run_comprehensive_benchmarks(self):

        # Initialize temp directory
        self.cleanup()
        TEMP_DIR.mkdir(parents=True, exist_ok=True)

        # Test different combinations of symbols and data sizes
        symbol_counts = self.symbol_counts  # Different number of symbols
        row_counts = self.row_counts  # Different data sizes
        scenarios = list(product(symbol_counts, row_counts))
        

        results = {}
        print("Starting comprehensive benchmarks...")
        
        for num_symbols, num_rows in scenarios:
            print(f"\nScenario: {num_symbols} symbols, {num_rows:,} rows")
            scenario_key = f"symbols_{num_symbols}_rows_{num_rows}"
            scenario_results = {f"option_{i}": {'create_times': [], 'append_times': [], 'append_times_per_batch': [], 'query_times': [], 'total_times': []} for i in range(1, 4)}

            for repeat in range(self.num_repeats):
                print(f"  Repeat {repeat + 1}/{self.num_repeats}")
                # Reinitialize with new parameters
                self.num_rows = num_rows
                self.stock_symbols = self._generate_symbols(num_symbols)
                self.generate_synthetic_data()

                self.cleanup()
                TEMP_DIR.mkdir(exist_ok=True)

                for option in range(1, 4):
                    benchmark_func = getattr(self, f"benchmark_option_{option}")
                    option_results = benchmark_func()

                    # Store individual timings
                    scenario_results[f"option_{option}"]['create_times'].append(option_results['create_time'])
                    scenario_results[f"option_{option}"]['append_times'].append(option_results['append_time'])
                    scenario_results[f"option_{option}"]['query_times'].append(option_results['query_time'])
                    total_time = option_results['create_time'] + option_results['append_time'] + option_results['query_time']
                    scenario_results[f"option_{option}"]['total_times'].append(total_time)

                    # Store append times per batch
                    scenario_results[f"option_{option}"]['append_times_per_batch'].append(option_results['append_time_per_batch'])

            # After repeats, compute averages
            for option in range(1, 4):
                for metric in ['create_times', 'append_times', 'query_times', 'total_times']:
                    times = scenario_results[f"option_{option}"][metric]
                    avg_time = sum(times) / len(times)
                    std_time = (sum((x - avg_time) ** 2 for x in times) / len(times)) ** 0.5
                    scenario_results[f"option_{option}"][f"avg_{metric[:-1]}"] = avg_time
                    scenario_results[f"option_{option}"][f"std_{metric[:-1]}"] = std_time

            results[scenario_key] = scenario_results
            
        self.cleanup()
        self.results = results  # Store results for analysis and visualization
        return results

    def analyze_results(self, results=None):
        if results is None:
            results = self.results
        print("\nPerformance Analysis:")
        print("=" * 80)

        for scenario, scenario_results in results.items():
            num_symbols = int(scenario.split('_')[1])
            num_rows = int(scenario.split('_')[3])

            print(f"\nScenario: {num_symbols} symbols, {num_rows:,} rows")
            print("-" * 50)

            for option in range(1, 4):
                opt_results = scenario_results[f"option_{option}"]
                print(f"\nOption {option}:")
                print(f"  Create Time: {opt_results['avg_create_time']:.2f}s ± {opt_results['std_create_time']:.2f}s")
                print(f"  Append Time: {opt_results['avg_append_time']:.2f}s ± {opt_results['std_append_time']:.2f}s")
                print(f"  Query Time: {opt_results['avg_query_time']:.2f}s ± {opt_results['std_query_time']:.2f}s")
                print(f"  Total Time: {opt_results['avg_total_time']:.2f}s ± {opt_results['std_total_time']:.2f}s")

    
    def visualize_results(self, results):
        """Create 2D visualizations for benchmark results using plotly."""
        results_dir = Path("tests/benchmarks/results")
        results_dir.mkdir(parents=True, exist_ok=True)

        # Convert results to DataFrame
        rows = []
        for scenario, scenario_results in results.items():
            num_symbols = int(scenario.split('_')[1])
            num_rows = int(scenario.split('_')[3])
            for option in range(1, 4):
                opt_results = scenario_results[f"option_{option}"]
                rows.append({
                    'num_symbols': num_symbols,
                    'num_rows': num_rows,
                    'option': f'Option {option}',
                    'create_time': opt_results['avg_create_time'],
                    'append_time': opt_results['avg_append_time'],
                    'query_time': opt_results['avg_query_time'],
                    'total_time': opt_results['avg_total_time'],
                })
        
        df = pd.DataFrame(rows)
        metrics = {
            'create_time': 'Create Time (s)',
            'append_time': 'Append Time (s)',
            'query_time': 'Query Time (s)',
            'total_time': 'Total Time (s)'
        }

        for metric, title in metrics.items():
            fig = make_subplots(
                rows=2, cols=2,
                subplot_titles=[f'{n} Symbols' for n in sorted(df['num_symbols'].unique())],
                x_title='Number of Rows',
                y_title='Time (seconds)'
            )

            row_col_pairs = [(1,1), (1,2), (2,1), (2,2)]
            colors = ['blue', 'red', 'green']
            
            for i, num_symbols in enumerate(sorted(df['num_symbols'].unique())):
                row, col = row_col_pairs[i]
                symbol_df = df[df['num_symbols'] == num_symbols]
                
                for option, color in zip(range(1, 4), colors):
                    option_df = symbol_df[symbol_df['option'] == f'Option {option}']
                    fig.add_trace(
                        go.Scatter(
                            x=option_df['num_rows'],
                            y=option_df[metric],
                            name=f'Option {option}',
                            line=dict(color=color),
                            hovertemplate=(
                                f'Rows: %{{x}}<br>'
                                f'Time: %{{y:.2f}}s<br>'
                                f'Option {option}'
                            ),
                        ),
                        row=row,
                        col=col,
                    )

            fig.update_layout(
                title=title,
                height=800,
                width=1200,
                showlegend=True,
                legend=dict(
                    yanchor="top",
                    y=0.99,
                    xanchor="left",
                    x=0.01
                ),
            )

            # Save each metric's plot separately
            fig.write_html(results_dir / f"benchmark_results_{metric}.html")
            print(f"Benchmark visualization for {title} saved to {results_dir}/benchmark_results_{metric}.html")

        fig_batches = go.Figure()
    
        for option in range(1, 4):
            batch_times = []
            for scenario, scenario_results in results.items():
                option_results = scenario_results[f"option_{option}"]
                # append_times_per_batch is a list of lists (one list per repeat)
                append_times_per_batch = option_results.get('append_times_per_batch', [])
                for per_repeat_batch_times in append_times_per_batch:
                    batch_times.extend(per_repeat_batch_times)  # Flatten the list

            fig_batches.add_trace(go.Box(
                y=batch_times,
                name=f'Option {option}',
                boxpoints='all',
                jitter=0.3,
                pointpos=-1.8
            ))

        fig_batches.update_layout(
            title='Append Performance Distribution Across Batches',
            yaxis_title='Time per Batch (seconds)',
            showlegend=True
        )
        
        fig_batches.write_html(results_dir / "benchmark_results_append_batches.html")
        print(f"Append batch performance visualization saved to {results_dir}/benchmark_results_append_batches.html")

if __name__ == "__main__":
    # tracer = VizTracer(ignore_c_function=True, exclude_files=["*/duckdb/*"])
    # tracer.start()
    benchmark = StorageDesignBenchmark() 
    results = benchmark.run_comprehensive_benchmarks()
    benchmark.analyze_results(results)
    benchmark.visualize_results(results)
    # tracer.stop()
    # tracer.save("tests/benchmarks/results/benchmark_results.json")