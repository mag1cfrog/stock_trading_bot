from collections import deque
import signal
import sys

from loguru import logger

from visualization.protocols import VisualizerProtocol, VisualizationBackendProtocol
from visualization.backends.dash.dash_backend import DashBackend
from streamer.crypto_quotes_data_streamer import CryptoQuotesDataStreamer

class CryptoQuotesDashVisualizer(VisualizerProtocol):
    symbol: str

    def __init__(self, api_key: str, secret_key: str, symbol: str = "BTC/USD") -> None:
        """
        Initializes the CryptoQuotesDashVisualizer with streaming and Dash visualization capabilities.

        Args:
            api_key (str): Alpaca API key.
            secret_key (str): Alpaca Secret key.
            symbol (str): Cryptocurrency symbol to visualize (default: "BTC/USD").
        """
        self.symbol = symbol

        # Initialize data buffer
        self.data_buffer = deque(maxlen=1000)

        # Initialize the streamer
        self.streamer = CryptoQuotesDataStreamer(
            api_key=api_key,
            secret_key=secret_key,
            symbol=symbol,
            data_buffer=self.data_buffer
        )

        # Initialize Visualization Backend (Dash in this case)
        self.visualization_backend: VisualizationBackendProtocol = DashBackend(
            data_buffer=self.data_buffer,
            title=f"Real-Time {symbol} Price Visualization"
        )

        # Register signal handlers for graceful shutdown
        signal.signal(signal.SIGINT, self.signal_handler)
        signal.signal(signal.SIGTERM, self.signal_handler)

    def run(self) -> None:
        """
        Starts the streamer and runs the visualization backend.
        """
        # Start the streamer
        self.streamer.start()
        logger.info(f"{self.symbol} Quotes Dash Visualizer is running. Awaiting data...")

        # Run the visualization backend
        self.visualization_backend.run()

    def signal_handler(self, sig, frame):
        """
        Handles interrupt signals to ensure graceful shutdown.

        Args:
            sig: Signal number.
            frame: Current stack frame.
        """
        logger.info("CryptoQuotesDashVisualizer: Received interrupt signal, shutting down gracefully...")
        self.streamer.stop_stream()
        sys.exit(0)