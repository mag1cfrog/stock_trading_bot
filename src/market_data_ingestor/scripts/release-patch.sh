#!/bin/bash
set -e

echo "🚀 Starting patch release for market_data_ingestor..."

# Run cargo release (already in the correct directory)
cargo release patch --no-verify --execute --no-publish

echo "📝 Generating changelog..."
git cliff -c cliff.toml --latest -p CHANGELOG.md

echo "📦 Committing changelog..."
git add CHANGELOG.md
git commit -m "docs: update changelog for patch release"

echo "🔄 Pushing changes..."
git push

echo "✅ Patch release completed successfully!"