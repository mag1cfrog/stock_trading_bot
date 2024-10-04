# src/stock_trading_bot/streamer/streamer.py

from loguru import logger
import threading
import os
import sys
from collections import deque
from alpaca.data.live import CryptoDataStream

class CryptoStreamer:
    def __init__(self, api_key: str, secret_key: str, symbol: str, data_buffer: deque)  -> None:
        """
        Initializes the CryptoStreamer with API credentials and subscription details.
        
        Args:
            api_key (str): Alpaca API key.
            secret_key (str): Alpaca Secret key.
            symbol (str): Cryptocurrency symbol to subscribe to (e.g., "BTC/USD").
            data_buffer (deque): Thread-safe deque to store incoming data.
        """
        self.api_key = api_key
        self.secret_key = secret_key
        self.symbol = symbol
        self.data_buffer = data_buffer

        # Initialize the CryptoDataStream
        self.crypto_stream = CryptoDataStream(self.api_key, self.secret_key)
        self.crypto_stream.subscribe_quotes(self.on_crypto_quote, self.symbol)

    async def on_crypto_quote(self, data: dict) -> None:
        """
        Asynchronously handles incoming crypto quotes.
        
        Args:
            data: Data object containing timestamp, bid_price, and ask_price.
        """
        self.data_buffer.append({
            'timestamp': data.timestamp,
            'bid_price': data.bid_price,
            'ask_price': data.ask_price
        })
        logger.debug(f"Received data at {data.timestamp}: Bid={data.bid_price}, Ask={data.ask_price}")

    def run_stream(self):
        """
        Starts the CryptoDataStream and handles exceptions.
        """
        try:
            logger.info("Starting CryptoDataStream...")
            self.crypto_stream.run()
        except ValueError as ve:
            if 'connection limit exceeded' in str(ve).lower():
                logger.error(f"Connection limit exceeded: {ve}")
                self.stop_stream()
                # Exit the entire script to prevent further attempts
                os._exit(1)
            else:
                logger.error(f"ValueError in CryptoDataStream: {ve}")
        except Exception as e:
            logger.error(f"Error in CryptoDataStream: {e}")

    def start(self):
        """
        Starts the CryptoDataStream in a separate daemon thread.
        """
        self.thread = threading.Thread(target=self.run_stream, daemon=True)
        self.thread.start()
        logger.info("CryptoDataStream thread started.")

    def stop_stream(self):
        """
        Gracefully stops the CryptoDataStream.
        """
        logger.info("Stopping CryptoDataStream...")
        try:
            self.crypto_stream.stop()
            logger.info("CryptoDataStream stopped successfully.")
        except Exception as e:
            logger.error(f"Error while stopping CryptoDataStream: {e}")
