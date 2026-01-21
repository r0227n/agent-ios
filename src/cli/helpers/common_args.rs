//! Common command-line arguments shared across multiple commands.
//!
//! This module provides reusable argument structures that can be embedded
//! in various CLI commands using `#[command(flatten)]`.

use clap::Args;

use super::OutputFormat;

/// Basic device selection arguments (platform + udid).
///
/// Used by commands that only need to identify the target device
/// without any output format options.
#[derive(Args, Debug, Clone)]
pub struct DeviceArgs {
    /// Platform (ios or android). Auto-detected if not specified.
    #[arg(short = 'p', long)]
    pub platform: Option<String>,

    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,
}

/// Device selection arguments with output format.
///
/// Used by commands that need both device selection and output formatting.
#[derive(Args, Debug, Clone)]
pub struct DeviceOutputArgs {
    /// Platform (ios or android). Auto-detected if not specified.
    #[arg(short = 'p', long)]
    pub platform: Option<String>,

    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,

    /// Output format (human or json).
    #[arg(short = 'o', long, value_enum, default_value = "human")]
    pub output: OutputFormat,
}
