pub mod agent;
pub mod helpers;
pub mod idb;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent-mobile")]
#[command(about = "Rust-based iOS Development Bridge")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// IDB-compatible commands (full idb CLI compatibility)
    Idb {
        #[command(subcommand)]
        command: Box<idb::IdbCommands>,
    },

    // AI-optimized commands (top-level)
    /// Take a screenshot of the device
    Screenshot {
        /// Output path for the screenshot
        path: String,
        /// Target device UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Tap at screen coordinates
    Tap {
        /// X coordinate
        x: f64,
        /// Y coordinate
        y: f64,
        /// Target device UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Swipe gesture from start to end coordinates
    Swipe {
        /// Start X coordinate
        x_start: f64,
        /// Start Y coordinate
        y_start: f64,
        /// End X coordinate
        x_end: f64,
        /// End Y coordinate
        y_end: f64,
        /// Duration of the swipe in seconds
        #[arg(short, long, default_value = "0.5")]
        duration: f64,
        /// Target device UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Type text on the device
    #[command(name = "type")]
    TypeText {
        /// Text to type
        text: String,
        /// Target device UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Launch an application
    Launch {
        /// Bundle ID of the app to launch
        bundle_id: String,
        /// Target device UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Install an application
    Install {
        /// Path to the .app or .ipa bundle
        bundle_path: String,
        /// Target device UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Get accessibility tree for the current screen
    Accessibility {
        /// Include nested elements
        #[arg(short, long)]
        nested: bool,
        /// Target device UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Suggest best simulators/emulators for testing (AI-optimized)
    SuggestSimulator {
        /// Target platform (ios or android, default: ios)
        #[arg(long, default_value = "ios")]
        platform: String,

        /// Number of suggestions (1-10)
        #[arg(long, default_value = "4", value_parser = clap::value_parser!(u8).range(1..=10))]
        count: u8,

        /// Output in human-readable format (default: JSON)
        #[arg(long)]
        human: bool,
    },
}
