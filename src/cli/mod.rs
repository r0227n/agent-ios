pub mod app;
pub mod core;
pub mod device;
pub mod helpers;
pub mod idb;
pub mod record;
pub mod session;
pub mod snapshot;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent-mobile")]
#[command(about = "AI Agent 向け モバイルアプリ E2E テスト CLI")]
#[command(version)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    /// Session name (can also be set via AGENT_MOBILE_SESSION env var)
    #[arg(long, global = true, env = "AGENT_MOBILE_SESSION")]
    pub session: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // ==================== Core Commands (AI Agent 向け) ====================
    /// Tap an element by ref, text, coordinates, or key
    Tap(core::TapArgs),

    /// Long press on an element by ref, text, coordinates, or position
    #[command(name = "long-press")]
    LongPress(core::LongPressArgs),

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

    /// Record screen to MP4 video file
    Record(record::RecordArgs),

    // ==================== Existing Commands ====================
    /// Application management (launch, terminate, install, list)
    App(app::AppArgs),

    /// Device management (list, boot, shutdown)
    Device(device::DeviceArgs),

    /// Session management (list, show, create, destroy)
    Session(session::SessionArgs),

    /// IDB-compatible commands (full idb CLI compatibility)
    Idb {
        #[command(subcommand)]
        command: Box<idb::IdbCommands>,
    },
}
