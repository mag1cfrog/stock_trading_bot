#!/bin/bash
# /home/hanbo/repo/stock_trading_bot/src/hook-debug.sh
set -x  # Print commands for debugging

# Log everything to a file
exec &> /tmp/cliff-hook.log

git cliff -w "/home/hanbo/repo/stock_trading_bot/" -c "src/market_data_ingestor/cliff.toml" \
  --include-path "src/market_data_ingestor/**" \
  -o "src/market_data_ingestor/CHANGELOG.md" \
  --latest