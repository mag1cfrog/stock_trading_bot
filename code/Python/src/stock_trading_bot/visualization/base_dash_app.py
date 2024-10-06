import dash
from dash import dcc, html
from dash.dependencies import Input, Output
from loguru import logger

class BaseDashApp:
    def __init__(self, title: str="Dash Application") -> None:
        """
        Initializes the BaseDashApp with common Dash configurations.
        
        Args:
            title (str): The title of the Dash application.
        """
        self.app = dash.Dash(__name__)
        self.app.title = title

        # Define a basic layout that can be extended or overridden
        self.app.layout = html.Div([
            html.H1(title),
            dcc.Graph(id='base-graph'),
            dcc.Interval(
                id='base-interval',
                interval=5*1000,  # 5 seconds
                n_intervals=0
            )
        ])

        # Register base callbacks if any (optional)
        self.register_base_callbacks()

    def register_base_callbacks(self) -> None:
        """
        Registers callbacks common to all Dash apps.
        Override or extend in child classes as needed.
        """
        @self.app.callback(
            Output('base-graph', 'figure'),
            [Input('base-interval', 'n_intervals')]
        )
        def update_base_graph(n: int) -> dict:
            """
            A placeholder callback for the base graph.
            Child classes should override this method.
            
            Args:
                n (int): Interval count.
            
            Returns:
                plotly.graph_objs.Figure: An empty figure.
            """
            logger.debug("BaseDashApp: update_base_graph called.")
            return {
                "data": [],
                "layout": {}
            }

    def run(self, use_debug: bool=True, if_use_reloader: bool=False) -> None:
        """
        Runs the Dash server.
        
        Args:
            debug (bool): Whether to run the Dash app in debug mode.
            use_reloader (bool): Whether to use the Flask reloader.
        """
        self.app.run_server(debug=use_debug, use_reloader=if_use_reloader)