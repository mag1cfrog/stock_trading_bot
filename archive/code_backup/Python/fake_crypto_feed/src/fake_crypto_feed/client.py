import asyncio
import websockets
import logging

async def listen(ws_port: int=8765) -> None:
    uri = f"ws://localhost:{ws_port}"
    max_retries = 5
    retry_delay = 1  # Start with 1 second
    for attempt in range(1, max_retries + 1):
        try:
            async with websockets.connect(uri) as websocket:
                try:
                    while True:
                        message = await websocket.recv()
                        print(f"Received: {message}")
                except websockets.exceptions.ConnectionClosed:
                    print("Connection closed")
        except (ConnectionRefusedError, OSError) as e:
            logging.warning(f"Attempt {attempt}: Connection failed. Retrying in {retry_delay} seconds...")
            if attempt == max_retries:
                logging.error("Max retries reached. Exiting.")
                return
            await asyncio.sleep(retry_delay)
            retry_delay *= 2  # Exponential backoff

def run_client(ws_port: int=8765) -> None:
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    loop.run_until_complete(listen(ws_port))

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    run_client()