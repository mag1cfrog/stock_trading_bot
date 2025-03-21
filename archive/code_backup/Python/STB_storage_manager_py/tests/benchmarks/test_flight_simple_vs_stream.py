import io
import pathlib
from typing import Dict, Union
import pyarrow as pa
import pyarrow.flight
import pyarrow.parquet

TOTAL_ROWS = 200000
TOTAL_COLUMNS = 1000


class InMemoryFileSystem:
    def __init__(self):
        self.files: Dict[str, io.BytesIO] = {}

    def open(self, path: str, mode: str = "rb") -> io.BytesIO:
        if mode.startswith("w"):
            self.files[path] = io.BytesIO()
            return self.files[path]
        return self.files.get(path, io.BytesIO())

    def exists(self, path: str) -> bool:
        return path in self.files

    def unlink(self, path: str) -> None:
        if path in self.files:
            del self.files[path]

    def iterdir(self):
        return [pathlib.Path(name) for name in self.files.keys()]


class SimpleFlightServer(pa.flight.FlightServerBase):
    def __init__(self, location="grpc://0.0.0.0:8815", **kwargs):
        super(SimpleFlightServer, self).__init__(location, **kwargs)
        self._location = location
        self._fs = InMemoryFileSystem()

    def _make_flight_info(self, dataset):
        buffer = self._fs.open(dataset)
        schema = pa.parquet.read_schema(buffer)
        buffer.seek(0)
        metadata = pa.parquet.read_metadata(buffer)
        descriptor = pa.flight.FlightDescriptor.for_path(dataset.encode("utf-8"))
        endpoints = [pa.flight.FlightEndpoint(dataset, [self._location])]
        return pa.flight.FlightInfo(
            schema, descriptor, endpoints, metadata.num_rows, metadata.serialized_size
        )

    def list_flights(self, context, criteria):
        for dataset in self._fs.iterdir():
            yield self._make_flight_info(dataset.name)

    def get_flight_info(self, context, descriptor):
        return self._make_flight_info(descriptor.path[0].decode("utf-8"))

    def do_put(self, context, descriptor, reader, writer):
        dataset = descriptor.path[0].decode("utf-8")
        buffer = self._fs.open(dataset, "wb")
        data_table = reader.read_all()
        pa.parquet.write_table(data_table, buffer)

    def do_get(self, context, ticket):
        dataset = ticket.ticket.decode("utf-8")
        buffer = self._fs.open(dataset)
        return pa.flight.RecordBatchStream(pa.parquet.read_table(buffer))

    def do_drop_dataset(self, dataset):
        self._fs.unlink(dataset)


class streamFlightServer(pa.flight.FlightServerBase):
    def __init__(self, location="grpc://0.0.0.0:8815", **kwargs):
        super(streamFlightServer, self).__init__(location, **kwargs)
        self._location = location
        self._fs = InMemoryFileSystem()

    def _make_flight_info(self, dataset):
        buffer = self._fs.open(dataset)
        schema = pa.parquet.read_schema(buffer)
        buffer.seek(0)
        metadata = pa.parquet.read_metadata(buffer)
        descriptor = pa.flight.FlightDescriptor.for_path(dataset.encode("utf-8"))
        endpoints = [pa.flight.FlightEndpoint(dataset, [self._location])]
        return pa.flight.FlightInfo(
            schema, descriptor, endpoints, metadata.num_rows, metadata.serialized_size
        )

    def list_flights(self, context, criteria):
        for dataset in self._fs.iterdir():
            yield self._make_flight_info(dataset.name)

    def get_flight_info(self, context, descriptor):
        return self._make_flight_info(descriptor.path[0].decode("utf-8"))

    def do_put(self, context, descriptor, reader, writer):
        dataset = descriptor.path[0].decode("utf-8")
        sink = self._fs.open(dataset, "wb")
        with pa.parquet.ParquetWriter(sink, reader.schema) as writer:
            for batch in reader:
                writer.write_table(pa.Table.from_batches([batch.data]))

    def do_get(self, context, ticket):
        dataset = ticket.ticket.decode("utf-8")
        # Stream data from a file
        sink = self._fs.open(dataset, "rb")
        pqfile = pa.parquet.ParquetFile(sink)
        return pa.flight.GeneratorStream(pqfile.schema_arrow, pqfile.iter_batches())

    def do_drop_dataset(self, dataset):
        self._fs.unlink(dataset)


def generate_arrow_table_with_row_column_number(
    row_number: int, column_number: int
) -> pa.Table:
    """
    Generate an Arrow table with row_number rows and column_number columns, filled with random values.
    """
    import random

    # Generate random data
    data = {
        f"column_{i}": [random.random() for i in range(row_number)]
        for i in range(column_number)
    }
    return pa.table(data)


def timer():
    """
    Context manager to time the execution of a code block.
    """
    import time

    class Timer:
        def __enter__(self):
            self.start = time.time()
            return self

        def __exit__(self, *args):
            self.end = time.time()
            self.interval = self.end - self.start

    return Timer()


def benchmark_simple_flight(data_table: pa.Table):
    print("-" * 80)
    print("Benchmarking simple flight")
    simple_server = SimpleFlightServer(location="grpc://0.0.0.0:8816")

    import threading

    server_thread = threading.Thread(target=simple_server.serve)
    server_thread.start()

    client = pa.flight.connect("grpc://0.0.0.0:8816")

    # Upload a new dataset

    # Time the upload process
    with timer() as t:
        upload_descriptor = pa.flight.FlightDescriptor.for_path("uploaded.parquet")
        writer, _ = client.do_put(upload_descriptor, data_table.schema)
        writer.write_table(data_table)
        writer.close()
    print(f"Upload time: {t.interval:.2f}s")

    # Retrieve metadata of newly uploaded dataset
    flight = client.get_flight_info(upload_descriptor)
    descriptor = flight.descriptor
    print(
        "Path:",
        descriptor.path[0].decode("utf-8"),
        " Rows:",
        flight.total_records,
        " Size(MB):",
        f"{flight.total_bytes / 1024**2 :.2f}",
    )
    # print("=== Schema ===")
    # print(flight.schema)
    # print("==============")

    # Time the download process
    with timer() as t:
        reader = client.do_get(flight.endpoints[0].ticket)
        read_table = reader.read_all()
    print(
        f"Download time: {t.interval:.2f}s",
    )

    server_thread.join(5)
    simple_server.shutdown()
    simple_server.wait()

    print("-" * 80)


def benchmark_stream_flight(data_table: pa.Table, predefined_total_rows: int):
    print("-" * 80)
    print("Benchmarking stream flight")

    stream_server = streamFlightServer()

    import threading

    server_thread = threading.Thread(target=stream_server.serve)
    server_thread.start()

    client = pa.flight.connect("grpc://0.0.0.0:8815")

    # # Upload a new dataset
    # NUM_BATCHES = 1024
    # ROWS_PER_BATCH = 4096
    # upload_descriptor = pa.flight.FlightDescriptor.for_path("streamed.parquet")
    # batch = pa.record_batch([
    #     pa.array(range(ROWS_PER_BATCH)),
    # ], names=["ints"])
    # writer, _ = client.do_put(upload_descriptor, batch.schema)
    # with writer:
    #     for _ in range(NUM_BATCHES):
    #         writer.write_batch(batch)

    # # Read content of the dataset
    # flight = client.get_flight_info(upload_descriptor)
    # reader = client.do_get(flight.endpoints[0].ticket)
    # total_rows = 0
    # for chunk in reader:
    #     total_rows += chunk.data.num_rows
    # print("Got", total_rows, "rows total, expected", NUM_BATCHES * ROWS_PER_BATCH)

    NUM_BATCHES = 1024
    ROWS_PER_BATCH = data_table.num_rows // NUM_BATCHES
    # Time the upload process
    with timer() as t:
        upload_descriptor = pa.flight.FlightDescriptor.for_path("streamed.parquet")
        # Transform the arrow table into batches
        batch = pa.record_batch(
            [
                data_table.column(j).combine_chunks().slice(0, ROWS_PER_BATCH)
                for j in range(data_table.num_columns)
            ],
            names=data_table.column_names,
        )

        writer, _ = client.do_put(upload_descriptor, batch.schema)
        with writer:
            for start in range(0, data_table.num_rows, ROWS_PER_BATCH):
                length = min(ROWS_PER_BATCH, data_table.num_rows - start)
                writer.write_batch(
                    pa.record_batch(
                        [
                            data_table.column(j).combine_chunks().slice(start, length)
                            for j in range(data_table.num_columns)
                        ],
                        names=data_table.column_names,
                    )
                )

    print(f"Upload time: {t.interval:.2f}s")

    # Time the download process
    with timer() as t:
        upload_descriptor = pa.flight.FlightDescriptor.for_path("streamed.parquet")
        flight = client.get_flight_info(upload_descriptor)
        reader = client.do_get(flight.endpoints[0].ticket)
        total_rows = 0
        for chunk in reader:
            total_rows += chunk.data.num_rows

    print("Got", total_rows, "rows total, expected", predefined_total_rows)
    print(
        f"Download time: {t.interval:.2f}s",
    )

    server_thread.join(5)
    stream_server.shutdown()
    stream_server.wait()

    print("-" * 80)


def main():
    data_table = generate_arrow_table_with_row_column_number(TOTAL_ROWS, TOTAL_COLUMNS)
    benchmark_stream_flight(data_table, TOTAL_ROWS)
    benchmark_simple_flight(data_table)


if __name__ == "__main__":
    main()
