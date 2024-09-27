# stock_trading_bot

## Overview
This project aims to develop an automated stock trading system that can autonomously download stock data, analyze it using predefined algorithms, and execute trades based on the analyzed results. The system is designed to leverage the best tools and technologies for each task, ensuring high performance, reliability, and accuracy in trading decisions.

## Key Features
- **Real-Time Data Fetching**: Uses Alpaca's API to fetch stock market data. Both Python and Go SDKs have been tested for performance and suitability for high-frequency API calls.
  
- **Algorithmic Trading**: The system is designed to execute trades based on real-time analysis of market data using predefined trading strategies.

- **Asynchronous Data Handling**: Python's async capabilities have been implemented to enhance the efficiency of API interactions for I/O-bound tasks like data fetching and processing.

- **Custom Data Storage with DuckDB**: A custom DuckDB manager was developed to efficiently manage stock data storage operations. This lightweight solution simplifies data management by eliminating the overhead of distributed storage systems like Apache Iceberg, while still providing high performance on a single node.

## Development Progress

### API Performance Testing
Multiple approaches were tested to determine the best way to fetch real-time stock data:

- **Alpaca SDK vs. Custom Solutions**:
   - While our custom Python and Go asynchronous implementations significantly outperformed the official Alpaca SDK (achieving up to 402 API calls per minute), we encountered an issue where our IP was temporarily banned due to excessive API requests.
   - To avoid further issues, we decided to stick with the official Alpaca SDK for now, which remains sufficient for our current needs and ensures compliance with API rate limits.

### Storage Solutions
- **Apache Iceberg (Initial Testing)**:
   - We successfully deployed a local Apache Iceberg system using PySpark + MinIO, and documented the steps in [this setup guide](./doc/setup_pyspark_iceberg.md).
   - However, the setup proved to be cumbersome, requiring a Spark cluster for even lightweight operations. Given the current data volume and computation requirements, Spark brings unnecessary overhead.

- **DuckDB Manager (Current Solution)**:
   - We developed a custom DuckDB-based storage system specifically for stock data. This solution operates efficiently on a single node, significantly reducing complexity and overhead compared to distributed solutions like Spark. The DuckDB manager handles common stock data operations such as data snapshots, querying, and cleanup, allowing for easier integration with our trading logic.

## Next Steps
- **Stock Trading Strategy Exploration**: Begin researching and testing various stock trading strategies. We aim to implement strategies that are optimized for the current system and can leverage both real-time and historical data for decision-making.
  
- **Enhanced Data Visualization**: Explore ways to better visualize stock data to assist in both strategy development and real-time monitoring. This could include finding optimized visualization libraries or even building custom visualization tools tailored to our specific data needs.

- **System Optimization**: Continue exploring ways to optimize both Python and Go components for better performance in high-frequency trading scenarios.

## License
This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Documentation
- [Development Log](./doc/dev-log/index.md)
