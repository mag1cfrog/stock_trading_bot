import os
import threading
import asyncio
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

# Setup logging for detailed debug output
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Load environment variables from a .env file
load_dotenv()

# Retrieve API keys from environment variables
api_key = os.getenv("APCA_API_KEY_ID")
secret_key = os.getenv("APCA_API_SECRET_KEY")

if not api_key or not secret_key:
    logger.error("API key and Secret key must be set in the environment variables.")
    sys.exit(1)

# Initialize the CryptoDataStream with your API credentials
crypto_stream = CryptoDataStream(api_key, secret_key)

# Initialize a thread-safe deque to store incoming data
# Limiting to the latest 100 data points for visualization
data_buffer = deque(maxlen=100)

# Define an asynchronous handler function for incoming crypto quotes
async def on_crypto_quote(data):
    # Append the incoming data to the deque
    data_buffer.append({
        'timestamp': data.timestamp,
        'bid_price': data.bid_price,
        'ask_price': data.ask_price
    })
    logger.debug(f"Received data at {data.timestamp}: Bid={data.bid_price}, Ask={data.ask_price}")

# Subscribe to crypto quotes for the symbol "BTC/USD"
crypto_stream.subscribe_quotes(on_crypto_quote, "BTC/USD")

def run_crypto_stream():
    """
    Function to run the CryptoDataStream.
    This will be executed in a separate daemon thread.
    """
    try:
        logger.info("Starting CryptoDataStream...")
        crypto_stream.run()
    except ValueError as ve:
        if 'connection limit exceeded' in str(ve).lower():
            logger.error(f"Connection limit exceeded: {ve}")
            stop_stream()
            # Exit the entire script to prevent further attempts
            os._exit(1)
        else:
            logger.error(f"ValueError in CryptoDataStream: {ve}")
    except Exception as e:
        logger.error(f"Error in CryptoDataStream: {e}")

# Start the CryptoDataStream in a separate daemon thread
stream_thread = threading.Thread(target=run_crypto_stream, daemon=True)
stream_thread.start()

# Initialize Dash app
app = dash.Dash(__name__)
app.title = "Real-Time Crypto Price Visualization"

# Define the layout of the Dash app
app.layout = html.Div([
    html.H1("Real-Time BTC/USD Price"),
    dcc.Graph(id='live-crypto-graph'),
    dcc.Interval(
        id='graph-update',
        interval=1*1000,  # in milliseconds
        n_intervals=0
    )
])

# Define the callback to update the graph
@app.callback(
    Output('live-crypto-graph', 'figure'),
    [Input('graph-update', 'n_intervals')]
)
def update_graph_live(n):
    """
    Update the Plotly graph with the latest data from the deque.
    """
    # Check if there is data to display
    if not data_buffer:
        return go.Figure()

    # Extract data from the deque
    timestamps = [data['timestamp'] for data in data_buffer]
    bid_prices = [data['bid_price'] for data in data_buffer]
    ask_prices = [data['ask_price'] for data in data_buffer]

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

# Define a function to gracefully stop the CryptoDataStream
def stop_stream():
    logger.info("Stopping CryptoDataStream...")
    try:
        crypto_stream.stop()
    except Exception as e:
        logger.error(f"Error while stopping CryptoDataStream: {e}")

# Define a signal handler for graceful shutdown
def signal_handler(sig, frame):
    logger.info("Received interrupt signal, shutting down gracefully...")
    stop_stream()
    # Allow Dash to shut down
    sys.exit(0)

# Register the signal handler for SIGINT (Ctrl+C) and SIGTERM
signal.signal(signal.SIGINT, signal_handler)
signal.signal(signal.SIGTERM, signal_handler)

# Register atexit to ensure stream is stopped
import atexit
atexit.register(stop_stream)

# Run the Dash app
if __name__ == '__main__':
    try:
        app.run_server(debug=True)
    except Exception as e:
        logger.error(f"Error running Dash server: {e}")
        stop_stream()
        sys.exit(1)
