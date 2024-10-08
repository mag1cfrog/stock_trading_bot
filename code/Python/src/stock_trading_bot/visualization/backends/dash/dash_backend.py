import logging

import dash
from dash import dcc, html
from dash.dependencies import Input, Output
from loguru import logger
import plotly.graph_objs as go
from typing import Deque

from stock_trading_bot.visualization.protocols import VisualizationBackendProtocol
from stock_trading_bot.visualization.data_buffer import DataBuffer

class DashBackend(VisualizationBackendProtocol):
    def __init__(self, data_buffer: DataBuffer, title: str="Dash Application") -> None:
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
        logging.getLogger('werkzeug').setLevel(logging.ERROR)

        # State tracking attributes
        self.has_logged_no_data = False
        self.last_timestamp = None


        self.setup_layout()
        self.setup_callbacks()

    def setup_layout(self) -> None:
        """
        Sets up the Dash application layout.
        """
        self.app.layout = html.Div([
            html.H1(self.title),
            dcc.Graph(id='live-graph'),
            dcc.Interval(
                id='graph-update',
                interval=1*1000,  # Update every second
                n_intervals=0
            )
        ])

    def setup_callbacks(self) -> None:
        """
        Sets up the Dash callbacks for updating the graph.
        """
        @self.app.callback(
            Output('live-graph', 'figure'),
            [Input('graph-update', 'n_intervals')]
        )
        def update_graph_live(n: int) -> go.Figure:
            try:
                # buffer_size = len(self.data_buffer)
                # logger.debug(f"DashBackend: Buffer size is {buffer_size}.")

                if self.data_buffer.is_empty():
                    if not self.has_logged_no_data:
                        logger.info("DashBackend: No data available to update graph.")
                        self.has_logged_no_data = True
                    return go.Figure()
                else:
                    # Reset the 'no data' flag if data is available
                    if self.has_logged_no_data:
                        self.has_logged_no_data = False

                    # Retrieve the latest timestamp
                    latest_timestamp = self.data_buffer.get_latest_timestamp()

                    # Check if new data has arrived
                    if latest_timestamp != self.last_timestamp:
                        logger.info("DashBackend: Graph updated with new data.")
                        self.last_timestamp = latest_timestamp
                    else:
                        # No new data; no logging needed
                        pass

                    # Extract data from the deque
                    data_snapshot = self.data_buffer.get_snapshot()
                    timestamps = [data['timestamp'] for data in data_snapshot]
                    bid_prices = [data['bid_price'] for data in data_snapshot]
                    ask_prices = [data['ask_price'] for data in data_snapshot]

                # Create the Plotly figure outside the lock to minimize lock holding time
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
                    title=self.title,
                    xaxis_title="Timestamp",
                    yaxis_title="Price (USD)",
                    template="plotly_dark",
                    margin=dict(l=40, r=40, t=40, b=40)
                )

                # logger.debug("DashBackend: Graph updated with new data.")
                return fig
            except Exception as e:
                logger.error(f"DashBackend: Error updating graph: {e}")
                return go.Figure()



    def run(self) -> None:
        """
        Runs the Dash server.
        """
        logger.info("DashBackend: Running Dash server...")
        self.app.run_server(debug=False, use_reloader=False)