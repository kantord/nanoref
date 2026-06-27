use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod check;
mod checksum;
mod ls;
mod mint;
mod search;

#[derive(Parser)]
#[command(
    name = "nref",
    about = "Database-free information linking using global references"
)]
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
        #[arg(long)]
        json: bool,
    },
    /// Generate and print a new nanoref marker
    Mint {
        #[arg(long)]
        json: bool,
    },
    /// Validate marker checksums and report errors
    Check {
        #[arg(default_value = ".", value_name = "DIR")]
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Ls { path, json } => ls::run(&path, json),
        Commands::Mint { json } => {
            mint::run(json);
            Ok(())
        }
        Commands::Check { path, json } => {
            let errors = check::run(&path, json)?;
            if errors > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
