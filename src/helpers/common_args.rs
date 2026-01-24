//! Common command-line arguments shared across multiple commands.
//!
//! This module provides reusable argument structures that can be embedded
//! in various CLI commands using `#[command(flatten)]`.

use clap::Args;

use super::format::OutputFormat;

/// Basic device selection arguments (udid only).
///
/// Used by commands that only need to identify the target device
/// without any output format options.
/// Platform is auto-detected from the UDID.
#[derive(Args, Debug, Clone)]
pub struct DeviceArgs {
    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,
}

/// Output format only (for commands that don't need file output).
#[derive(Args, Debug, Clone)]
pub struct FormatArgs {
    /// Output format (text or json).
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

/// Output format + file output (for snapshot etc.).
#[derive(Args, Debug, Clone)]
pub struct FormatOutputArgs {
    /// Output format (text or json).
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Output to file instead of stdout.
    #[arg(short = 'o', long)]
    pub output: Option<String>,
}

/// Device selection + output format.
///
/// Used by commands that need both device selection and output formatting.
/// Platform is auto-detected from the UDID.
#[derive(Args, Debug, Clone)]
pub struct DeviceFormatArgs {
    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,

    /// Output format (text or json).
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}
