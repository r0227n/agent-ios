//! CLI コマンド構造定義
//!
//! clap による CLI パーサーとコマンド enum を提供します。

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
    Tap(crate::core::tap::TapArgs),

    /// Check a checkbox/switch (idempotent: only taps if not already checked)
    Check(crate::core::check::CheckArgs),

    /// Uncheck a checkbox/switch (idempotent: only taps if not already unchecked)
    Uncheck(crate::core::check::CheckArgs),

    /// Select a value from a picker/spinner
    Select(crate::core::select::SelectArgs),

    /// Long press on an element by ref, text, coordinates, or position
    #[command(name = "long-press")]
    LongPress(crate::core::long_press::LongPressArgs),

    /// Fill a text field (clear + type)
    Fill(crate::core::fill::FillArgs),

    /// Type text into focused field (append)
    #[command(name = "type")]
    Type(crate::core::type_cmd::TypeArgs),

    /// Swipe gesture
    Swipe(crate::core::swipe::SwipeArgs),

    /// Scroll within element or screen
    Scroll(crate::core::scroll::ScrollArgs),

    /// Get element property
    Get(crate::core::get::GetArgs),

    /// Check element state (returns exit code)
    Is(crate::core::is_cmd::IsArgs),

    /// Wait for element to appear/disappear
    Wait(crate::core::wait::WaitArgs),

    /// Take screenshot
    Screenshot(crate::core::screenshot::ScreenshotArgs),

    /// Find elements by semantic locators and optionally perform actions
    Find(crate::core::find::FindArgs),

    /// Capture UI snapshot with element references for AI agents
    Snapshot(crate::snapshot::SnapshotArgs),

    /// Record screen to MP4 video file
    Record(crate::record::RecordArgs),

    /// Stream device console output (logs)
    Console(crate::console::ConsoleArgs),

    // ==================== Existing Commands ====================
    /// Application management (launch, terminate, install, list)
    App(crate::app::AppArgs),

    /// Device management (list, boot, shutdown)
    Device(crate::device::DeviceArgs),

    /// Session management (list, show, create, destroy)
    Session(crate::session::SessionArgs),

    /// IDB-compatible commands (full idb CLI compatibility)
    Idb {
        #[command(subcommand)]
        command: Box<crate::idb::IdbCommands>,
    },
}
