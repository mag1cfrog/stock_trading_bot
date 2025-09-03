use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Asset Sync CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Catalog(CatalogCmd),
}

#[derive(Args)]
struct CatalogCmd {
    #[command(subcommand)]
    sub: CatalogSub,
}

#[derive(Subcommand)]
enum CatalogSub {
    Sync {
        #[arg(long, value_name = "FILE")]
        file: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        prune: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Catalog(CatalogCmd {
            sub:
                CatalogSub::Sync {
                    file,
                    dry_run,
                    prune,
                },
        }) => {
            // 1) Read TOML
            let s = std::fs::read_to_string(&file)?;
            let cat: asset_sync::catalog::config::Catalog = toml::from_str(&s)?;

            // 2) Open DB (your helpers) + ensure migrations ran somewhere earlier in your flow
            let db_url = std::env::var("DATABASE_URL")?;
            let mut conn = asset_sync::db::connection::connect_sqlite(&db_url)?;

            // 3) Sync
            let opt = asset_sync::catalog::sync::SyncOptions { dry_run, prune };
            asset_sync::catalog::sync::sync_catalog(&mut conn, cat, opt)?;
        }
    }

    Ok(())
}
