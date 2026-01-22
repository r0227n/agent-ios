//! Console command for streaming device logs
//!
//! This module provides the `console` command which streams real-time
//! console output from iOS and Android devices.

pub mod android;
pub mod ios;

use clap::{Args, Subcommand};

use crate::cli::helpers::CommandResult;

/// Console output streaming arguments
#[derive(Args)]
pub struct ConsoleArgs {
    #[command(subcommand)]
    pub platform: ConsolePlatform,
}

/// Target platform for console streaming
#[derive(Subcommand)]
pub enum ConsolePlatform {
    /// Stream iOS device console output
    Ios {
        /// Target device/simulator UDID (auto-selects if only one device)
        #[arg(short, long)]
        udid: Option<String>,
    },
    /// Stream Android device logcat
    Android {
        /// Target device serial (auto-selects if only one device)
        #[arg(short, long)]
        serial: Option<String>,
    },
}

/// Run the console command
pub async fn run(args: ConsoleArgs) -> CommandResult {
    match args.platform {
        ConsolePlatform::Ios { udid } => ios::run(udid).await,
        ConsolePlatform::Android { serial } => android::run(serial).await,
    }
}
