//! Output format helpers for CLI commands.
//!
//! Provides a unified `OutputFormat` enum for consistent output format options across all commands.

use clap::ValueEnum;

/// Output format for CLI commands.
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable output (default).
    #[default]
    Human,
    /// JSON output for machine consumption.
    Json,
}

impl OutputFormat {
    /// Returns true if the output format is JSON.
    pub fn is_json(&self) -> bool {
        matches!(self, OutputFormat::Json)
    }
}
