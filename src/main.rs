use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod check;
mod checksum;
mod ls;
mod mint;
mod search;

#[derive(Parser)]
#[command(name = "nref", about = "Database-free information linking using global references")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all nanoref markers and where they appear
    Ls {
        #[arg(default_value = ".", value_name = "DIR")]
        path: PathBuf,
    },
    /// Generate and print a new nanoref marker
    Mint,
    /// Validate marker checksums and report errors
    Check {
        #[arg(default_value = ".", value_name = "DIR")]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Ls { path } => ls::run(&path),
        Commands::Mint => mint::run(),
        Commands::Check { path } => {
            let errors = check::run(&path)?;
            if errors > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
