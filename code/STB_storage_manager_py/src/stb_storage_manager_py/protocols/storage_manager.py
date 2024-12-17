from typing import Protocol


class StorageManager(Protocol):
    """Storage manager protocol."""

    def __enter__(self):
        """Enter the storage manager context."""
        pass

    def __exit__(self, exc_type, exc_value, traceback):
        """Exit the storage manager context."""
        pass

    def init_schema(self, **kwargs) -> None:
        """Initialize the storage schema."""
        pass

    def add_data(self, **kwargs) -> None:
        """Add new data to the storage system."""
        pass

    
