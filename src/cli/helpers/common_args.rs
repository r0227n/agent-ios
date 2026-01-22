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
/// Replaces the deprecated `DeviceOutputArgs`.
#[derive(Args, Debug, Clone)]
pub struct DeviceFormatArgs {
    /// Platform (ios or android). Auto-detected if not specified.
    #[arg(short = 'p', long)]
    pub platform: Option<String>,

    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,

    /// Output format (text or json).
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}
