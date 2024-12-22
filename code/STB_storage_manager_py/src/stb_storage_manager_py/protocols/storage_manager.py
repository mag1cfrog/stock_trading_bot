from typing import Protocol


class StorageManager(Protocol):
    """Storage manager protocol."""

    def __enter__(self):
        """Enter the storage manager context."""
        pass

    def __exit__(self, exc_type, exc_value, traceback):
        """Exit the storage manager context."""
        pass

    def _put_data(self, **kwargs) -> None:
        """Initialize the storage schema."""
        pass

    def _get_data(self, **kwargs) -> None:
        """Add new data to the storage system."""
        pass
