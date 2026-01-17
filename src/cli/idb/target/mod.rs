//! Target management commands
//!
//! This module provides simulator/device lifecycle management commands.
//! Most commands use xcrun simctl directly, while connect/disconnect/describe use gRPC.

pub mod boot;
pub mod clone;
pub mod connect;
pub mod create;
pub mod delete;
pub mod describe;
pub mod disconnect;
pub mod erase;
pub mod shutdown;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum TargetCommands {
    /// Boot a simulator
    Boot {
        /// Simulator UDID to boot
        udid: String,

        /// Boot in headless mode (no simulator window)
        #[arg(long)]
        headless: bool,
    },

    /// Shutdown a simulator
    Shutdown {
        /// Simulator UDID to shutdown
        udid: String,
    },

    /// Erase a simulator (reset to clean state)
    Erase {
        /// Simulator UDID to erase
        udid: String,
    },

    /// Create a new simulator
    Create {
        /// Device type (e.g., "iPhone 15 Pro")
        device_type: String,

        /// OS version runtime (e.g., "iOS-17-0")
        os_version: String,
    },

    /// Clone a simulator
    Clone {
        /// Source simulator UDID to clone
        udid: String,
    },

    /// Delete a simulator
    Delete {
        /// Simulator UDID to delete
        udid: String,
    },

    /// Delete all simulators
    DeleteAll,

    /// Connect to a companion
    Connect {
        /// Companion host address
        #[arg(long)]
        host: Option<String>,

        /// Companion gRPC port
        #[arg(long)]
        port: Option<u16>,

        /// Target UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Disconnect from a companion
    Disconnect {
        /// Target UDID to disconnect
        udid: String,
    },

    /// Describe a target
    Describe {
        /// Target UDID
        #[arg(short, long)]
        udid: Option<String>,

        /// Fetch diagnostics information
        #[arg(long)]
        diagnostics: bool,
    },
}
