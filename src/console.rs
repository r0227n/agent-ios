//! Console command for streaming device logs

use agent_mobile_core::OutputWriter;
use agent_mobile_gateway::stream_console_logs;
use clap::Args;

use crate::helpers::client::CommandResult;
use crate::helpers::common_args::DeviceArgs;
use crate::helpers::signal::setup_ctrl_c_handler;
use crate::helpers::target::resolve_target;

/// Console output streaming arguments
#[derive(Args)]
pub struct ConsoleArgs {
    /// Output file path. Streams to stdout if not specified.
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Device selection options.
    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Run the console command
pub async fn run(args: ConsoleArgs) -> CommandResult {
    let target = resolve_target(args.device.udid.as_deref()).await?;

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
    stream_console_logs(target.platform, Some(target.udid.as_str()), writer, stop_rx).await?;

    Ok(())
}
