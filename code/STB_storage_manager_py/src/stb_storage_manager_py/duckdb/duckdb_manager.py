from stb_storage_manager_py.duckdb import db_utils
from stb_storage_manager_py.protocols import StorageManager


class DuckDBManager(StorageManager):
    def __init__(self):
        pass

    def __enter__(self):
        pass

    def __exit__(self, exc_type, exc_value, traceback):
        pass

    def init_schema(self, **kwargs) -> None:
        pass

    def add_data(self, **kwargs) -> None:
        pass
