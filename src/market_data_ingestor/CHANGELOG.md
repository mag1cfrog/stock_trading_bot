# Changelog
All notable changes to market_data_ingestor will be documented in this file.
## market_data_ingestor-v2.0.1


### Code Refactoring

- Reorganize market data ingestor module structure and remove unused components ([d1e2de9](https://github.com/mag1cfrog/stock_trading_bot/commit/d1e2de9c8262e308c7c4a3ae1506be175ff732bf))

- Remove StockBarsParams struct and related imports ([b4cd09c](https://github.com/mag1cfrog/stock_trading_bot/commit/b4cd09c2ea17baf85121723f5a6077029dbca942))

- Remove stockbars module from market data ingestor ([8d1fd55](https://github.com/mag1cfrog/stock_trading_bot/commit/8d1fd55e5302512b69faed9187c5bab4c161dc55))

- Update BarSeries references and remove bar_series module ([de3b900](https://github.com/mag1cfrog/stock_trading_bot/commit/de3b9003b10c52ce93b064a7544e247b4d8c9b9c))


### Documentation

- Update changelog for major release ([f830855](https://github.com/mag1cfrog/stock_trading_bot/commit/f830855b78ee9d6e67c5f54b46b057b082188b2e))


### Maintenance

- Release ([dadbbc2](https://github.com/mag1cfrog/stock_trading_bot/commit/dadbbc2cd8f5541d75ce00bb5a2336ac93d26cad))

## market_data_ingestor-v2.0.0


### Bug Fixes

- Update indexmap dependency to use workspace configuration ([931d85b](https://github.com/mag1cfrog/stock_trading_bot/commit/931d85b9b848773877e5e6c24bc1776f8d1b4c43))


### Code Refactoring

- Remove python related code ([ecb0ae9](https://github.com/mag1cfrog/stock_trading_bot/commit/ecb0ae92594dfef14541537147f3093639484e7c))


### Documentation

- Update changelog for patch release ([30ed93f](https://github.com/mag1cfrog/stock_trading_bot/commit/30ed93f9a1aea54e194ad4a962f919cba59fc89c))


### Maintenance

- Release ([ff07976](https://github.com/mag1cfrog/stock_trading_bot/commit/ff079767292853bad09eed27f13553af081cc846))


### Style

- Improve error logging format in Python initialization functions ([118e834](https://github.com/mag1cfrog/stock_trading_bot/commit/118e834e7933983f181ce3b49d3c4b746497d2c3))

## market_data_ingestor-v1.3.5


### Bug Fixes

- Improve error handling for 'uv' command in Python setup ([a9a3df1](https://github.com/mag1cfrog/stock_trading_bot/commit/a9a3df18470fbedd9bd8699145e7f8eb6a5a5258))

- Improve formatting and clarity of warning messages in build script ([a9bb085](https://github.com/mag1cfrog/stock_trading_bot/commit/a9bb085a512782263ffd0f074171128c9232672a))

- Simplify directory check in find_site_packages function ([b73a0cc](https://github.com/mag1cfrog/stock_trading_bot/commit/b73a0ccf231f7b068b33b7634a9bb1d2a8f37843))

- Correct default value for base_delay_ms in batch command tests ([16b3b05](https://github.com/mag1cfrog/stock_trading_bot/commit/16b3b05a8f69b93111fc8ec1b3905c9ed69a3c9b))

- Remove invalid error case tests for timeframe parsing ([99a2b59](https://github.com/mag1cfrog/stock_trading_bot/commit/99a2b5984eb1dd20dbd38b0e2839ac93ce5532b3))


### Documentation

- Update changelog for patch release ([2416ec7](https://github.com/mag1cfrog/stock_trading_bot/commit/2416ec7e7d97dc0f07608251ad3f3fda659262c8))

- Add README for release scripts with usage instructions and troubleshooting ([f0b1270](https://github.com/mag1cfrog/stock_trading_bot/commit/f0b1270860fb7278a73b8ea9e73d3de09e7c9bff))

- Add comprehensive README for market data ingestor with usage, configuration, and architecture details ([c382b0e](https://github.com/mag1cfrog/stock_trading_bot/commit/c382b0eac516baa7454bd91efa1787f63633d75e))


### Features

- Update dependencies for asset_sync and market_data_ingestor, add Range enum for time range specification ([1ac3e1e](https://github.com/mag1cfrog/stock_trading_bot/commit/1ac3e1e0ea958ead68bc20bc24903af345e06b71))

- Add serde rename_all attribute to AssetClass enum for consistent serialization ([2ba2063](https://github.com/mag1cfrog/stock_trading_bot/commit/2ba2063f3bfe469251415127cc8d373e8ef56985))


### Maintenance

- Comment out unused Rust analyzer features and environment variable ([c45450e](https://github.com/mag1cfrog/stock_trading_bot/commit/c45450ea312054b87f78bad86616ceb4bcc3561a))

- Release ([aa68fda](https://github.com/mag1cfrog/stock_trading_bot/commit/aa68fda9a2884465f17b2f9c13f650497c3a6ced))


### Style

- Format code for consistency and readability across multiple files ([13cbc62](https://github.com/mag1cfrog/stock_trading_bot/commit/13cbc626cdca0cd4331a47bd30143f898136c5d7))

## market_data_ingestor-v1.3.4


### Bug Fixes

- Update changelog generation command to include latest changes ([b6cdfdf](https://github.com/mag1cfrog/stock_trading_bot/commit/b6cdfdfbc9d860cfa3c4fa21d9f2dfe4524f9ef8))


### Maintenance

- Release ([f40c9f8](https://github.com/mag1cfrog/stock_trading_bot/commit/f40c9f8a8920a5100bc897a47befc43049c4fd6b))

## [Latest]


### Bug Fixes

- Update pre-release-hook to post-release-hook in Cargo.toml ([c0fb896](https://github.com/mag1cfrog/stock_trading_bot/commit/c0fb89617bd87344daaf65253f7901620baddc15))

- Correct post-release-hook to pre-release-hook in Cargo.toml ([2e31f26](https://github.com/mag1cfrog/stock_trading_bot/commit/2e31f2677b5153f576c82ea68ae90a9222695700))

## market_data_ingestor-v1.3.0


### Bug Fixes

- Update conditional compilation for alpaca-python-sdk in params and stockbars modules ([a2bbe73](https://github.com/mag1cfrog/stock_trading_bot/commit/a2bbe731b0ce5b0aac17546425ffea40f57f9c3b))

- Add TimeFrameUnit import for test module ([f5856e9](https://github.com/mag1cfrog/stock_trading_bot/commit/f5856e9f84fdfd5ff9088cd4af27b724d1207a89))

- Improve error handling in parse_timeframe function for invalid units ([fe03d30](https://github.com/mag1cfrog/stock_trading_bot/commit/fe03d302d3d86bd2d946e9f864068bddc2954044))

- Update rustdoc for fetch_bars method to return Vec<BarSeries> instead of Vec<Bar> ([7e9a395](https://github.com/mag1cfrog/stock_trading_bot/commit/7e9a39538dfbc5aeb3c89ff62d04f3502f0f29e4))

- Correct spelling of "alpaca" in log message for fetch_bars method ([bf535c7](https://github.com/mag1cfrog/stock_trading_bot/commit/bf535c780a54f8f60ac42361c8d033a65532d9a4))

- Update parse_timeframe function to accept lowercase month unit ([9ae92d3](https://github.com/mag1cfrog/stock_trading_bot/commit/9ae92d3336078282d8f97c8c485855c04b8663a2))

- Update parse_timeframe to return boxed TimeFrameError for invalid unit ([fe09e1b](https://github.com/mag1cfrog/stock_trading_bot/commit/fe09e1b8b3a1922e6c2cc9926b680ad8c2eb6f7e))

- Update example code in DataProvider documentation for consistency ([47f91ad](https://github.com/mag1cfrog/stock_trading_bot/commit/47f91ad0c1790417b50017ae435bf247eb0aa762))

- Ensure conditional compilation for CLI tests with alpaca-python-sdk feature ([c42e078](https://github.com/mag1cfrog/stock_trading_bot/commit/c42e078562dcb1f1552590aaa17f0bdcd5634c4f))

- Improve error messages for I/O and Polars errors in Error enum ([6f8d40a](https://github.com/mag1cfrog/stock_trading_bot/commit/6f8d40af2ae09612bf2bf763b46b13b18ba7e35e))

- Update bincode dependency configuration and improve timeframe parsing logic ([017e876](https://github.com/mag1cfrog/stock_trading_bot/commit/017e87697a59bd8d3f2d16a90f84b97dbe70fe54))

- Remove unused feature flag for alpaca-python-sdk ([63f4c12](https://github.com/mag1cfrog/stock_trading_bot/commit/63f4c12c0025d787783b46bae13e1b3cf0715690))

- Remove feature flag for alpaca-python-sdk and update timeframe initialization in tests ([271a318](https://github.com/mag1cfrog/stock_trading_bot/commit/271a31835dd0a4892c7d63d06278aab9fc77d35c))

- Update import path for MarketDataError and remove unnecessary unwrap in timeframe initialization ([c4d22e0](https://github.com/mag1cfrog/stock_trading_bot/commit/c4d22e021153dcd668c1ecbdca269a1243c971b1))

- Simplify error message formatting and improve readability in various modules ([f84a641](https://github.com/mag1cfrog/stock_trading_bot/commit/f84a641c2f5bfaf5f5558214a493643d69a5b17e))

- Improve string formatting in IOError display implementation ([9761c56](https://github.com/mag1cfrog/stock_trading_bot/commit/9761c56bcd54cd9025dde25432352e52bb6ad808))

- Remove feature flag for alpaca-python-sdk in utils and improve string formatting in error messages ([6725ac7](https://github.com/mag1cfrog/stock_trading_bot/commit/6725ac70d1500e6cf147976b9010b684743b1d92))

- Update .gitignore to exclude all .vscode files except settings.json and add settings.json for rust-analyzer configuration ([837e615](https://github.com/mag1cfrog/stock_trading_bot/commit/837e61570b0e4450fad599e764719ae08ed3e769))

- Update build script to install 'alpaca-py' instead of 'alpaca-trade-api' and change pip command to add ([9d05d7f](https://github.com/mag1cfrog/stock_trading_bot/commit/9d05d7f9699ea55e790639260872a41c55d16e49))

- Correct header key format for Alpaca API secret key ([33a9a6f](https://github.com/mag1cfrog/stock_trading_bot/commit/33a9a6f890ed387eea8a92849018f40581cb89f9))

- Update virtual environment creation command and improve test configuration handling ([30096f0](https://github.com/mag1cfrog/stock_trading_bot/commit/30096f02913a92bb4b5ab3e82a28fd226661cb22))

- Update virtual environment creation command and install dependencies correctly ([38de0ed](https://github.com/mag1cfrog/stock_trading_bot/commit/38de0ed3935c4e0e05245e1975eebdf4e5725575))

- Remove debug print statement for APCA_API_KEY_ID and ensure DataFrame index is reset ([76bc9c4](https://github.com/mag1cfrog/stock_trading_bot/commit/76bc9c40e2e7f05bf62d09d0bd3dd3b1e859cc0c))

- Enhance virtual environment initialization with robust library path handling and improved import error reporting ([39057a2](https://github.com/mag1cfrog/stock_trading_bot/commit/39057a21ebb6dccc2f342211a2801e40751c383d))

- Enhance DataFrame handling in dataframe_to_bar_series function and improve trade_count conversion logic ([34f0c83](https://github.com/mag1cfrog/stock_trading_bot/commit/34f0c8384547895a347af6043ac3f54b89f8d342))

- Suppress clippy warning for needless range loop in dataframe_to_bar_series function ([7c1fca9](https://github.com/mag1cfrog/stock_trading_bot/commit/7c1fca9ca15d32cc1c6ee065fa768bf95f9a415f))


### Code Refactoring

- Simplify TimeFrame constructor by removing validation logic ([c81ed04](https://github.com/mag1cfrog/stock_trading_bot/commit/c81ed04ec982a928dd21fcfaeb26fb452fe302a8))

- Add PartialEq, Eq, PartialOrd, and Ord traits to TimeFrameUnit and TimeFrame structs ([eccbe5e](https://github.com/mag1cfrog/stock_trading_bot/commit/eccbe5eaaaa8132127c9ba7b55dbf1c0926f8c3d))

- Remove symbol field from Bar struct documentation for clarity ([a7eb2da](https://github.com/mag1cfrog/stock_trading_bot/commit/a7eb2da8d44bff417bf20066bfb105b6f1d54438))

- Rename TimeSeries struct to BarSeries and update fetch_bars method to return Vec<BarSeries> ([df09f3a](https://github.com/mag1cfrog/stock_trading_bot/commit/df09f3af5dde3b40a003e3d6433ca33260590ccd))

- Restore and implement validation logic in TimeFrame struct ([bec0cef](https://github.com/mag1cfrog/stock_trading_bot/commit/bec0cefe18177b9bed60b5f6d43cfc84a1fefa8b))

- Replace errors module with legacy_errors and update references throughout the codebase ([97dd0eb](https://github.com/mag1cfrog/stock_trading_bot/commit/97dd0ebbcc92797d87bf0255440a4c89f8c59a4f))

- Add conditional compilation for alpaca-python-sdk feature in multiple modules ([77ca881](https://github.com/mag1cfrog/stock_trading_bot/commit/77ca88101a1d399efe582887dc2b952e890230c3))

- Consolidate ProviderError into a single module and remove redundant file ([e48d744](https://github.com/mag1cfrog/stock_trading_bot/commit/e48d744dc2036eab83cc42bb02e68c7dde32ac68))

- Enhance documentation for DataProvider trait and fetch_bars method ([7e1e5e2](https://github.com/mag1cfrog/stock_trading_bot/commit/7e1e5e2ef984dc65f3dd764c52626d95e610e984))

- Update dependencies in Cargo.toml for alpaca-python-sdk and make optional adjustments ([4ace99c](https://github.com/mag1cfrog/stock_trading_bot/commit/4ace99cbbf05dff1d61156c8d2d3ae82f4115fd7))

- Update error type in fetch_bars method to use unified Error type ([fd796aa](https://github.com/mag1cfrog/stock_trading_bot/commit/fd796aae4ad35c7e09e9b752d4b5a72504200165))

- Update Sink error handling to use SinkError type ([7ac4d4c](https://github.com/mag1cfrog/stock_trading_bot/commit/7ac4d4c1838827491196d048f5d94a0888856539))

- Remove unused error variants from Error enum ([50e1b05](https://github.com/mag1cfrog/stock_trading_bot/commit/50e1b05c5720c72f94108f8ee9a3d4489d72559f))

- Update dependencies in Cargo.toml for alpaca-python-sdk and make optional adjustments ([3cd6e59](https://github.com/mag1cfrog/stock_trading_bot/commit/3cd6e592a007ceea86ea654c4f2a1a3d67968fcc))

- Add conditional compilation for commands and params modules based on alpaca-python-sdk feature ([2af0a1c](https://github.com/mag1cfrog/stock_trading_bot/commit/2af0a1c563f46cde23007c98ee66d4bd2ba1aae8))

- Add conditional compilation for commands and params modules based on alpaca-python-sdk feature ([919b815](https://github.com/mag1cfrog/stock_trading_bot/commit/919b815c13f772f5502d3a6616b4f4f06d3b19b9))

- Update thiserror dependency to use workspace configuration across modules ([a1bf075](https://github.com/mag1cfrog/stock_trading_bot/commit/a1bf075a7b1ee0c355bb54f5cd0da4c4ba203fce))

- Replace unwrap with ? for error handling in AlpacaProvider headers ([243a987](https://github.com/mag1cfrog/stock_trading_bot/commit/243a9876e86588cf85d6a0e0261e2d5f0d1281b2))

- Simplify error handling in AlpacaProvider's fetch_bars method by removing custom error message for JSON deserialization ([71497d0](https://github.com/mag1cfrog/stock_trading_bot/commit/71497d0bc5c65f1c919d9449b115a93f8d677108))

- Rename error variant from Request to Reqwest for clarity in ProviderError enum ([ed614a0](https://github.com/mag1cfrog/stock_trading_bot/commit/ed614a001da2da4c5f5eea5fe7f2469279a5fc84))

- Remove serde_json from alpaca-python-sdk dependencies in Cargo.toml ([56c9cf3](https://github.com/mag1cfrog/stock_trading_bot/commit/56c9cf30714ffae3c82052bcd991c107bf9798ad))

- Break singluar alpaca_rest provider code file into smaller modules ([5a5ebdf](https://github.com/mag1cfrog/stock_trading_bot/commit/5a5ebdfb2dd29ddbabc046f8d1205497a4c28caf))

- Extract parameter construction logic into a separate function for clarity and reusability ([4c81c6c](https://github.com/mag1cfrog/stock_trading_bot/commit/4c81c6cbcbbfedd7b2cd687488976ee16de2c7d9))

- Simplify response conversion to BarSeries in AlpacaProvider ([3e448c9](https://github.com/mag1cfrog/stock_trading_bot/commit/3e448c9d3c3ec31337617ddd0d001195a5b36c88))

- Enhance fetch_bars method to handle pagination and accumulate bars ([90c4627](https://github.com/mag1cfrog/stock_trading_bot/commit/90c462789c19bb5f62487efb1de2483548da2318))

- Clean up code formatting and improve readability across multiple files ([d299889](https://github.com/mag1cfrog/stock_trading_bot/commit/d29988924033689c6ec74aa7c4ee799d9a736259))

- Update dependencies and enhance AlpacaProvider tests for improved functionality ([4ad06d6](https://github.com/mag1cfrog/stock_trading_bot/commit/4ad06d65cc347d7e9e9d193061d359d2e26c14ca))

- Adjust feature flags for Python SDK initialization in utils ([53975c4](https://github.com/mag1cfrog/stock_trading_bot/commit/53975c49fd6766617bf7301a26f44932b5a5438b))

- Remove conditional compilation for Alpaca Python SDK in request modules ([fb0e5b8](https://github.com/mag1cfrog/stock_trading_bot/commit/fb0e5b883e626187b2b2e0555362734fb03ee4b0))

- Remove validation logic from TimeFrame struct and associated tests ([bfa5c73](https://github.com/mag1cfrog/stock_trading_bot/commit/bfa5c73aeea21f35f76ae4639cae964e26167205))


### Documentation

- Update BarsRequestParams timeframe documentation for clarity and detail ([a0ad52d](https://github.com/mag1cfrog/stock_trading_bot/commit/a0ad52d5fb8b42291cb7a7fd749afcd249e92642))

- Update module documentation for DataProvider trait with example usage ([867bd68](https://github.com/mag1cfrog/stock_trading_bot/commit/867bd6894b2c128a46a045d251d8965ddf02920d))

- Enhance documentation for Bar struct with detailed field descriptions ([7c79f8d](https://github.com/mag1cfrog/stock_trading_bot/commit/7c79f8d59b4a5e641df21d968c6c1a0546acf2fe))

- Add comment for ProviderInitError enum to clarify its purpose ([8b46179](https://github.com/mag1cfrog/stock_trading_bot/commit/8b46179dbd1e59b5ec59bb9195fcdecd50466457))


### Features

- Add asset and request_params modules to market_data_ingestor ([d2406a3](https://github.com/mag1cfrog/stock_trading_bot/commit/d2406a3ff3321ddde7e9dd3b8dca7f90df197c2f))

- Define AssetClass enum with UsEquity and Futures variants ([36a5285](https://github.com/mag1cfrog/stock_trading_bot/commit/36a5285c49d6ef4dc2b80817f896040909caea99))

- Implement BarsRequestParams struct for market data requests ([e9cb348](https://github.com/mag1cfrog/stock_trading_bot/commit/e9cb3488e7289c01f0b86491428bc5ea6e721bfa))

- Enhance BarsRequestParams struct with detailed documentation for clarity ([2c969d3](https://github.com/mag1cfrog/stock_trading_bot/commit/2c969d37fe74329adabd0429433d5701c02063d5))

- Add documentation for AssetClass enum to clarify asset types ([68e6a06](https://github.com/mag1cfrog/stock_trading_bot/commit/68e6a06de25a63547172362f4a97c0e84dea1ea5))

- Add Bar struct for market data representation ([87a2d3f](https://github.com/mag1cfrog/stock_trading_bot/commit/87a2d3f987acb94670c2c3b3becf208924f8a2dc))

- Implement DataProvider trait with async_trait ([5120deb](https://github.com/mag1cfrog/stock_trading_bot/commit/5120debc2ecd8034b3501ae0ed22a0596d31cac1))

- Add bar_series module to manage time-series data for symbols ([e22c8d7](https://github.com/mag1cfrog/stock_trading_bot/commit/e22c8d746d7df16ef2e3e861c8b73d128c954c75))

- Introduce unified error type and update references in DataProvider trait ([869a00c](https://github.com/mag1cfrog/stock_trading_bot/commit/869a00ca46abeeff24c7c55fb70ea9f675e3021b))

- Add ProviderError type for handling errors in DataProvider implementations and update Cargo.toml dependencies ([fa5d903](https://github.com/mag1cfrog/stock_trading_bot/commit/fa5d90393b72a5a159dfbd1f154067a8c79e5867))

- Add DataSink trait and SinkError enum for handling data writing operations ([1e20e91](https://github.com/mag1cfrog/stock_trading_bot/commit/1e20e9173279a04a11e28c9c5db10e3e5b57aafd))

- Add secrecy dependency and implement AlpacaProvider struct for secure API key handling ([5a07436](https://github.com/mag1cfrog/stock_trading_bot/commit/5a0743608d39d0c5ac2526e3e2e533b186bed9f3))

- Integrate shared_utils for environment variable handling in AlpacaProvider ([b9a7aec](https://github.com/mag1cfrog/stock_trading_bot/commit/b9a7aec4a790b08d0d414a0cd56337fbf4f3ac08))

- Add trade_count and vwap fields to Bar struct for enhanced market data representation ([42e87e1](https://github.com/mag1cfrog/stock_trading_bot/commit/42e87e1ce6051870d8ccd97673b53b0e49803380))

- Enhance AlpacaProvider to include trade_count and vwap in bar data retrieval ([3b7018c](https://github.com/mag1cfrog/stock_trading_bot/commit/3b7018c7f3973dcc13b30052cbaf1df7bd559d49))

- Add provider-specific parameters to BarsRequestParams and enhance Alpaca provider with new request options ([f113bc6](https://github.com/mag1cfrog/stock_trading_bot/commit/f113bc638a7add76ae7b0567ce00e9bde0087d53))

- Update AlpacaProvider to support additional query parameters for bar data retrieval ([949f431](https://github.com/mag1cfrog/stock_trading_bot/commit/949f43118bc9e6cefb1c9675c105f7af5cfe9953))

- Add timeframe validation for Alpaca bars request parameters ([164b13f](https://github.com/mag1cfrog/stock_trading_bot/commit/164b13f3d05a15520c95b72c26ad01a87dc540e8))

- Add build script for Python virtual environment setup and dependency installation ([53523bc](https://github.com/mag1cfrog/stock_trading_bot/commit/53523bc4fb3f9c778b3f4556443a072098b6b6cf))

- Add dotenvy dependency and load environment variables in tests ([b0e5ea4](https://github.com/mag1cfrog/stock_trading_bot/commit/b0e5ea4c0ca7343f8f75edb564987c6d3ad317c4))

- Add extra environment variables for rust-analyzer configuration ([28ac146](https://github.com/mag1cfrog/stock_trading_bot/commit/28ac1463bba8e9e7e8742018a4b08ebdc01866b1))

- Add Alpaca subscription plans and implement rate limiting in AlpacaProvider ([420d025](https://github.com/mag1cfrog/stock_trading_bot/commit/420d02559ea15fe145d362808e693ea1cd3bf33d))

- Add subscription plan validation and enhance request validation in AlpacaProvider ([2ac8953](https://github.com/mag1cfrog/stock_trading_bot/commit/2ac8953a9a778f6a33533f755e6298b65414d044))


### Maintenance

- Add .gitignore to exclude target directory ([62a3863](https://github.com/mag1cfrog/stock_trading_bot/commit/62a38631944d5da366be54aae3b8ec8e323c2e1a))

- Add .vscode to .gitignore to exclude IDE configuration files ([4b8db00](https://github.com/mag1cfrog/stock_trading_bot/commit/4b8db007e2845d545c91549941e86fc527ea382d))

- Release ([6d857d6](https://github.com/mag1cfrog/stock_trading_bot/commit/6d857d6974b42d27419ff492d4d9fd6cda5451d6))


### Testing

- Add unit tests for timeframe validation and parameter construction in AlpacaBarsParams ([a4e3bc3](https://github.com/mag1cfrog/stock_trading_bot/commit/a4e3bc3c07d7701e4d82b2f4b3ac406f3a72b3ce))

- Add unit test for fetching bars in AlpacaProvider with sorting validation ([830c4b3](https://github.com/mag1cfrog/stock_trading_bot/commit/830c4b3c2e582a4679471c29378f1cb71e047ad7))

- Add pagination test for AlpacaProvider to validate bar fetching limits ([04ff500](https://github.com/mag1cfrog/stock_trading_bot/commit/04ff500b5d23e857da1055d12bdcc3b775406697))

- Add unit tests for date range validation and request validation in Alpaca subscription plans ([5750c96](https://github.com/mag1cfrog/stock_trading_bot/commit/5750c96f72c8cf404b52a597239e68e2dc5d82cf))

- Add validation for Alpaca subscription plan in fetch_bars test ([458145c](https://github.com/mag1cfrog/stock_trading_bot/commit/458145cab7854afd6bf8bae58409d2b388fc6aca))

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

