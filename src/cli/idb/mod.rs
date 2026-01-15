pub mod launch;
pub mod list_targets;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum IdbCommands {
    /// Lists connected and available targets
    ListTargets {
        /// Which device types to list (device, simulator, mac)
        #[arg(long)]
        only: Option<String>,

        /// Output in human-readable format (like idb)
        #[arg(long)]
        human: bool,
    },

    /// Launch an application
    Launch {
        /// Application bundle identifier
        bundle_id: String,

        /// Arguments to pass to the application
        #[arg(trailing_var_arg = true)]
        app_arguments: Vec<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,

        /// Wait for debugger to attach
        #[arg(short = 'd', long)]
        wait_for_debugger: bool,

        /// Bring app to foreground if already running
        #[arg(short = 'f', long)]
        foreground_if_running: bool,

        /// Wait for process to exit, tailing output
        #[arg(short = 'w', long)]
        wait_for: bool,

        /// Write process PID to this file
        #[arg(short = 'p', long)]
        pid_file: Option<String>,
    },
}
