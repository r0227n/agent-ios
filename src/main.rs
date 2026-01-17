mod cli;
mod companion;
mod grpc;
mod types;

use clap::Parser;
use cli::idb::IdbCommands;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Idb { command } => match command {
            IdbCommands::ListTargets { only, human } => {
                cli::idb::list_targets::run(only, human).await?;
            }
            IdbCommands::Launch {
                bundle_id,
                app_arguments,
                udid,
                wait_for_debugger,
                foreground_if_running,
                wait_for,
                pid_file,
            } => {
                cli::idb::launch::run(
                    bundle_id,
                    app_arguments,
                    udid,
                    wait_for_debugger,
                    foreground_if_running,
                    wait_for,
                    pid_file,
                )
                .await?;
            }
            IdbCommands::Kill => {
                cli::idb::kill::run().await?;
            }
            IdbCommands::Screenshot { dest_path, udid } => {
                cli::idb::screenshot::run(dest_path, udid).await?;
            }
            IdbCommands::Focus { udid } => {
                cli::idb::focus::run(udid).await?;
            }
            IdbCommands::Log {
                udid,
                source,
                log_arguments,
            } => {
                cli::idb::log::run(udid, source, log_arguments).await?;
            }
            IdbCommands::Install {
                bundle_path,
                udid,
                make_debuggable,
                override_mtime,
                compression,
                json,
            } => {
                cli::idb::install::run(
                    bundle_path,
                    udid,
                    make_debuggable,
                    override_mtime,
                    compression,
                    json,
                )
                .await?;
            }
            IdbCommands::Uninstall { bundle_id, udid } => {
                cli::idb::uninstall::run(bundle_id, udid).await?;
            }
            IdbCommands::Rm {
                paths,
                udid,
                bundle_id,
            } => {
                cli::idb::rm::run(paths, udid, bundle_id).await?;
            }
            IdbCommands::XctestList { udid } => {
                cli::idb::xctest_list::run(udid).await?;
            }
        },
    }

    Ok(())
}
