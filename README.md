# stock_trading_bot

## Overview
This project aims to develop an automated stock trading system that can autonomously download stock data, analyze it using predefined algorithms, and execute trades based on the analyzed results. The system is designed to leverage the best tools and technologies for each task, ensuring high performance, reliability, and accuracy in trading decisions.

## Current Status
As of now, the project is in the initial stages of development. The primary focus is on setting up the data ingestion module. We are currently benchmarking Python and Go to determine the most effective language for API data fetching, specifically using the Alpaca API for real-time and historical market data.

## Goals
- **Data Ingestion**: Establish a robust mechanism to fetch and store stock market data efficiently using the Alpaca API.
- **Data Storage**: Design and implement a storage solution that optimizes data retrieval and manipulation for trading analysis.
- **Data Analysis**: Develop algorithms to analyze stock data and generate trading signals. This stage will focus on the accuracy and computational efficiency of the algorithms.
- **Trade Execution**: Implement a trading module that can execute trades based on analysis results in real-time with high reliability.

## Technologies
- **API**: Alpaca API for fetching stock market data.
- **Programming Languages**: Python is used for the majority of the system design and implementation due to its extensive library support and ease of use. Go is being benchmarked for tasks that require high performance, such as API data fetching.
- **Data Storage**: The format and storage system are yet to be determined but will focus on efficiency and speed.

## License
This project is licensed under the MIT License - see the [LICENSE.md](./LICENSE) file for details.

## Documentation
- [Development Log](./doc/dev-log/index.md)