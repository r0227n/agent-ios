//! Debug server commands
//!
//! This module provides commands for managing the LLDB debug server.

pub mod start;
pub mod status;
pub mod stop;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum DebugServerCommands {
    /// Start the debug server for an application
    Start {
        /// Bundle ID of the application to debug
        bundle_id: String,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Stop the debug server
    Stop {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Get the status of the debug server
    Status {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}
