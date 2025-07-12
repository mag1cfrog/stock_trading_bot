# Release Scripts

This directory contains automated release scripts for the `market_data_ingestor` crate. These scripts handle version bumping, changelog generation, and Git operations for different types of releases.

## Prerequisites

Before using these scripts, ensure you have:

1. **Git** - For version control operations
2. **cargo-release** - Install with `cargo install cargo-release`
3. **git-cliff** - Install with `cargo install git-cliff`
4. **Proper Git setup** - Clean working directory with all changes committed

## Usage

**Important**: All scripts must be run from the `market_data_ingestor` directory (not from the `scripts` directory).

```bash
# Navigate to the correct directory
cd /path/to/market_data_ingestor

# Then run the desired script
./scripts/release-patch.sh
```

## Available Scripts

### Patch Release (`release-patch.sh`)
Use for backward-compatible bug fixes and minor improvements.

```bash
./scripts/release-patch.sh
```

**Example**: `1.2.3` → `1.2.4`

### Minor Release (`release-minor.sh`)
Use for new features that are backward-compatible.

```bash
./scripts/release-minor.sh
```

**Example**: `1.2.3` → `1.3.0`

### Major Release (`release-major.sh`)
Use for breaking changes that are not backward-compatible.

```bash
./scripts/release-major.sh
```

**Example**: `1.2.3` → `2.0.0`

⚠️ **Warning**: This script includes a confirmation prompt due to the breaking nature of major releases.

## What Each Script Does

1. **Version Bumping**: Uses `cargo release` to increment the version in `Cargo.toml`
2. **Changelog Generation**: Uses `git-cliff` to generate/update the `CHANGELOG.md` file
3. **Git Operations**: Commits the changelog and pushes changes to the remote repository

## Script Workflow

Each script follows this pattern:

1. Runs `cargo release [patch|minor|major]` with `--no-verify --execute --no-publish`
2. Generates changelog entries using `git cliff`
3. Commits the updated changelog
4. Pushes all changes to the remote repository

## Configuration

The scripts use:
- `cliff.toml` for changelog configuration
- `Cargo.toml` for release configuration
- Conventional commit format for changelog generation

## Troubleshooting

### Permission Denied
If you get permission errors, make the scripts executable:

```bash
chmod +x scripts/release-*.sh
```

### Git Issues
Ensure your working directory is clean:

```bash
git status
git add .
git commit -m "your changes"
```

### Missing Dependencies
Install required tools:

```bash
cargo install cargo-release git-cliff
```

## Notes

- Scripts use `--no-publish` flag, so they won't automatically publish to crates.io
- The changelog is generated based on conventional commits
- All changes are automatically pushed to the remote repository
- Make sure you're on the correct branch before running release scripts

## Example Workflow

```bash
# 1. Navigate to the correct directory
cd /home/hanbo/repo/stock_trading_bot/src/market_data_ingestor

# 2. Ensure working directory is clean
git status

# 3. Run the appropriate release script
./scripts/release-patch.sh

# 4. Verify the release
git log --oneline -5
```