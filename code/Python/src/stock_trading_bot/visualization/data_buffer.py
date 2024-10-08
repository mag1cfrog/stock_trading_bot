from collections import deque
from dataclasses import dataclass, field
from threading import RLock, Lock
from typing import Deque, List, Dict, Optional

from loguru import logger


@dataclass
class DataBuffer:
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
                return self.buffer[-1].get('timestamp')
            return None
        
    def __len__(self) -> int:
        with self.lock:
            return len(self.buffer)