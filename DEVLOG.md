# Development Log for Automated Stock Trading System

This log documents the development process, decisions, challenges, and progress of the automated stock trading system project.

## 2024-06-04

### Introduction
Today marks the official start of the automated stock trading system project. The main goal of this project is to develop a system that can autonomously download stock data, analyze it using pre-defined algorithms, and execute trades based on the analysis.

### Initial Setup
- Created the GitHub repository to host the project code and documentation.
- Set up the basic folder structure accommodating Python and Go codebases.
- Configured the `.gitignore` file suitable for Python and Go development environments.

### Goals
- **Benchmarking Performance**: The first technical objective is to benchmark the API fetching capabilities of Python and Go. This will help determine which programming language is more suited for handling real-time stock data efficiently.
- **Project Documentation**: Begin documenting the development process systematically in this DEVLOG.md to track progress, decisions, and learnings.

### Next Steps
- ~~Research and select the stock market APIs that will be used for fetching the data.~~ We would use Alpaca as the stock data API source.
- Prepare initial benchmarking scripts for both Python and Go to compare performance in terms of speed and resource utilization.
- Outline the criteria for evaluating the benchmark results.

### Challenges Anticipated
- Anticipating potential challenges with rate limiting of stock market APIs during high-frequency data fetching.
- Uncertainties about the optimal concurrency model in Python for managing simultaneous API calls efficiently.

### Reflection
Setting clear, measurable goals at the outset is crucial for maintaining focus and assessing progress. The setup phase is crucial for ensuring that the technical infrastructure is in place for the upcoming development challenges.

## Development Log Entry - 2024-06-05

### Testing the Alpaca-py SDK Performance

#### Objective:
To test the performance of the official `alpaca-py` SDK to determine its efficiency in making API calls and to understand how it handles various request sizes under the free tier's documented rate limits.

#### Methodology:
- Conducted two sets of tests to measure the number of API calls that could be completed within a minute using the `alpaca-py` SDK.
- The first test used small time ranges for the requests, aiming to maximize the number of calls and test the API's response to rapid successive calls.
- The second test used longer time ranges for the requests to see how the SDK handles larger data volumes and to assess the impact of JSON file writing on the call rate.

#### Findings:
- **High Performance on Small Time Ranges**: Surprisingly, when testing with small time ranges, the SDK was able to perform over 380 API calls within a minute, significantly exceeding the documented limit of 200 calls per minute. This suggests that the SDK and the API can handle a higher throughput under optimal conditions than the official documentation states.
- **Bottleneck on Larger Requests**: When the time range was increased, the number of API calls achievable within a minute dropped to just over 110. This reduction is likely due to the increased data volume per request and the time taken to write this data to JSON files, rather than a limitation of the API call rate itself.

#### Conclusion:
The `alpaca-py` SDK is capable of handling a high frequency of API calls efficiently when dealing with small data requests. However, performance bottlenecks may occur when dealing with larger data volumes, primarily due to the time required to process and store the data rather than the API's intrinsic limitations.

### Next Steps:
- Investigate methods to optimize JSON file writing or explore the use of more efficient data storage formats (e.g., binary formats like Parquet) to alleviate the bottlenecks associated with large data volumes.
- Continue monitoring SDK performance and API limits to ensure efficient utilization and compliance with Alpaca's service terms.
