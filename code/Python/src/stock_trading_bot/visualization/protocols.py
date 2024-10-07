from typing import Protocol

class VisualizerProtocol(Protocol):
    symbol: str

    def run(self) -> None:
        """Start the visualizer."""
        ...