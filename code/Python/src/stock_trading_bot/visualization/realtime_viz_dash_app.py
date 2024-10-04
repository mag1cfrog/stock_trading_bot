import os
import threading
from collections import deque
from alpaca.data.live import CryptoDataStream
from dotenv import load_dotenv
import dash
from dash import dcc, html
from dash.dependencies import Input, Output
import plotly.graph_objs as go
import signal
import sys
import logging

class CryptoVisualizer:
    def __init__(self):
        # Setup logging for detailed debug output
        logging.basicConfig(level=logging.DEBUG)  # Enable DEBUG level
        self.logger = logging.getLogger(self.__class__.__name__)
        
        # Load environment variables from a .env file
        load_dotenv()
        
        # Retrieve API keys from environment variables
        self.api_key = os.getenv("APCA_API_KEY_ID")
        self.secret_key = os.getenv("APCA_API_SECRET_KEY")
        
        if not self.api_key or not self.secret_key:
            self.logger.error("API key and Secret key must be set in the environment variables.")
            sys.exit(1)
        
        # Initialize the CryptoDataStream with your API credentials
        self.crypto_stream = CryptoDataStream(self.api_key, self.secret_key)
        
        # Initialize a thread-safe deque to store incoming data
        # Limiting to the latest 100 data points for visualization
        self.data_buffer = deque(maxlen=100)
        
        # Initialize Dash app
        self.app = dash.Dash(__name__)
        self.app.title = "Real-Time Crypto Price Visualization"
        
        # Define the layout of the Dash app
        self.app.layout = html.Div([
            html.H1("Real-Time BTC/USD Price"),
            dcc.Graph(id='live-crypto-graph'),
            dcc.Interval(
                id='graph-update',
                interval=1*1000,  # in milliseconds
                n_intervals=0
            )
        ])
        
        # Define the callback to update the graph
        self.app.callback(
            Output('live-crypto-graph', 'figure'),
            [Input('graph-update', 'n_intervals')]
        )(self.update_graph_live)
        
        # Subscribe to crypto quotes for the symbol "BTC/USD"
        self.crypto_stream.subscribe_quotes(self.on_crypto_quote, "BTC/USD")
        
        # Register atexit to ensure stream is stopped
        import atexit
        atexit.register(self.stop_stream)
        
        # Register the signal handler for SIGINT (Ctrl+C) and SIGTERM
        signal.signal(signal.SIGINT, self.signal_handler)
        signal.signal(signal.SIGTERM, self.signal_handler)
    
    async def on_crypto_quote(self, data):
        # Append the incoming data to the deque
        self.data_buffer.append({
            'timestamp': data.timestamp,
            'bid_price': data.bid_price,
            'ask_price': data.ask_price
        })
        self.logger.debug(f"Received data at {data.timestamp}: Bid={data.bid_price}, Ask={data.ask_price}")
    
    def run_crypto_stream(self):
        try:
            self.logger.info("Starting CryptoDataStream...")
            self.crypto_stream.run()
        except ValueError as ve:
            if 'connection limit exceeded' in str(ve).lower():
                self.logger.error(f"Connection limit exceeded: {ve}")
                self.stop_stream()
                # Exit the entire script to prevent further attempts
                os._exit(1)
            else:
                self.logger.error(f"ValueError in CryptoDataStream: {ve}")
        except Exception as e:
            self.logger.error(f"Error in CryptoDataStream: {e}")
    
    def start_stream_thread(self):
        # Start the CryptoDataStream in a separate daemon thread
        stream_thread = threading.Thread(target=self.run_crypto_stream, daemon=True)
        stream_thread.start()
    
    def stop_stream(self):
        self.logger.info("Stopping CryptoDataStream...")
        try:
            self.crypto_stream.stop()
        except Exception as e:
            self.logger.error(f"Error while stopping CryptoDataStream: {e}")
    
    def signal_handler(self, sig, frame):
        self.logger.info("Received interrupt signal, shutting down gracefully...")
        self.stop_stream()
        # Allow Dash to shut down
        sys.exit(0)
    
    def update_graph_live(self, n):
        """
        Update the Plotly graph with the latest data from the deque.
        """
        # Check if there is data to display
        if not self.data_buffer:
            return go.Figure()
    
        # Extract data from the deque
        timestamps = [data['timestamp'] for data in self.data_buffer]
        bid_prices = [data['bid_price'] for data in self.data_buffer]
        ask_prices = [data['ask_price'] for data in self.data_buffer]
    
        # Create the Plotly figure
        fig = go.Figure()
    
        fig.add_trace(go.Scatter(
            x=timestamps,
            y=bid_prices,
            mode='lines+markers',
            name='Bid Price',
            line=dict(color='blue'),
            marker=dict(size=6)
        ))
    
        fig.add_trace(go.Scatter(
            x=timestamps,
            y=ask_prices,
            mode='lines+markers',
            name='Ask Price',
            line=dict(color='red'),
            marker=dict(size=6)
        ))
    
        # Update layout for better visualization
        fig.update_layout(
            title="Real-Time BTC/USD Bid and Ask Prices",
            xaxis_title="Timestamp",
            yaxis_title="Price (USD)",
            xaxis=dict(range=[min(timestamps), max(timestamps)]),
            yaxis=dict(range=[min(bid_prices + ask_prices) * 0.99, max(bid_prices + ask_prices) * 1.01]),
            template="plotly_dark",
            margin=dict(l=40, r=40, t=40, b=40)
        )
    
        return fig
    
    def run_dash(self):
        # Start the CryptoDataStream thread
        self.start_stream_thread()
        # Run the Dash app without the reloader
        try:
            self.app.run_server(debug=True, use_reloader=False)
        except Exception as e:
            self.logger.error(f"Error running Dash server: {e}")
            self.stop_stream()
            sys.exit(1)
    
def main():
    visualizer = CryptoVisualizer()
    visualizer.run_dash()

if __name__ == '__main__':
    main()
