from typing import Protocol


class StreamerProtocol(Protocol):
    symbol: str

    def start(self) -> None:
        """Start the data streamer."""
        ...

    def stop_stream(self) -> None:
        """Stop the data streamer."""
        ...
