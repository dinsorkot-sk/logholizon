use anyhow::Result;
use clap::{Parser, Subcommand};
use logholizon_core::{backup, db, seed, Config};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "logholizon")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Migrate,
    Seed,
    Backup {
        path: PathBuf,
    },
    Restore {
        path: PathBuf,
        #[arg(long)]
        force: bool,
    },
    Check,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::from_env();
    let pool = db::connect(&config.database_url).await?;
    match cli.command {
        Command::Migrate => {
            db::migrate(&pool).await?;
            println!("migrations applied");
        }
        Command::Seed => {
            db::migrate(&pool).await?;
            seed::seed(&pool).await?;
            println!("seed applied");
        }
        Command::Backup { path } => {
            db::migrate(&pool).await?;
            backup::backup(&pool, &path).await?;
            println!("backup created: {}", path.display());
        }
        Command::Restore { path, force } => {
            anyhow::ensure!(force, "restore requires --force");
            let destination = db::database_path(&config.database_url)?;
            backup::restore(&path, destination).await?;
            println!("restored {} to {}", path.display(), destination.display());
        }
        Command::Check => {
            db::migrate(&pool).await?;
            anyhow::ensure!(
                db::integrity_check(&pool).await?,
                "database integrity check failed"
            );
            println!("database ok");
        }
    }
    Ok(())
}
