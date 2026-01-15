mod cli;
mod companion;
mod grpc;
mod simctl;
mod types;

use clap::Parser;
use cli::idb::IdbCommands;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Idb { command } => match command {
            IdbCommands::ListTargets { only, human } => {
                cli::idb::list_targets::run(only, human).await?;
            }
        },
    }

    Ok(())
}
