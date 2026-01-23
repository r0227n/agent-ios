//! Console command for streaming device logs

use agent_mobile_core::OutputWriter;
use agent_mobile_gateway::{stream_console_logs, DeviceResolver};
use clap::Args;

use crate::helpers::{setup_ctrl_c_handler, CommandResult, DeviceArgs};

/// Console output streaming arguments
#[derive(Args)]
pub struct ConsoleArgs {
    /// Output file path. Streams to stdout if not specified.
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Run the console command
pub async fn run(args: ConsoleArgs) -> CommandResult {
    // Use gateway for platform detection
    let platform = DeviceResolver::resolve_platform(args.device.platform.as_deref()).await?;

    // Create output writer
    let writer = match args.output.as_deref() {
        Some("-") => {
            return Err("Invalid output path: '-'. Use without -o option for stdout output.".into())
        }
        Some(path) => OutputWriter::tee_from_path(path)?,
        None => OutputWriter::tee_from_path("-")?,
    };

    // Setup Ctrl+C handler
    let stop_rx = setup_ctrl_c_handler();

    // Stream logs via gateway
    stream_console_logs(platform, args.device.udid.as_deref(), writer, stop_rx).await?;

    Ok(())
}
