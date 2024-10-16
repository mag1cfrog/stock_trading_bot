import asyncio
import datetime
import functools
import json
import random
import logging

import psutil
from prometheus_client import start_http_server, Gauge, Counter
import websockets
from websockets import WebSocketServer
import multiprocessing
from multiprocessing.synchronize import Event

# Initialize Prometheus metrics
cpu_usage = Gauge('Python_sender_cpu_usage_percent', 'CPU usage percentage of the data sender')
ram_usage = Gauge('Python_sender_ram_usage_mb', 'RAM usage in MB of the data sender')
io_read = Gauge('sender_io_read_bytes', 'IO read bytes per second of the data sender')
io_write = Gauge('sender_io_write_bytes', 'IO write bytes per second of the data sender')
sent_messages = Counter('Python_sender_sent_messages_total', 'Total number of messages sent by the data sender')
error_count = Counter('Python_sender_error_count_total', 'Total number of errors encountered by the data sender')

async def collect_metrics() -> None:
    process = psutil.Process()

    # Initialize CPU measurement
    process.cpu_percent(interval=None)
    prev_io = process.io_counters()

    while True:
        
        # Get CPU usage
        cpu = process.cpu_percent(interval=None)
        cpu_usage.set(cpu)

        # Get RAM usage in MB
        ram = process.memory_info().rss / (1024 * 1024)
        ram_usage.set(ram)

        # Get I/O counters
        io_counters = process.io_counters()
        io_r = io_counters.read_bytes - prev_io.read_bytes
        io_w = io_counters.write_bytes - prev_io.write_bytes
        prev_io = io_counters

        io_read.set(io_r)
        io_write.set(io_w)

        await asyncio.sleep(1)  # Update every 5 seconds

async def price_feed(websocket: WebSocketServer, _path: str, interval: float) -> None:
    bid_price = random.uniform(10000, 20000)
    ask_price = bid_price + random.uniform(5, 15)

    while True:
        try:
            jitter = random.uniform(-0.1, 0.1)  # Jitter between -10% to +10%
            actual_interval = interval * (1 + jitter)
            await asyncio.sleep(actual_interval)

            # Determine price change direction
            direction = random.choice([-1, 1])

            # Generate percentage change between 1% and 3%
            change_percentage = random.uniform(0.01, 0.03)

            # Update bid_price and ask_price
            bid_change = bid_price * change_percentage * direction
            ask_change = ask_price * (change_percentage + random.uniform(-0.005, 0.005)) * direction

            bid_price += bid_change
            ask_price += ask_change

            if ask_price <= bid_price:
                ask_price = bid_price + random.uniform(5, 15)

            data = {
                "timestamp": datetime.datetime.utcnow().isoformat() + "Z",
                "bid_price": round(bid_price, 2),
                "ask_price": round(ask_price, 2)
            }

            await websocket.send(json.dumps(data))
            logging.info(data)
            sent_messages.inc()

        except Exception as e:
            logging.error(f"Error: {e}")
            error_count.inc()


async def start_server(start_event: Event, wb_port: int=8765, interval: float = 0.01) -> None:
    handler = functools.partial(price_feed, interval=interval)
    server = await websockets.serve(handler, 'localhost', wb_port)
    start_event.set()  # Signal that the server is ready
    await server.wait_closed()
    

def run_sender(start_event: Event, prometheus_port: int=8000, wb_port: int=8765) -> None:
    # Start Prometheus metrics server on port 8000
    start_http_server(prometheus_port)
    
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    loop.create_task(collect_metrics())

    # Signal that the server is starting
    # start_event.set()

    loop.run_until_complete(start_server(start_event, wb_port))


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
    run_sender(multiprocessing.Event())