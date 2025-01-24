from collections import deque
from dataclasses import dataclass, field
from threading import RLock, Lock
from typing import Deque, List, Dict, Optional

from loguru import logger


@dataclass
class DataBuffer:
    """
    DataBuffer class for storing and managing data points in a thread-safe manner.

    Args:
        maxlen (int): Maximum number of data points to store in the buffer (default: 1000).

    Attributes:
        buffer (Deque[Dict]): Deque containing the data points.
        lock (Lock): Lock for thread-safe access to the buffer.

    Methods:
        append(data: Dict) -> None: Appends a data point to the buffer.
        get_snapshot() -> List[Dict]: Returns a snapshot of the buffer.
        is_empty() -> bool: Checks if the buffer is empty.
        get_latest_timestamp() -> Optional[str]: Returns the timestamp of the latest data point.
        __len__() -> int: Returns the number of data points in the buffer.

    """

    maxlen: int = 1000
    buffer: Deque[Dict] = field(init=False)
    lock: Lock = field(default_factory=Lock)

    def __post_init__(self):
        self.buffer = deque(maxlen=self.maxlen)

    # In DataBuffer
    def append(self, data: Dict) -> None:
        logger.debug("DataBuffer: Attempting to acquire lock for append.")
        with self.lock:
            logger.debug("DataBuffer: Lock acquired for append.")
            self.buffer.append(data)
        logger.debug("DataBuffer: Lock released after append.")

    def get_snapshot(self) -> List[Dict]:
        with self.lock:
            return list(self.buffer)

    def is_empty(self) -> bool:
        with self.lock:
            return len(self.buffer) == 0

    def get_latest_timestamp(self) -> Optional[str]:
        with self.lock:
            if self.buffer:
                return self.buffer[-1].get("timestamp")
            return None

    def __len__(self) -> int:
        with self.lock:
            return len(self.buffer)
