#!/bin/bash
set -e

echo "🚀 Starting major release for market_data_ingestor..."
echo "⚠️  WARNING: This is a MAJOR release with breaking changes!"
read -p "Are you sure you want to continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Major release cancelled."
    exit 1
fi

cargo release major --no-verify --execute --no-publish

echo "📝 Generating changelog..."
git cliff -c cliff.toml -p CHANGELOG.md

echo "📦 Committing changelog..."
git add CHANGELOG.md
git commit -m "docs: update changelog for major release"

echo "🔄 Pushing changes..."
git push

echo "✅ Major release completed successfully!"