import os
from datetime import datetime
from alpaca.data.historical import StockHistoricalDataClient
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame

api_key, secret_key = os.getenv('ALPACA_API_KEY'), os.getenv('ALPACA_SECRET_KEY')

client = StockHistoricalDataClient(api_key, secret_key)

request_params = StockBarsRequest(
                        symbol_or_symbols=["NVDA"],
                        timeframe=TimeFrame.Day,
                        start=datetime(2022, 7, 1),
                        end=datetime(2023, 9, 1)
                 )

bars = client.get_stock_bars(request_params)

print(bars.df)