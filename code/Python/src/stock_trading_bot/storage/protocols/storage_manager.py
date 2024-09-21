from typing import Protocol


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