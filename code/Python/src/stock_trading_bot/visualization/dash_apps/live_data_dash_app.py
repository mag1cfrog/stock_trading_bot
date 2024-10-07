from collections import deque

from dash import dcc, html
from dash.dependencies import Input, Output
from loguru import logger
import plotly.graph_objs as go


from ..backends.dash import BaseDashApp

class LiveDataDashApp(BaseDashApp):
    def __init__(self, data_buffer: deque, live_app_title: str="Real-Time Data Dash App", symbol: str="BTC/USD") -> None:
        """
        Initializes the LiveDataDashApp with live data visualization capabilities.
        
        Args:
            data_buffer (deque): Thread-safe deque containing the latest data points.
            live_app_title (str): The title of the Dash application.
            symbol (str): The cryptocurrency symbol being visualized.
        """
        super().__init__(title=live_app_title)
        self.data_buffer = data_buffer
        self.symbol = symbol

        # Override or extend the layout if necessary
        self.app.layout = self.create_layout()

        # Register live data specific callbacks
        self.register_live_data_callbacks()

    def create_layout(self) -> html.Div:
        """
        Creates a specialized layout for live data visualization.
        
        Returns:
            dash.html.Div: The layout for the live data Dash app.
        """
        return html.Div([
            html.H1(f"Real-Time {self.symbol} Price"),
            dcc.Graph(id='live-crypto-graph'),
            dcc.Interval(
                id='graph-update',
                interval=1*1000,  # 1 second
                n_intervals=0
            )
        ])

    def register_live_data_callbacks(self) -> None:
        """
        Registers callbacks specific to live data visualization.
        Overrides the base class callbacks if necessary.
        """
        @self.app.callback(
            Output('live-crypto-graph', 'figure'),
            [Input('graph-update', 'n_intervals')]
        )
        def update_live_graph(n: int) -> go.Figure:
            """
            Updates the Plotly graph with the latest data from the deque.
            
            Args:
                n (int): Interval count.
            
            Returns:
                plotly.graph_objs.Figure: Updated graph figure.
            """
            if not self.data_buffer:
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
                title=f"Real-Time {self.symbol} Bid and Ask Prices",
                xaxis_title="Timestamp",
                yaxis_title="Price (USD)",
                xaxis=dict(range=[min(timestamps), max(timestamps)]),
                yaxis=dict(range=[min(bid_prices + ask_prices) * 0.99, max(bid_prices + ask_prices) * 1.01]),
                template="plotly_dark",
                margin=dict(l=40, r=40, t=40, b=40)
            )

            logger.debug("LiveDataDashApp: Graph updated with new data.")
            return fig
