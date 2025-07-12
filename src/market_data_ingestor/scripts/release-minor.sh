#!/bin/bash
set -e

echo "🚀 Starting minor release for market_data_ingestor..."

cargo release minor --no-verify --execute --no-publish

echo "📝 Generating changelog..."
git cliff -c cliff.toml -p CHANGELOG.md

echo "📦 Committing changelog..."
git add CHANGELOG.md
git commit -m "docs: update changelog for minor release"

echo "🔄 Pushing changes..."
git push

echo "✅ Minor release completed successfully!"