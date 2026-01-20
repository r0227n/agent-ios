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

    /// Suggest best simulators/emulators for testing (AI-optimized)
    SuggestSimulator {
        /// Target platform (ios or android, default: ios)
        #[arg(short = 'p', long, default_value = "ios")]
        platform: String,

        /// Number of suggestions (1-10)
        #[arg(long, default_value = "4", value_parser = clap::value_parser!(u8).range(1..=10))]
        count: u8,

        /// Output in human-readable format (default: JSON)
        #[arg(long)]
        human: bool,
    },
}
