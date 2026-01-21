//! Snapshot command: capture UI accessibility tree with element refs.
//!
//! Provides a snapshot of the current UI state with reference IDs (`@e1`, `@e2`, etc.)
//! for efficient AI agent interaction.

mod collector;
mod extractor;
mod ref_generator;
mod tree_printer;
pub mod types;

use chrono::Utc;
use clap::Args;

use crate::cli::helpers::{with_client, CommandResult, DeviceArgs, OutputFormat};
use types::Snapshot;

/// Snapshot command arguments.
#[derive(Args, Debug)]
pub struct SnapshotArgs {
    /// Show only interactive elements (buttons, text fields, etc.)
    #[arg(short = 'i', long)]
    pub interactive: bool,

    /// Remove empty structural elements
    #[arg(short = 'c', long)]
    pub compact: bool,

    /// Limit tree depth (e.g., -d 3 shows only top 3 levels)
    #[arg(short = 'd', long)]
    pub depth: Option<u32>,

    /// Output to file instead of stdout
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Output format (text or json). Default: text.
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Disable scrolling (by default, scrolling is enabled to capture all elements)
    #[arg(long)]
    pub no_scroll: bool,

    /// Maximum number of scroll operations (default: 5)
    #[arg(long, default_value = "5")]
    pub max_scrolls: u32,

    /// Delay between scroll operations in milliseconds (default: 500)
    #[arg(long, default_value = "500")]
    pub scroll_delay: u64,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the snapshot command.
pub async fn run(args: SnapshotArgs) -> CommandResult {
    let interactive = args.interactive;
    let compact = args.compact;
    let depth = args.depth;
    let output_path = args.output.clone();
    let format = args.format;
    let no_scroll = args.no_scroll;
    let max_scrolls = args.max_scrolls;
    let scroll_delay = args.scroll_delay;

    with_client(args.device.udid.as_deref(), |mut client| async move {
        // Collect raw elements (with or without scrolling)
        let raw_elements = if no_scroll {
            // --no-scroll: Single accessibility info fetch (original behavior)
            let json_str = client.accessibility_info(None, true).await?;
            let json: serde_json::Value = serde_json::from_str(&json_str)?;
            extractor::extract_ios_elements(&json)
        } else {
            // Default: Use collector with scrolling to capture all elements
            let config = collector::SnapshotCollectorConfig {
                max_scrolls,
                delay_ms: scroll_delay,
                ..Default::default()
            };
            let snapshot_collector = collector::SnapshotCollector::new(config);
            snapshot_collector.collect_all(&mut client, None).await?
        };

        // Generate refs
        let elements = ref_generator::generate_refs(&raw_elements);

        // Create snapshot
        let snapshot = Snapshot {
            snapshot_id: generate_snapshot_id(),
            timestamp: Utc::now(),
            elements,
        };

        // Format output
        let output_str = match format {
            OutputFormat::Text => {
                let options = tree_printer::PrintOptions {
                    interactive_only: interactive,
                    compact,
                    max_depth: depth,
                };
                tree_printer::print_tree(&snapshot.elements, &options)
            }
            OutputFormat::Json => serde_json::to_string_pretty(&snapshot)?,
        };

        // Write output
        if let Some(path) = output_path {
            std::fs::write(&path, &output_str)?;
            println!("Snapshot saved to: {}", path);
        } else {
            print!("{}", output_str);
        }

        Ok(())
    })
    .await
}

/// Generate a unique snapshot ID.
fn generate_snapshot_id() -> String {
    let id = nanoid::nanoid!(8);
    format!("snap_{}", id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_snapshot_id() {
        let id1 = generate_snapshot_id();
        let id2 = generate_snapshot_id();

        assert!(id1.starts_with("snap_"));
        assert!(id2.starts_with("snap_"));
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 13); // "snap_" + 8 chars
    }
}
