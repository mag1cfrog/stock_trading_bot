# Development Log for Automated Stock Trading System- 



- [Development Log for Automated Stock Trading System-](#development-log-for-automated-stock-trading-system-)
  - [2024-06-04](#2024-06-04)
    - [Introduction](#introduction)
    - [Initial Setup](#initial-setup)
    - [Goals](#goals)
    - [Next Steps](#next-steps)
    - [Challenges Anticipated](#challenges-anticipated)
    - [Reflection](#reflection)
  - [Development Log Entry - 2024-06-05](#development-log-entry---2024-06-05)
    - [Testing the Alpaca-py SDK Performance](#testing-the-alpaca-py-sdk-performance)
      - [Objective:](#objective)
      - [Methodology:](#methodology)
      - [Findings:](#findings)
      - [Conclusion:](#conclusion)
    - [Next Steps:](#next-steps-1)
  - [2024-06-06: Performance Testing of Alpaca Official GO SDK](#2024-06-06-performance-testing-of-alpaca-official-go-sdk)
    - [Objective](#objective-1)
    - [Methodology](#methodology-1)
    - [Results](#results)
    - [Comparison with Python SDK](#comparison-with-python-sdk)
    - [Conclusion](#conclusion-1)
    - [Next Steps](#next-steps-2)
  - [2024-06-07: Asynchronous API Call Performance Testing and Future Planning](#2024-06-07-asynchronous-api-call-performance-testing-and-future-planning)
    - [Objective](#objective-2)
    - [Methodology](#methodology-2)
    - [Results](#results-1)
    - [Conclusion](#conclusion-2)
    - [Next Steps](#next-steps-3)
  - [2024-06-09: Testing Customized Go Program for API Fetching](#2024-06-09-testing-customized-go-program-for-api-fetching)
    - [Test Setup and Findings:](#test-setup-and-findings)
    - [Observations:](#observations)
    - [Conclusions:](#conclusions)


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


## 2024-06-06: Performance Testing of Alpaca Official GO SDK

### Objective
Evaluate and compare the performance of Alpaca's official GO SDK against the previously tested Python SDK using similar parameters.

### Methodology
Performed API call tests using the official GO SDK for Alpaca. Two distinct parameter sets were used to measure the performance under different data request sizes:

1. **Small Time Range Test** — This test aimed to maximize the number of API calls within a minute, simulating a high-frequency data fetching scenario.
2. **Large Time Range Test** — This test involved fetching a larger span of data per call, expected to assess the SDK's handling of more substantial data loads.

### Results
- **Small Time Range Test**: Achieved **397 API calls** within one minute.
- **Large Time Range Test**: Achieved **133 API calls** within one minute.

### Comparison with Python SDK
The performance difference between the GO SDK and the Python SDK is relatively minimal. Both SDKs showed similar capabilities in handling high-frequency and high-volume data requests, though there was a slight edge with the GO SDK in the high-frequency scenario.

### Conclusion
The Alpaca GO SDK demonstrates robust performance, slightly outperforming the Python SDK in scenarios demanding higher API call frequencies. Both SDKs are adequately equipped to handle substantial data volumes, indicating efficient handling of larger data ranges without significant performance degradation.

### Next Steps
- Explore optimization opportunities in the usage of both SDKs to further enhance performance.
- Consider detailed profiling to identify any underlying bottlenecks or inefficiencies in data handling.

## 2024-06-07: Asynchronous API Call Performance Testing and Future Planning

### Objective
To evaluate the performance of asynchronous API calls using a custom Python implementation and compare it with the performance of Alpaca's official SDK under the same conditions.

### Methodology
Implemented an asynchronous method in Python to make continuous API calls to Alpaca's data endpoint for a specified large time range. This test was conducted to measure the number of API calls that could be successfully made within one minute.

### Results
- **Asynchronous Python Implementation**: Managed to achieve approximately **160 API calls** per minute.
- **Alpaca Official SDK**: Reached around **130 API calls** per minute using the same parameters and conditions.

### Conclusion
The custom asynchronous Python approach demonstrated a higher throughput in terms of API calls per minute compared to the official SDK. This improvement underscores the potential benefits of using native Python asynchronous capabilities to enhance API interaction efficiency, particularly for I/O-bound tasks.

### Next Steps
- **Planning and Implementing Data Storage**: The next phase will focus on developing an efficient structure for initial data storage. Despite the custom implementation's higher performance, we will utilize Alpaca's official SDK for data extraction due to its ease of use. This decision allows us to focus more on building and optimizing the overall system architecture.
- **System Architecture Development**: Prioritize the design and implementation of the system's big picture, ensuring that data handling and storage are optimized for scalability and performance.


## 2024-06-09: Testing Customized Go Program for API Fetching

Today, we conducted extensive testing on our customized Go program designed to fetch data from Alpaca's API. Our objective was to compare its performance against our previously implemented Python async method under similar conditions.

### Test Setup and Findings:
- **Connection Method:** Due to an IP ban presumably from excessive testing, we utilized a VPN, which slightly reduced our connection speed.
- **Pool Size Variance:** We experimented with different pool sizes in the Go program:
  - Size = 1: 396 calls/min
  - Size = 2: 359 calls/min
  - Size = 3: 402 calls/min
  - Size = 4: 272 calls/min
  - Size = 5: 402 calls/min (with errors indicating "retries exceeded")

### Observations:
- The Go program's performance was not linear with the pool size, which suggests that there might be optimal concurrency levels for API calling that do not simply scale with the number of workers.
- Despite the reduced network performance due to the VPN, the Go program significantly outperformed the Python async script, making around 400 calls per minute compared to Python's 145 calls per minute.

### Conclusions:
Although the customized Go program demonstrates superior performance, indicating a substantial advantage in using Go for API fetching tasks, we have decided to use this knowledge as a proof of concept rather than immediately implementing these customizations. For now, our focus will shift towards building a more comprehensive and general system. The insights gained will guide future optimizations but are not a priority in the current phase of development.


