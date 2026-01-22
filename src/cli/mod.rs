pub mod app;
pub mod core;
pub mod device;
pub mod helpers;
pub mod idb;
pub mod snapshot;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent-mobile")]
#[command(about = "AI Agent 向け モバイルアプリ E2E テスト CLI")]
#[command(version)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // ==================== Core Commands (AI Agent 向け) ====================
    /// Tap an element by ref, text, coordinates, or key
    Tap(core::TapArgs),

    /// Fill a text field (clear + type)
    Fill(core::FillArgs),

    /// Type text into focused field (append)
    #[command(name = "type")]
    Type(core::TypeArgs),

    /// Swipe gesture
    Swipe(core::SwipeArgs),

    /// Scroll within element or screen
    Scroll(core::ScrollArgs),

    /// Get element property
    Get(core::GetArgs),

    /// Check element state (returns exit code)
    Is(core::IsArgs),

    /// Wait for element to appear/disappear
    Wait(core::WaitArgs),

    /// Take screenshot
    Screenshot(core::ScreenshotArgs),

    /// Capture UI snapshot with element references for AI agents
    Snapshot(snapshot::SnapshotArgs),

    // ==================== Existing Commands ====================
    /// Application management (launch, terminate, install, list)
    App(app::AppArgs),

    /// Device management (list, boot, shutdown)
    Device(device::DeviceArgs),

    /// IDB-compatible commands (full idb CLI compatibility)
    Idb {
        #[command(subcommand)]
        command: Box<idb::IdbCommands>,
    },
}
