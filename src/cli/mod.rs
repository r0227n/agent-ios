pub mod app;
pub mod device;
pub mod element;
pub mod helpers;
pub mod hid;
pub mod idb;
pub mod snapshot;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent-mobile")]
#[command(about = "Rust-based iOS Development Bridge")]
#[command(version)]
#[command(disable_help_subcommand = true)]
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

    /// HID operations (touch gestures + keyboard input)
    Hid(hid::HidArgs),

    /// Element navigation and interaction
    Element(element::ElementArgs),

    /// Application management (launch, terminate, install, list)
    App(app::AppArgs),

    /// Device management (list, boot, shutdown)
    Device(device::DeviceArgs),

    /// Capture UI snapshot with element references for AI agents
    Snapshot(snapshot::SnapshotArgs),
}
