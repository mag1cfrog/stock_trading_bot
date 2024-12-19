import time
import duckdb
import multiprocessing  # Use multiprocessing instead of concurrent.futures
import pyarrow as pa
import pyarrow.parquet
import pyarrow.flight
import grpc
import pandas as pd
import logging  # Add import for logging

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
)

# ...existing code...


def generate_fake_data(num_rows, db_file="test_data.duckdb"):
    logging.info(f"Generating fake data with {num_rows} rows of 64-bit integers.")
    conn = duckdb.connect(db_file)

    # Define the maximum value for a 64-bit integer
    max_int64 = 2**63 - 1

    # SQL to generate a table of random 64-bit integers
    sql_command = f"""
    CREATE TABLE test_data AS 
    SELECT CAST({max_int64} * RANDOM() AS BIGINT) AS random_value
    FROM range(0, {num_rows})
    """
    conn.execute(sql_command)
    conn.close()
    logging.info(f"Fake data generated and stored in {db_file}.")
    return db_file


def rest_api_server(db_file):
    from fastapi import FastAPI
    import uvicorn
    import duckdb

    app = FastAPI()
    conn = duckdb.connect(db_file)

    @app.get("/data")
    def get_data():
        logging.info("REST API server received a request for data.")
        start_time = time.time()
        data = conn.execute("SELECT * FROM test_data").fetchall()
        prepare_time = time.time() - start_time
        logging.info(f"Data prepared in {prepare_time:.4f} seconds.")
        return {"data": data, "prepare_time": prepare_time}

    logging.info("Starting REST API server.")
    uvicorn.run(app, host="0.0.0.0", port=5000)


def rest_api_client():
    import requests

    logging.info("REST API client started.")
    start_time = time.time()
    response = requests.get("http://localhost:5000/data")
    transfer_time = time.time() - start_time
    logging.info(f"Data received from REST API in {transfer_time:.4f} seconds.")
    json_data = response.json()
    data = json_data["data"]
    prepare_time = json_data["prepare_time"]
    start_parquet_time = time.time()
    df = pd.DataFrame(data, columns=["value"])
    table = pa.Table.from_pandas(df)
    pa.parquet.write_table(table, "data_rest.parquet")
    parquet_time = time.time() - start_parquet_time
    logging.info(f"Data written to Parquet file in {parquet_time:.4f} seconds.")
    return prepare_time, transfer_time, parquet_time


def grpc_server(db_file):
    from concurrent import futures
    import grpc
    from data_pb2 import DataResponse, Empty
    import data_pb2_grpc
    import duckdb

    # Define maximum message size (100MB)
    MAX_MESSAGE_LENGTH = 100 * 1024 * 1024

    class DataService(data_pb2_grpc.DataServiceServicer):
        def __init__(self, conn):
            self.conn = conn

        def GetData(self, request, context):
            logging.info("gRPC server received a request for data.")
            start_time = time.time()
            data = self.conn.execute("SELECT * FROM test_data").fetchall()
            prepare_time = time.time() - start_time
            logging.info(f"Data prepared in {prepare_time:.4f} seconds.")
            response = DataResponse()
            for row in data:
                response.data.append(row[0])
            response.prepare_time = prepare_time
            return response

    server = grpc.server(
        futures.ThreadPoolExecutor(max_workers=10),
        options=[
            ("grpc.max_send_message_length", MAX_MESSAGE_LENGTH),
            ("grpc.max_receive_message_length", MAX_MESSAGE_LENGTH),
        ],
    )
    conn = duckdb.connect(db_file)
    data_pb2_grpc.add_DataServiceServicer_to_server(DataService(conn), server)
    server.add_insecure_port("[::]:50051")
    logging.info("Starting gRPC server.")
    server.start()
    server.wait_for_termination()


def grpc_client():
    import grpc
    from data_pb2 import Empty
    import data_pb2_grpc

    # Define maximum message size (100MB)
    MAX_MESSAGE_LENGTH = 1000 * 1024 * 1024

    logging.info("gRPC client started.")
    start_time = time.time()
    channel = grpc.insecure_channel(
        "localhost:50051",
        options=[
            ("grpc.max_send_message_length", MAX_MESSAGE_LENGTH),
            ("grpc.max_receive_message_length", MAX_MESSAGE_LENGTH),
        ],
    )
    stub = data_pb2_grpc.DataServiceStub(channel)
    response = stub.GetData(Empty())
    transfer_time = time.time() - start_time
    logging.info(f"Data received from gRPC server in {transfer_time:.4f} seconds.")
    data = list(response.data)  # Convert to standard Python list
    start_parquet_time = time.time()
    df = pd.DataFrame(data, columns=["value"])
    table = pa.Table.from_pandas(df)
    pa.parquet.write_table(table, "data_grpc.parquet")
    parquet_time = time.time() - start_parquet_time
    logging.info(f"Data written to Parquet file in {parquet_time:.4f} seconds.")
    return response.prepare_time, transfer_time, parquet_time


def find_free_port():
    import socket

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("", 0))  # Bind to a free port provided by the host.
        return s.getsockname()[1]  # Return the port number assigned.


def arrow_flight_server(db_file, port=None):
    import pyarrow.flight
    import duckdb

    class FlightServer(pa.flight.FlightServerBase):
        def __init__(self, db_file: str = "test_data.duckdb", port=None):
            if port is None:
                port = find_free_port()
            location = f"grpc://0.0.0.0:{port}"
            super().__init__(location)
            self._location = location
            self.conn = duckdb.connect(db_file)
            self.data = None

            # Write port to a temporary file for client to read
            with open("arrow_flight_port.txt", "w") as f:
                f.write(str(port))

        def _make_flight_info(self):
            if self.data is None:
                self.data = self.conn.execute(
                    "SELECT * FROM test_data"
                ).fetch_arrow_table()
            descriptor = pa.flight.FlightDescriptor.for_path(b"benchmark_data")
            endpoints = [pa.flight.FlightEndpoint("benchmark_data", [self._location])]
            return pa.flight.FlightInfo(
                self.data.schema,
                descriptor,
                endpoints,
                self.data.num_rows,
                -1,  # Unknown total bytes
            )

        def get_flight_info(self, context, descriptor):
            return self._make_flight_info()

        def do_get(self, context, ticket):
            logging.info("Arrow Flight server received a request for data.")
            start_time = time.time()

            if self.data is None:
                self.data = self.conn.execute(
                    "SELECT * FROM test_data"
                ).fetch_arrow_table()

            prepare_time = time.time() - start_time
            logging.info(f"Data prepared in {prepare_time:.4f} seconds.")

            # Add prepare_time as custom metadata
            # metadata = [pa.py_buffer(str(prepare_time).encode())]

            return pa.flight.RecordBatchStream(self.data)

    logging.info("Starting Arrow Flight server.")
    server = FlightServer()
    server.serve()


def arrow_flight_client():
    # Read port from temporary file
    with open("arrow_flight_port.txt", "r") as f:
        port = int(f.read().strip())

    logging.info("Arrow Flight client started.")
    client = pa.flight.connect(f"grpc://localhost:{port}")

    start_time = time.time()

    # Get flight info first
    descriptor = pa.flight.FlightDescriptor.for_path(b"benchmark_data")
    flight_info = client.get_flight_info(descriptor)

    # Get the actual data
    reader = client.do_get(flight_info.endpoints[0].ticket)
    data = reader.read_all()

    transfer_time = time.time() - start_time
    logging.info(
        f"Data received from Arrow Flight server in {transfer_time:.4f} seconds."
    )

    # Write to parquet
    start_parquet_time = time.time()
    pa.parquet.write_table(data, "data_arrow.parquet")
    parquet_time = time.time() - start_parquet_time
    logging.info(f"Data written to Parquet file in {parquet_time:.4f} seconds.")

    # prepare_time = float(metadata[0].buf.decode('utf-8')) if metadata else 0

    return transfer_time, parquet_time


def wait_for_container(container, timeout=30):
    """Wait for container to be ready by checking its logs and status"""
    import time

    start_time = time.time()
    while time.time() - start_time < timeout:
        try:
            # Check container status
            container.reload()
            status = container.status
            if status == "exited":
                logs = container.logs().decode("utf-8")
                raise RuntimeError(f"Container exited unexpectedly. Logs:\n{logs}")

            # Check logs for server start message
            if status == "running":
                logs = container.logs().decode("utf-8")
                if "Starting Arrow Flight server" in logs:
                    time.sleep(2)  # Give it more time to fully start
                    return True

            time.sleep(0.5)
        except Exception as e:
            logging.error(f"Error checking container status: {e}")
            return False

    # If we get here, we timed out
    try:
        logs = container.logs().decode("utf-8")
        logging.error(f"Container startup timed out. Logs:\n{logs}")
    except Exception as e:
        logging.error(f"Failed to get container logs: {e}")
    return False


def setup_docker_arrow_flight(db_file):
    import docker
    import shutil
    import tempfile
    import os
    from pathlib import Path

    # Find a free port for the Docker container
    docker_port = find_free_port()

    # Create a temporary directory for Docker context
    with tempfile.TemporaryDirectory() as temp_dir:
        # Copy necessary files to temp directory
        temp_db = os.path.join(temp_dir, "test_data.duckdb")
        temp_script = os.path.join(temp_dir, "server.py")
        temp_requirements = os.path.join(temp_dir, "requirements.txt")
        temp_dockerfile = os.path.join(temp_dir, "Dockerfile")

        # Copy database file
        shutil.copy2(db_file, temp_db)

        # Create a simplified server script
        with open(temp_script, "w") as f:
            f.write("""
import logging
import os
import pathlib
import pyarrow as pa
import pyarrow.flight
import pyarrow.parquet
import duckdb

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class FlightServer(pa.flight.FlightServerBase):
    def __init__(self, db_file, port):
        location = f"grpc://0.0.0.0:{port}"
        super().__init__(location)
        self._location = location
        self._repo = pathlib.Path('./data')
        self._repo.mkdir(exist_ok=True)
        
        # Load data from DuckDB and save as parquet
        conn = duckdb.connect(db_file)
        self.data = conn.execute("SELECT * FROM test_data").fetch_arrow_table()
        self.data_path = self._repo / "benchmark_data.parquet"
        pa.parquet.write_table(self.data, self.data_path)
        logger.info(f"Data saved to {self.data_path}")

    def _make_flight_info(self, dataset):
        dataset_path = self._repo / dataset
        schema = pa.parquet.read_schema(dataset_path)
        metadata = pa.parquet.read_metadata(dataset_path)
        descriptor = pa.flight.FlightDescriptor.for_path(dataset.encode('utf-8'))
        endpoints = [pa.flight.FlightEndpoint(dataset, [self._location])]
        return pa.flight.FlightInfo(schema,
                                  descriptor,
                                  endpoints,
                                  metadata.num_rows,
                                  metadata.serialized_size)

    def list_flights(self, context, criteria):
        for dataset in self._repo.iterdir():
            yield self._make_flight_info(dataset.name)

    def get_flight_info(self, context, descriptor):
        return self._make_flight_info(descriptor.path[0].decode('utf-8'))

    def do_get(self, context, ticket):
        dataset = ticket.ticket.decode('utf-8')
        dataset_path = self._repo / dataset
        logger.info(f"Reading data from {dataset_path}")
        return pa.flight.RecordBatchStream(pa.parquet.read_table(dataset_path))

if __name__ == '__main__':
    port = int(os.environ.get('PORT', 8815))
    logger.info(f"Starting Arrow Flight server on port {port}")
    server = FlightServer('/app/test_data.duckdb', port)
    logger.info("Server instance created, starting to serve")
    server.serve()
""")

        # Create requirements.txt with minimal dependencies
        with open(temp_requirements, "w") as f:
            f.write(
                """
pyarrow>=14.0.1
duckdb>=0.9.2
""".strip()
            )

        # Create Dockerfile
        with open(temp_dockerfile, "w") as f:
            f.write(f"""FROM python:3.12-slim

WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \\
    build-essential \\
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

ENV PORT={docker_port}
ENV PYTHONUNBUFFERED=1

CMD ["python", "server.py"]
""")

        try:
            client = docker.from_env()
            logging.info("Building Docker image...")
            image, build_logs = client.images.build(
                path=temp_dir, tag="arrow-flight-server", forcerm=True
            )

            logging.info(f"Starting container on port {docker_port}...")
            container = client.containers.run(
                "arrow-flight-server",
                detach=True,
                network_mode="host",
                environment={"PORT": str(docker_port)},
                remove=True,
            )

            # Wait for container to be ready with better logging
            for i in range(30):  # 30 second timeout
                try:
                    container.reload()
                    if container.status == "exited":
                        logs = container.logs().decode("utf-8")
                        raise RuntimeError(f"Container exited. Logs:\n{logs}")

                    logs = container.logs().decode("utf-8")
                    if "Starting Arrow Flight server" in logs:
                        if "Server instance created, starting to serve" in logs:
                            time.sleep(1)  # Give the server a moment to start listening
                            with open("arrow_flight_port.txt", "w") as f:
                                f.write(str(docker_port))
                            return container
                except Exception as e:
                    logging.error(f"Error checking container: {e}")
                time.sleep(1)

            raise RuntimeError("Container failed to start within timeout")

        except Exception as e:
            logging.error(f"Error setting up Docker container: {e}")
            if "container" in locals():
                try:
                    logs = container.logs().decode("utf-8")
                    logging.error(f"Container logs:\n{logs}")
                    container.stop()
                except:
                    pass
            raise


def arrow_flight_client_docker():
    # Read port from temporary file
    with open("arrow_flight_port.txt", "r") as f:
        port = int(f.read().strip())

    logging.info("Docker-based Arrow Flight client started.")
    # Connect to containerized server
    client = pa.flight.connect(f"grpc://localhost:{port}")

    start_time = time.time()

    # Get flight info
    descriptor = pa.flight.FlightDescriptor.for_path(b"benchmark_data.parquet")
    try:
        # Get flight info with retry
        max_retries = 3
        retry_delay = 1
        last_error = None

        for attempt in range(max_retries):
            try:
                flight_info = client.get_flight_info(descriptor)
                reader = client.do_get(flight_info.endpoints[0].ticket)
                data = reader.read_all()

                transfer_time = time.time() - start_time
                logging.info(
                    f"Data received from Docker Arrow Flight server in {transfer_time:.4f} seconds."
                )

                start_parquet_time = time.time()
                pa.parquet.write_table(data, "data_arrow_docker.parquet")
                parquet_time = time.time() - start_parquet_time
                logging.info(
                    f"Data written to Parquet file in {parquet_time:.4f} seconds."
                )

                return transfer_time, parquet_time

            except pa.flight.FlightUnavailableError as e:
                if attempt < max_retries - 1:
                    time.sleep(retry_delay)
                    retry_delay *= 2
                else:
                    raise RuntimeError(
                        f"Failed to connect after {max_retries} attempts"
                    ) from e

    except Exception as e:
        logging.error(f"Error in Docker Arrow Flight client: {e}")
        raise


def cleanup(dbfile="test_data.duckdb"):
    import os

    logging.info("Cleaning up generated files.")
    if os.path.exists(dbfile):
        os.remove(dbfile)
    if os.path.exists("data_rest.parquet"):
        os.remove("data_rest.parquet")
    if os.path.exists("data_grpc.parquet"):
        os.remove("data_grpc.parquet")
    if os.path.exists("data_arrow.parquet"):
        os.remove("data_arrow.parquet")
    if os.path.exists("data_arrow_docker.parquet"):
        os.remove("data_arrow_docker.parquet")
    if os.path.exists("arrow_flight_port.txt"):
        os.remove("arrow_flight_port.txt")
    # Clean up Docker resources
    try:
        import docker

        client = docker.from_env()
        if client.containers.list(filters={"name": "arrow-flight-server"}):
            client.images.remove("arrow-flight-server", force=True)
    except Exception as e:
        logging.warning(f"Failed to clean up Docker resources: {e}")


def benchmark():
    num_rows = 5000000  # Predefined row number

    logging.info("Starting benchmark.")
    cleanup()
    db_file = generate_fake_data(num_rows)

    # Start REST API server process
    logging.info("Launching REST API server.")
    rest_server_process = multiprocessing.Process(
        target=rest_api_server, args=(db_file,)
    )
    rest_server_process.start()
    time.sleep(1)  # Give the server time to start

    # Run REST API client
    rest_prepare_time, rest_transfer_time, rest_parquet_time = rest_api_client()
    logging.info("REST API benchmark completed.")

    # Terminate REST API server
    rest_server_process.terminate()
    rest_server_process.join()

    # Start gRPC server process
    logging.info("Launching gRPC server.")
    grpc_server_process = multiprocessing.Process(target=grpc_server, args=(db_file,))
    grpc_server_process.start()
    time.sleep(1)  # Give the server time to start

    # Run gRPC client
    grpc_prepare_time, grpc_transfer_time, grpc_parquet_time = grpc_client()
    logging.info("gRPC benchmark completed.")

    # Terminate gRPC server
    grpc_server_process.terminate()
    grpc_server_process.join()

    # Start Arrow Flight server process
    logging.info("Launching Arrow Flight server.")
    arrow_server_process = multiprocessing.Process(
        target=arrow_flight_server, args=(db_file,)
    )
    arrow_server_process.start()
    time.sleep(1)  # Give the server time to start

    # Run Arrow Flight client
    arrow_transfer_time, arrow_parquet_time = arrow_flight_client()
    logging.info("Arrow Flight benchmark completed.")

    # Terminate Arrow Flight server
    arrow_server_process.terminate()
    arrow_server_process.join()

    # Start Docker-based Arrow Flight server
    logging.info("Launching Docker-based Arrow Flight server.")
    container = setup_docker_arrow_flight(db_file)
    time.sleep(2)  # Give the containerized server time to start

    # Run Docker-based Arrow Flight client
    arrow_docker_transfer_time, arrow_docker_parquet_time = arrow_flight_client_docker()
    logging.info("Docker-based Arrow Flight benchmark completed.")

    # Stop Docker container
    container.stop()

    # Collect and print benchmark results
    logging.info("Benchmark results:")
    print("REST API:")
    print(f"  Prepare time: {rest_prepare_time}")
    print(f"  Transfer time: {rest_transfer_time}")
    print(f"  Parquet write time: {rest_parquet_time}\n")
    print("gRPC:")
    print(f"  Prepare time: {grpc_prepare_time}")
    print(f"  Transfer time: {grpc_transfer_time}")
    print(f"  Parquet write time: {grpc_parquet_time}\n")
    print("Arrow Flight:")
    print(f"  Transfer time: {arrow_transfer_time}")
    print(f"  Parquet write time: {arrow_parquet_time}")
    print("\nArrow Flight (Docker):")
    print(f"  Transfer time: {arrow_docker_transfer_time}")
    print(f"  Parquet write time: {arrow_docker_parquet_time}")

    cleanup()
    logging.info("Benchmark completed and cleanup done.")


if __name__ == "__main__":
    benchmark()
