pub mod app;
pub mod clipboard;
pub mod device;
pub mod element;
pub mod gesture;
pub mod helpers;
pub mod idb;
pub mod keyboard;
pub mod privacy;

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

    /// Touch gesture operations (tap, swipe, scroll, long-press)
    Gesture(gesture::GestureArgs),

    /// Keyboard input operations (text, keys, buttons)
    Keyboard(keyboard::KeyboardArgs),

    /// Element navigation and interaction
    Element(element::ElementArgs),

    /// Application management (launch, terminate, install, list)
    App(app::AppArgs),

    /// Device management (list, boot, shutdown)
    Device(device::DeviceArgs),

    /// Clipboard operations (copy, paste)
    Clipboard(clipboard::ClipboardArgs),

    /// Privacy/permission management (grant, revoke, reset)
    Privacy(privacy::PrivacyArgs),
}
