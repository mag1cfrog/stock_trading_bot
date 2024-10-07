import dash
from dash import dcc, html
from dash.dependencies import Input, Output
from loguru import logger
import plotly.graph_objs as go
from typing import Deque

from ...protocols import VisualizationBackendProtocol

class DashBackend(VisualizationBackendProtocol):
    def __init__(self, data_buffer: Deque[dict], title: str="Dash Application") -> None:
        """
        Initializes the BaseDashApp with common Dash configurations.
        
        Args:
            data_buffer (Deque[dict]): Thread-safe deque containing the latest data points.
            title (str): The title of the Dash application.
        """
        self.data_buffer = data_buffer
        self.title = title
        self.app = dash.Dash(__name__)
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
        def update_graph_live(n):
            if not self.data_buffer:
                logger.debug("DashBackend: No data available to update graph.")
                return go.Figure()

            # Extract data from the deque
            timestamps = [data['timestamp'] for data in self.data_buffer]
            bid_prices = [data['bid_price'] for data in self.data_buffer]
            ask_prices = [data['ask_price'] for data in self.data_buffer]

            # Create the Plotly figure
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

            logger.debug("DashBackend: Graph updated with new data.")
            return fig

    def update_visualization(self, data_buffer: Deque[dict]) -> None:
        """
        Updates the visualization with new data.
        
        Args:
            data_buffer (Deque[dict]): The latest data points.
        """
        # Since Dash uses callbacks and the Interval component handles updates,
        # this method might not be necessary. However, it's included to conform to the protocol.
        pass

    def run(self) -> None:
        """
        Runs the Dash server.
        """
        logger.info("DashBackend: Running Dash server...")
        self.app.run_server(debug=False, use_reloader=False)