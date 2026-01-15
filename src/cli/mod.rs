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
    /// IDB-compatible commands
    Idb {
        #[command(subcommand)]
        command: idb::IdbCommands,
    },
}
