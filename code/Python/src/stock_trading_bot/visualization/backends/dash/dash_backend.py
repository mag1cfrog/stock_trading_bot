import logging
from typing import Dict, List, Tuple

import dash
from dash import dcc, html
from dash.dependencies import Input, Output
from loguru import logger
import plotly.graph_objs as go

from stock_trading_bot.visualization.protocols import VisualizationBackendProtocol
from stock_trading_bot.visualization.data_buffer import DataBuffer


class DashBackend(VisualizationBackendProtocol):
    def __init__(
        self, data_buffer: DataBuffer, title: str = "Dash Application"
    ) -> None:
        """
        Initializes the BaseDashApp with common Dash configurations.

        Args:
            data_buffer (DataBuffer): Shared data buffer containing the latest data points.
            title (str): The title of the Dash application.
        """
        self.data_buffer = data_buffer
        self.title = title
        self.app = dash.Dash(__name__)

        # Suppress werkzeug logs
        logging.getLogger("werkzeug").setLevel(logging.ERROR)

        # State tracking attributes
        self.has_logged_no_data = False
        self.last_timestamp = None

        self.setup_layout()
        self.setup_callbacks()

    def setup_layout(self) -> None:
        """
        Sets up the Dash application layout.
        """
        self.app.layout = html.Div(
            [
                html.H1(self.title),
                dcc.Graph(id="live-graph"),
                dcc.Interval(
                    id="graph-update",
                    interval=1 * 1000,  # Update every second
                    n_intervals=0,
                ),
            ]
        )

    def setup_callbacks(self) -> None:
        """
        Sets up the Dash callbacks for updating the graph.
        """

        @self.app.callback(
            Output("live-graph", "figure"), [Input("graph-update", "n_intervals")]
        )
        def update_graph_live(n: int) -> go.Figure:
            """
            Callback function to update the live graph based on new data.

            Args:
                n (int): Number of intervals that have passed.

            Returns:
                go.Figure: Updated Plotly figure.
            """
            return self._generate_figure()

    def _generate_figure(self) -> go.Figure:
        """
        Generates the Plotly figure based on the current data in the buffer.

        Returns:
            go.Figure: The updated figure to be displayed.
        """
        try:
            if self.data_buffer.is_empty():
                self._handle_no_data()
                return go.Figure()
            else:
                self._handle_new_data()

                # Extract data from the buffer
                data_snapshot = self.data_buffer.get_snapshot()
                timestamps, bid_prices, ask_prices = self._extract_data(data_snapshot)

            # Create the Plotly figure
            fig = self._create_plotly_figure(timestamps, bid_prices, ask_prices)
            return fig

        except Exception as e:
            logger.error(f"DashBackend: Error generating figure: {e}")
            return go.Figure()

    def _handle_no_data(self) -> None:
        """
        Handles the scenario when there's no data available in the buffer.
        Logs the event only once until data becomes available.
        """
        if not self.has_logged_no_data:
            logger.info("DashBackend: No data available to update graph.")
            self.has_logged_no_data = True

    def _handle_new_data(self) -> None:
        """
        Handles the scenario when new data is available in the buffer.
        Logs the update if new data has arrived.
        """
        # Reset the 'no data' flag if data is available
        if self.has_logged_no_data:
            self.has_logged_no_data = False

        # Retrieve the latest timestamp to check for new data
        latest_timestamp = self.data_buffer.get_latest_timestamp()

        # Check if new data has arrived
        if latest_timestamp != self.last_timestamp:
            logger.info("DashBackend: Graph updated with new data.")
            self.last_timestamp = latest_timestamp

    def _extract_data(
        self, data_snapshot: List[Dict]
    ) -> Tuple[List[str], List[float], List[float]]:
        """
        Extracts timestamps, bid prices, and ask prices from the data snapshot.

        Args:
            data_snapshot (List[Dict]): A list of data dictionaries.

        Returns:
            Tuple[List[str], List[float], List[float]]: Lists of timestamps, bid prices, and ask prices.
        """
        timestamps = [data["timestamp"] for data in data_snapshot]
        bid_prices = [data["bid_price"] for data in data_snapshot]
        ask_prices = [data["ask_price"] for data in data_snapshot]
        return timestamps, bid_prices, ask_prices

    def _create_plotly_figure(
        self, timestamps: List[str], bid_prices: List[float], ask_prices: List[float]
    ) -> go.Figure:
        """
        Creates a Plotly figure with bid and ask prices over time.

        Args:
            timestamps (List[str]): List of timestamps.
            bid_prices (List[float]): List of bid prices.
            ask_prices (List[float]): List of ask prices.

        Returns:
            go.Figure: The constructed Plotly figure.
        """
        fig = go.Figure()

        fig.add_trace(
            go.Scatter(
                x=timestamps,
                y=bid_prices,
                mode="lines+markers",
                name="Bid Price",
                line=dict(color="blue"),
                marker=dict(size=6),
            )
        )

        fig.add_trace(
            go.Scatter(
                x=timestamps,
                y=ask_prices,
                mode="lines+markers",
                name="Ask Price",
                line=dict(color="red"),
                marker=dict(size=6),
            )
        )

        fig.update_layout(
            title=self.title,
            xaxis_title="Timestamp",
            yaxis_title="Price (USD)",
            template="plotly_dark",
            margin=dict(l=40, r=40, t=40, b=40),
        )

        return fig

    def run(self) -> None:
        """
        Runs the Dash server.
        """
        logger.info("DashBackend: Running Dash server...")
        self.app.run_server(debug=False, use_reloader=False)
