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
    Seed {
        #[arg(long)]
        demo: bool,
    },
    Backup {
        path: PathBuf,
    },
    Restore {
        path: PathBuf,
        #[arg(long)]
        force: bool,
    },
    Check,
    /// Recovery: reset a user's password by username (admin console access
    /// required). Invalidates all sessions for that user.
    ResetPassword {
        username: String,
        password: String,
    },
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
        Command::Seed { demo } => {
            db::migrate(&pool).await?;
            seed::seed(&pool).await?;
            if demo {
                seed::seed_demo(&pool).await?;
                println!("seed applied (with demo data)");
            } else {
                println!("seed applied");
            }
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
        Command::ResetPassword { username, password } => {
            db::migrate(&pool).await?;
            let id = logholizon_core::auth::reset_password_by_username(&pool, &username, &password)
                .await?;
            println!("password reset for {username} ({id}); sessions invalidated");
        }
    }
    Ok(())
}
