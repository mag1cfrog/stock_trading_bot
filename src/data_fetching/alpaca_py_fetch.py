import os
import time
import logging
import itertools
import json
from datetime import datetime, timedelta
from alpaca.data.historical import StockHistoricalDataClient
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame




def test_alpaca_api_with_dynamic_time_range(symbol="NVDA", timeframe=TimeFrame.Day, duration=60, days_back=1, increment_by_days=2):

    
    api_key = os.getenv('ALPACA_API_KEY')
    secret_key = os.getenv('ALPACA_SECRET_KEY')

    client = StockHistoricalDataClient(api_key, secret_key)

    os.makedirs('./data/landing', exist_ok=True)

    # Define initial time range
    end_date = datetime.now() - timedelta(days=days_back)
    start_date = end_date - timedelta(days=increment_by_days)

    # Begin test
    start_time = time.time()
    end_time = start_time + duration
    calls = 0


    while time.time() < end_time:
        try:
            request_params = StockBarsRequest(
                symbol_or_symbols=[symbol],
                timeframe=timeframe,
                start=start_date,
                end=end_date
            )
            response = client.get_stock_bars(request_params)
            calls += 1
            file_path = f'./data/landing/response_{calls}.json'
            # response.df.to_json(file_path, orient='records')
            data = itertools.chain.from_iterable(response.dict().values())
            with open(file_path, 'w') as f:
                json.dump(list(data), f, default=str)
            logging.info(f"API call {calls} successful, data saved to {file_path}")

            # Update time range for next call
            start_date -= timedelta(days=increment_by_days)
            end_date -= timedelta(days=increment_by_days)

        except Exception as e:
            logging.error(f"Error on request {calls}: {str(e)}")
            break

    logging.info(f"Total API calls made: {calls}")


def test_alpaca_api_with_longest_time_range(symbol="NVDA", timeframe=TimeFrame.Day, duration=60, time_back=1):

    
    api_key = os.getenv('ALPACA_API_KEY')
    secret_key = os.getenv('ALPACA_SECRET_KEY')

    client = StockHistoricalDataClient(api_key, secret_key)

    os.makedirs('./data/landing', exist_ok=True)
    

    # Define initial time range
    end_date = datetime.now() - timedelta(days=1)
    # This alpaca api can get data since 2016
    start_date = datetime(2016, 1, 1)
    

    request_params = StockBarsRequest(
        symbol_or_symbols=[symbol],
        timeframe=timeframe,
        start=start_date,
        end=end_date
    )

    # Begin test
    start_time = time.time()
    end_time = start_time + duration
    calls = 0


    while time.time() < end_time:
        try:

            response = client.get_stock_bars(request_params)
            calls += 1
            file_path = f'./data/landing/response_{calls}.json'
            # response.df.to_json(file_path, orient='records')
            data = itertools.chain.from_iterable(response.dict().values())
            with open(file_path, 'w') as f:
                json.dump(list(data), f, default=str)
            logging.info(f"API call {calls} successful, data saved to {file_path}")

        except Exception as e:
            logging.error(f"Error on request {calls}: {str(e)}")
            break

    logging.info(f"Total API calls made: {calls}")




def main():
    os.makedirs('./logs', exist_ok=True)
    # Setup logging to file with timestamp in filename
    logging.basicConfig(level=logging.INFO,
                    format='%(asctime)s - %(levelname)s - %(message)s',
                    filename=f'./logs/alpaca_api_test_{datetime.now().strftime("%Y%m%d_%H%M%S")}.log',
                    filemode='a')  # Append mode
        
    test_alpaca_api_with_longest_time_range()


if __name__ == "__main__":
    main()