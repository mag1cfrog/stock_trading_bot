import os
import signal
import sys

import multiprocessing
from multiprocessing.synchronize import Event


def run_sender(start_event: Event) -> None:
    current_dir = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, current_dir)
    from fake_crypto_feed import sender
    sender.run_sender(start_event)


def run_client(start_event: Event) -> None:
    # Wait until the sender signals that it's ready
    start_event.wait()
    current_dir = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, current_dir)
    from fake_crypto_feed import client
    client.run_client()


def main() -> None:
    start_event = multiprocessing.Event()

    sender_process = multiprocessing.Process(target=run_sender, args=(start_event,), name='SenderProcess')
    client_process = multiprocessing.Process(target=run_client, args=(start_event,), name='ClientProcess')

    sender_process.start()
    client_process.start()

    # Define signal handler for graceful shutdown
    def signal_handler(sig, frame):
        print("Shutting down processes...")
        sender_process.terminate()
        client_process.terminate()
        sender_process.join()
        client_process.join()
        sys.exit(0)

    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    sender_process.join()
    client_process.join()

if __name__ == "__main__":
    main()