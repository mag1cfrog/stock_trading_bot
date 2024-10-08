# import asyncio
import atexit
import os
import threading

from alpaca.data.live import CryptoDataStream
from loguru import logger

from stock_trading_bot.streamer.base_streamer import BaseStreamer
from stock_trading_bot.streamer.protocols import StreamerProtocol
from stock_trading_bot.visualization.data_buffer import DataBuffer

class CryptoQuotesDataStreamer(BaseStreamer, StreamerProtocol):
    def __init__(self, api_key: str, secret_key: str, symbol: str, data_buffer: DataBuffer) -> None:
        """
        Initializes the CryptoQuotesDataStreamer with API credentials and subscription details.
        
        Args:
            api_key (str): Alpaca API key.
            secret_key (str): Alpaca Secret key.
            symbol (str): Cryptocurrency symbol to subscribe to (e.g., "BTC/USD").
            data_buffer (DataBuffer): Shared data buffer for incoming data.
        """
        super().__init__(symbol, data_buffer)
        self.api_key = api_key
        self.secret_key = secret_key

        # Initialize the CryptoDataStream
        self.crypto_stream = CryptoDataStream(self.api_key, self.secret_key)
        self.crypto_stream.subscribe_quotes(self.on_crypto_quote, self.symbol)

        # Register atexit to ensure the stream is stopped gracefully
        atexit.register(self.stop_stream)

    async def on_crypto_quote(self, data: dict) -> None:
        """
        Asynchronously handles incoming crypto quotes.
        
        Args:
            data: Data object containing timestamp, bid_price, and ask_price.
        """
        logger.debug(f"on_crypto_quote: Received data at {data.timestamp}")
        self.data_buffer.append({
            'timestamp': data.timestamp,
            'bid_price': data.bid_price,
            'ask_price': data.ask_price
        })
        logger.debug(f"on_crypto_quote: Appended data with {data.timestamp=}, {data.bid_price=} and {data.ask_price=}  to buffer.")

    def run_stream(self) -> None:
        """
        Starts the CryptoDataStream and handles exceptions.
        """
        try:
            logger.info("CryptoQuotesDataStreamer: Starting CryptoDataStream...")

            # # Create a new event loop for this thread
            # loop = asyncio.new_event_loop()
            # asyncio.set_event_loop(loop)

            # # Run the CryptoDataStream within the event loop
            # loop.run_until_complete(self.crypto_stream.run())
            self.crypto_stream.run()
        except ValueError as ve:
            if 'connection limit exceeded' in str(ve).lower():
                logger.error(f"CryptoQuotesDataStreamer: Connection limit exceeded: {ve}")
                self.stop_stream()
                # Exit the entire script to prevent further attempts
                os._exit(1)
            else:
                logger.error(f"CryptoQuotesDataStreamer: ValueError in CryptoDataStream: {ve}")
        except Exception as e:
            logger.error(f"CryptoQuotesDataStreamer: Error in CryptoDataStream: {e}")

    def stop_stream(self) -> None:
        """
        Gracefully stops the CryptoDataStream.
        """
        logger.info("Stopping CryptoDataStream...")
        try:
            self.crypto_stream.stop()
            logger.info("CryptoDataStream stopped successfully.")
        except Exception as e:
            logger.error(f"Error while stopping CryptoDataStream: {e}")
