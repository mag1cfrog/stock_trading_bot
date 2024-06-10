import os
import time
import logging
import itertools
from concurrent.futures import ThreadPoolExecutor
import json
import threading
from datetime import datetime, timedelta
from alpaca.data.historical import StockHistoricalDataClient
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame




# def test_alpaca_api_with_dynamic_time_range(symbol="NVDA", timeframe=TimeFrame.Day, duration=60, days_back=1, increment_by_days=2):

    
#     api_key = os.getenv('APCA_API_KEY_ID')
#     secret_key = os.getenv('APCA_API_SECRET_KEY')

#     client = StockHistoricalDataClient(api_key, secret_key)

#     os.makedirs('./data/landing', exist_ok=True)

#     # Define initial time range
#     end_date = datetime.now() - timedelta(days=days_back)
#     start_date = end_date - timedelta(days=increment_by_days)

#     # Begin test
#     start_time = time.time()
#     end_time = start_time + duration
#     calls = 0


#     while time.time() < end_time:
#         try:
#             request_params = StockBarsRequest(
#                 symbol_or_symbols=[symbol],
#                 timeframe=timeframe,
#                 start=start_date,
#                 end=end_date
#             )
#             response = client.get_stock_bars(request_params)
#             calls += 1
#             file_path = f'./data/landing/response_{calls}.json'
#             # response.df.to_json(file_path, orient='records')
#             data = itertools.chain.from_iterable(response.dict().values())
#             with open(file_path, 'w') as f:
#                 json.dump(list(data), f, default=str)
#             logging.info(f"API call {calls} successful, data saved to {file_path}")

#             # Update time range for next call
#             start_date -= timedelta(days=increment_by_days)
#             end_date -= timedelta(days=increment_by_days)

#         except Exception as e:
#             logging.error(f"Error on request {calls}: {str(e)}")
#             break

#     logging.info(f"Total API calls made: {calls}")


# def call_api_for_duration(client, request_params, duration):
#     end_time = time.time() + duration
#     calls = 0

#     while time.time() < end_time:
#         try:
#             response = client.get_stock_bars(request_params)
#             calls += 1
#             file_path = f'./data/landing/response_{calls}.json'
#             data = itertools.chain.from_iterable(response.dict().values())
#             # with open(file_path, 'w') as f:
#             #     json.dump(list(data), f, default=str)
#             logging.info(f"API call {calls} successful, data saved to {file_path}")

#         except Exception as e:
#             logging.error(f"Error on request {calls}: {str(e)}")
#             break

#     logging.info(f"Total API calls made: {calls}")


# def test_alpaca_api_with_longest_time_range(symbol="NVDA", timeframe=TimeFrame.Day, duration=60, time_back=1):

    
#     api_key = os.getenv('APCA_API_KEY_ID')
#     secret_key = os.getenv('APCA_API_SECRET_KEY')

#     client = StockHistoricalDataClient(api_key, secret_key)

#     os.makedirs('./data/landing', exist_ok=True)
    

#     # Define initial time range
#     end_date = datetime.now() - timedelta(days=1)
#     # This alpaca api can get data since 2016
#     start_date = datetime(2016, 1, 1)
    

#     request_params = StockBarsRequest(
#         symbol_or_symbols=[symbol],
#         timeframe=timeframe,
#         start=start_date,
#         end=end_date
#     )

#     # Create a separate thread for calling the API
#     api_thread = threading.Thread(target=call_api_for_duration, args=(client, request_params, duration))
#     api_thread.start()
#     api_thread.join()  # Wait for the thread to finish if necessary


def call_api(client, request_params, stop_event):
    calls = 0

    while not stop_event.is_set():
        try:
            response = client.get_stock_bars(request_params)
            calls += 1
            json_file_path = f'./data/landing/response_{calls}.json'
            parquet_file_path = f'./data/landing/response_{calls}.parquet'
            data_df = response.df.reset_index(drop=False)
            # Save data to parquet file
            data_df.to_parquet(parquet_file_path)
            
            # Save data to json file
            # data_dict = data_df.to_dict(orient='records')
            # with open(json_file_path, 'w') as f:
            #     json.dump(list(data_dict), f, default=str)

            logging.info(f"API call {calls} successful, data saved to {json_file_path}")
        except Exception as e:
            logging.error(f"Error on request {calls}: {str(e)}")
            break

    logging.info(f"Total API calls made: {calls}")

def test_alpaca_api_with_longest_time_range(symbol="NVDA", timeframe=TimeFrame.Day, duration=60):
    api_key = os.getenv('APCA_API_KEY_ID')
    secret_key = os.getenv('APCA_API_SECRET_KEY')
    client = StockHistoricalDataClient(api_key, secret_key)
    os.makedirs('./data/landing', exist_ok=True)
    end_date = datetime.now() - timedelta(weeks=1)
    start_date = datetime(2016, 1, 1)
    request_params = StockBarsRequest(
        symbol_or_symbols=[symbol],
        timeframe=timeframe,
        start=start_date,
        end=end_date
    )

    # Create a thread for calling the API
    stop_event = threading.Event()
    api_thread = threading.Thread(target=call_api, args=(client, request_params, stop_event))
    api_thread.start()
    
    # Allow the thread to run for the specified duration
    time.sleep(duration)
    stop_event.set()  # Signal the thread to stop making API calls
    api_thread.join()  # Wait for the thread to finish


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