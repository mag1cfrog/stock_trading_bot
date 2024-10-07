from collections import deque
from loguru import logger
import threading
from typing import Protocol, Deque

class BaseStreamer(Protocol):
    def __init__(self, symbol: str, data_buffer: deque) -> None:
        """
        Initializes the BaseStreamer with the symbol and data buffer.
        
        Args:
            symbol (str): The symbol to subscribe to (e.g., "BTC/USD").
            data_buffer (deque): Thread-safe deque to store incoming data.
        """
        self.symbol = symbol
        self.data_buffer = data_buffer
        self.thread = threading.Thread(target=self.run_stream, daemon=True)

    def run_stream(self) -> None:
        """
        Starts the data stream and handles exceptions.
        """
        ...

    def start(self) -> None:
        """
        Starts the data streaming in a separate thread.
        """
        logger.info(f"BaseStreamer: Starting data stream for {self.symbol}...")
        self.thread.start()
        logger.info("BaseStreamer: Data stream thread started.")

    def stop_stream(self) -> None:
        """
        Stops the data stream.
        """
        ...