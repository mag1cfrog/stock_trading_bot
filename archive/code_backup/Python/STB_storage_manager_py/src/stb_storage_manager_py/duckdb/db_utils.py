import pyarrow as pa
import pyarrow.flight as flight

from stb_storage_manager_py.protocols import StorageManager


class FlightServer(flight.FlightServerBase):
    def __init__(
        self, storage_manager: StorageManager, host: str = "localhost", port: int = 8815
    ) -> None:
        super().__init__(location=f"grpc://{host}:{port}")
        self.storage_manager: StorageManager = storage_manager
