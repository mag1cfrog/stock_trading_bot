from typing import Protocol, Deque


class VisualizerProtocol(Protocol):
    symbol: str

    def run(self) -> None:
        """Start the visualizer."""
        ...


class VisualizationBackendProtocol(Protocol):
    def setup_layout(self) -> None:
        """Set up the visualization layout."""
        ...

    def update_visualization(self, data_buffer: Deque[dict]) -> None:
        """Update the visualization with new data."""
        ...

    def run(self) -> None:
        """Run the visualization application."""
        ...
