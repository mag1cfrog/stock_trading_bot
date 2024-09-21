# stock_trading_bot

## Overview
This project aims to develop an automated stock trading system that can autonomously download stock data, analyze it using predefined algorithms, and execute trades based on the analyzed results. The system is designed to leverage the best tools and technologies for each task, ensuring high performance, reliability, and accuracy in trading decisions.

## Key Features
- **Real-Time Data Fetching**: Uses Alpaca's API to fetch stock market data. Both Python and Go SDKs have been tested for performance and suitability for high-frequency API calls.

- **Algorithmic Trading**: The system is designed to execute trades based on real-time analysis of market data using predefined trading strategies.

- **Asynchronous Data Handling**: Python's async capabilities have been implemented to enhance the efficiency of API interactions for I/O-bound tasks like data fetching and processing.


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

- **Planned Transition to DuckDB**:
   - We plan to replace Apache Iceberg with a home-built storage system based on DuckDB. The new system will be designed to handle regular stock data operations efficiently on a single node, reducing overhead and simplifying usage.
   - The goal is to create a lightweight and easy-to-use storage solution that meets our needs without the complexity of distributed systems like Spark.

## Next Steps
- **Data Storage with DuckDB**: Begin building a custom storage solution using DuckDB to manage stock data efficiently.

- **Trading Logic Implementation**: Work on implementing and testing the core trading algorithms based on real-time data.

- **System Optimization**: Continue exploring ways to optimize both Python and Go components for better performance in high-frequency trading scenarios.


## License
This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Documentation
- [Development Log](./doc/dev-log/index.md)