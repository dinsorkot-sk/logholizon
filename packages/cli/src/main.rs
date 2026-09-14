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
    /// Install a ready-made module package (ERP solution) from a JSON file.
    /// The package runs through the same lifecycle (draft → review →
    /// published → enabled) as user-created modules.
    InstallModule {
        /// Path to the `<name>.module.json` package file.
        path: PathBuf,
        /// Owner the module is installed under (tenant scope).
        #[arg(long, default_value = "admin")]
        owner: String,
    },
    /// List installed modules for an owner (`--admin` lists every owner).
    ListModules {
        /// Owner scope to list.
        #[arg(long, default_value = "admin")]
        owner: String,
        /// List modules across all owners (admin view).
        #[arg(long)]
        admin: bool,
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
            match backup::restore(&path, destination).await? {
                Some(rollback) => println!(
                    "restored {} to {}; rollback preserved at {}",
                    path.display(),
                    destination.display(),
                    rollback.display()
                ),
                None => println!(
                    "restored {} to {} (fresh database, no rollback needed)",
                    path.display(),
                    destination.display()
                ),
            }
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
        Command::InstallModule { path, owner } => {
            db::migrate(&pool).await?;
            let raw = std::fs::read_to_string(&path)?;
            let package: serde_json::Value = serde_json::from_str(&raw)?;
            let module = logholizon_core::module_package::install_package(
                &pool,
                &package,
                owner.trim(),
                Some("cli"),
            )
            .await?;
            println!(
                "installed module {} ({}) version {} status {}",
                module.name, module.id, module.semantic_version, module.status
            );
        }
        Command::ListModules { owner, admin } => {
            db::migrate(&pool).await?;
            let role = if admin { "admin" } else { "user" };
            let modules =
                logholizon_core::repository::list_modules(&pool, owner.trim(), role).await?;
            if modules.is_empty() {
                println!("no modules installed for owner {owner}");
            }
            for module in modules {
                println!(
                    "{} ({}) version {} status {}",
                    module.name, module.id, module.semantic_version, module.status
                );
            }
        }
    }
    Ok(())
}
