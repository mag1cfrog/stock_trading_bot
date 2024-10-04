import os

import asyncio
from alpaca.data.live import CryptoDataStream
from dotenv import load_dotenv

load_dotenv()

api_key = os.getenv("APCA_API_KEY_ID")
secret_key = os.getenv("APCA_API_SECRET_KEY")

# Initialize the CryptoDataStream with your API credentials
crypto_stream = CryptoDataStream(api_key, secret_key)

# Define an asynchronous handler function for incoming crypto quotes
async def on_crypto_quote(data):
    print("Crypto Data:", data)

# Subscribe to crypto quotes for the symbol "BTCUSD"
crypto_stream.subscribe_quotes(on_crypto_quote, "BTC/USD")

# Start the streaming session synchronously
crypto_stream.run()
