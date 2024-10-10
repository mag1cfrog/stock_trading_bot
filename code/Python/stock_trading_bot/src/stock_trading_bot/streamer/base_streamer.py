from typing import Deque
from loguru import logger
import threading
import signal
import sys

from stock_trading_bot.streamer.protocols import StreamerProtocol


class BaseStreamer(StreamerProtocol):
    def __init__(self, symbol: str, data_buffer: Deque[dict]) -> None:
        """
        Initializes the BaseStreamer with the symbol and data buffer.

        Args:
            symbol (str): The symbol to subscribe to (e.g., "BTC/USD").
            data_buffer (Deque[dict]): Thread-safe deque to store incoming data.
        """
        self.symbol = symbol
        self.data_buffer = data_buffer
        self.thread = threading.Thread(target=self.run_stream, daemon=True)

    def run_stream(self) -> None:
        """
        Placeholder method to run the data stream.
        Should be implemented by subclasses.
        """
        raise NotImplementedError("Subclasses must implement run_stream method.")

    def start(self) -> None:
        """
        Starts the data streaming in a separate thread.
        """
        logger.info(f"BaseStreamer: Starting data stream for {self.symbol}...")
        self.thread.start()
        logger.info("BaseStreamer: Data stream thread started.")

    def stop_stream(self) -> None:
        """
        Placeholder method to stop the data stream.
        Should be implemented by subclasses.
        """
        raise NotImplementedError("Subclasses must implement stop_stream method.")
