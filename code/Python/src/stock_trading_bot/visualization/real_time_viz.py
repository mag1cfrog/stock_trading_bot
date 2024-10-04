import os

import asyncio
from alpaca.data.live import CryptoDataStream
from dotenv import load_dotenv

load_dotenv()

api_key = os.getenv("APCA_API_KEY_ID")
secret_key = os.getenv("APCA_API_SECRET_KEY")
# Initialize the stream with your API key and secret key
crypto_stream = CryptoDataStream(api_key, secret_key)

async def on_crypto_quote(data):
    print("Crypto Data:", data)

async def setup_and_start():
    crypto_stream.subscribe_quotes(on_crypto_quote, "BTCUSD")
    await crypto_stream.run()

def main():
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    try:
        loop.run_until_complete(setup_and_start())
    finally:
        loop.close()

if __name__ == '__main__':
    main()
