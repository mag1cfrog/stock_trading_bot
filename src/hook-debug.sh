#!/bin/bash
# Basic debug script

# Create files in multiple locations to ensure we find the output
echo "Hook executed at $(date)" > /tmp/hook-test.log
echo "PWD: $(pwd)" >> /tmp/hook-test.log
echo "USER: $(whoami)" >> /tmp/hook-test.log
echo "ENV: $(env)" >> /tmp/hook-test.log

# Try a simple cliff command with direct output
git cliff -w "/home/hanbo/repo/stock_trading_bot/" \
  -c "src/market_data_ingestor/cliff.toml" \
  --include-path "src/market_data_ingestor/**" \
  -o "/home/hanbo/repo/stock_trading_bot/src/market_data_ingestor/CHANGELOG.md" \
  --latest || echo "Cliff failed" >> /tmp/hook-test.log