# Changelog
All notable changes to market_data_ingestor will be documented in this file.
## market_data_ingestor-v1.2.0


### Bug Fixes

- Update pyo3 dependency to be optional and clean up feature definitions ([e631ac4](https://github.com/mag1cfrog/stock_trading_bot/commit/e631ac43254ba7ed2c07450d3d77680905470539))

- Add conditional compilation for alpaca-python-sdk in module and initialization ([32d3bcf](https://github.com/mag1cfrog/stock_trading_bot/commit/32d3bcf672dd06823fcdcd45b3111a6822877934))

- Add conditional compilation for alpaca-python-sdk in StockBarData methods ([c21b64b](https://github.com/mag1cfrog/stock_trading_bot/commit/c21b64b8f107b362e26a3ddad4bf21cf3ce97287))

- Add conditional compilation for alpaca-python-sdk in batch and single request files ([0b7a512](https://github.com/mag1cfrog/stock_trading_bot/commit/0b7a512bba904e1014663f08a39058d11b77ce0d))

- Ensure conditional compilation for PyErr in MarketDataError conversions ([b22745f](https://github.com/mag1cfrog/stock_trading_bot/commit/b22745f18b9efe8b255197d6e12b0fd8d3019e58))

- Add conditional compilation for alpaca-python-sdk in TimeFrame implementations ([4c11d67](https://github.com/mag1cfrog/stock_trading_bot/commit/4c11d67892088529f69a5ea3af9f43330494e6a3))

- Add conditional compilation for alpaca-python-sdk in StockBarsParams struct ([95855e0](https://github.com/mag1cfrog/stock_trading_bot/commit/95855e008b0d4a6babf22d165d2d9b94813561ca))

- Update conditional compilation for CLI and alpaca-python-sdk in main function ([818a1a5](https://github.com/mag1cfrog/stock_trading_bot/commit/818a1a579bd556701f36bcd5f43c21ced614f9f5))

- Ensure conditional compilation for alpaca-python-sdk in StockBarData and Config imports ([89ad9e9](https://github.com/mag1cfrog/stock_trading_bot/commit/89ad9e939b309ba0291e7d4fcedbe132103272cb))

- Ensure conditional compilation for alpaca-python-sdk in single_request and batch_request modules ([cc7e227](https://github.com/mag1cfrog/stock_trading_bot/commit/cc7e22746ae8d422392eac26e618d95d0d4a00a6))

- Update conditional compilation for alpaca-python-sdk in lib, stockbars, timeframe, and historical request modules ([a68b368](https://github.com/mag1cfrog/stock_trading_bot/commit/a68b3686a74c5485f133a4aea37b5b7a8faa6945))


### Maintenance

- Release ([c4892c6](https://github.com/mag1cfrog/stock_trading_bot/commit/c4892c66bcbef00c827ef542169813f156ea0c2f))

## market_data_ingestor-v1.1.1


### Maintenance

- Release ([17ba064](https://github.com/mag1cfrog/stock_trading_bot/commit/17ba064b11db6d0be5d26d4d231d78d0041eda21))

## market_data_ingestor-v1.1.0


### Code Refactoring

- Clean up error handling and improve code formatting in market data ingestion ([cb5e2cf](https://github.com/mag1cfrog/stock_trading_bot/commit/cb5e2cfe3d47ba58aa3f86539d73957353b90810))

- Replace MarketDataError with IngestorError in StockBarData methods ([ef6f164](https://github.com/mag1cfrog/stock_trading_bot/commit/ef6f164e783cd281cf368c367ebf5feabf7f0b09))

- Clean up whitespace and formatting in historical market data module ([bef7dc1](https://github.com/mag1cfrog/stock_trading_bot/commit/bef7dc1132a7bbcff3bed07119a1729ee27e2d1c))


### Documentation

- Add module documentation for historical market data fetching functionality ([5848771](https://github.com/mag1cfrog/stock_trading_bot/commit/5848771aa32772baaf5f63e510ee0a67b0a12b0c))


### Features

- Add enhanced API methods for fetching historical bars data to memory and files ([55a7be7](https://github.com/mag1cfrog/stock_trading_bot/commit/55a7be7e2a6b7dce1a5b237ab3d4a8ea4543708f))

- Introduce custom error handling for market data ingestion and I/O operations ([61365ad](https://github.com/mag1cfrog/stock_trading_bot/commit/61365ad20a3fbce84223e5704420123a225ed108))

- Add support for creating StockBarData client with direct configuration ([1153725](https://github.com/mag1cfrog/stock_trading_bot/commit/115372595f46ab97ce7030502175aad1c1740eb6))

- Add CLI feature support for market data ingestor ([639fd7e](https://github.com/mag1cfrog/stock_trading_bot/commit/639fd7e197a3d33ed4ef83dc18a0c07e9ea19c82))


### Maintenance

- Update dependencies for bincode and pyo3 to latest versions ([f12b274](https://github.com/mag1cfrog/stock_trading_bot/commit/f12b274ed4d05ebc02bf9d8ee7baf8228afe6eff))

- Release ([07f6828](https://github.com/mag1cfrog/stock_trading_bot/commit/07f68282c1491874960f6bc9cc1d2347d47a9287))

## market_data_ingestor-v1.0.20


### Bug Fixes

- Update pre-release hook to include correct source path for changelog generation ([595b5f6](https://github.com/mag1cfrog/stock_trading_bot/commit/595b5f65549b4a45555fbab301dbe31b59bb3cbd))


### Maintenance

- Release ([74c1cd4](https://github.com/mag1cfrog/stock_trading_bot/commit/74c1cd494c3fb2242795b7e2dbe390287cf1a73a))

## market_data_ingestor-v1.0.19


### Bug Fixes

- Disable filtering of unconventional commit messages ([2102e92](https://github.com/mag1cfrog/stock_trading_bot/commit/2102e923a10bcd3d8769b9110c2b9e14f7b1a380))


### Maintenance

- Release ([a419e22](https://github.com/mag1cfrog/stock_trading_bot/commit/a419e22f49451b087c6e6f7fc156188550486bb1))

