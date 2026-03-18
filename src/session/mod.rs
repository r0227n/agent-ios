//! Session management commands
//!
//! This module provides commands for managing agent-mobile sessions.
//! Sessions allow binding a name to a device UDID for easier multi-device workflows.
//!
//! # Commands
//!
//! - `session list` - List all active sessions
//! - `session show` - Show current session information
//! - `session create <name> --udid <udid>` - Create a new session
//! - `session destroy <name>` - Destroy an existing session

pub mod app_context;
pub mod resolver;
pub mod state;

use chrono::Utc;
use clap::{Args, Subcommand};

use agent_mobile_core::Platform;

use crate::helpers::client::CommandResult;
use crate::helpers::format::OutputFormat;
use state::{SessionData, SessionState};

/// Session management arguments
#[derive(Args, Debug)]
pub struct SessionArgs {
    /// Session management action to execute.
    #[command(subcommand)]
    pub command: SessionCommands,
}

/// Session subcommands
#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// List all active sessions
    List {
        /// Output format (text or json)
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Show current session information (requires --session flag or AGENT_MOBILE_SESSION env)
    Show {
        /// Output format (text or json)
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Create a new session
    Create {
        /// Session name
        name: String,

        /// Device UDID to associate with this session
        #[arg(long)]
        udid: String,
    },

    /// Destroy an existing session
    Destroy {
        /// Session name to destroy
        name: String,
    },
}

/// Run session commands
pub async fn run(args: SessionArgs, current_session: Option<&str>) -> CommandResult {
    match args.command {
        SessionCommands::List { format } => list_sessions(format).await,
        SessionCommands::Show { format } => show_session(current_session, format).await,
        SessionCommands::Create { name, udid } => create_session(name, udid).await,
        SessionCommands::Destroy { name } => destroy_session(name).await,
    }
}

/// Generate JSON representation of a SessionData for output.
fn session_to_json(session: &SessionData) -> serde_json::Value {
    serde_json::json!({
        "name": session.name,
        "udid": session.udid,
        "platform": session.platform.as_str(),
        "app": session.app,
        "created_at": session.created_at.to_rfc3339(),
        "last_activity": session.last_activity.to_rfc3339(),
        "last_snapshot": session.last_snapshot.as_ref().map(|snap| {
            serde_json::json!({
                "snapshot_id": snap.snapshot_id,
                "timestamp": snap.timestamp.to_rfc3339(),
                "ref_count": snap.ref_count,
            })
        }),
    })
}

/// List all active sessions
async fn list_sessions(format: OutputFormat) -> CommandResult {
    let state = SessionState::new();
    let sessions = state.list_sessions()?;

    match format {
        OutputFormat::Json => {
            let output: Vec<_> = sessions.iter().map(session_to_json).collect();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Text => {
            if sessions.is_empty() {
                println!("No active sessions");
            } else {
                println!("{:<15} {:<40} {:<8} APP", "NAME", "UDID", "PLATFORM");
                println!("{}", "-".repeat(75));
                for session in sessions {
                    println!(
                        "{:<15} {:<40} {:<8} {}",
                        session.name,
                        session.udid,
                        session.platform,
                        session.app.as_deref().unwrap_or("-")
                    );
                }
            }
        }
    }

    Ok(())
}

/// Show current session information
async fn show_session(current_session: Option<&str>, format: OutputFormat) -> CommandResult {
    let session_name = current_session.ok_or(
        "No active session. Use --session flag or set AGENT_MOBILE_SESSION environment variable.",
    )?;

    let state = SessionState::new();
    let session = state
        .get_session(session_name)?
        .ok_or_else(|| format!("Session '{}' not found", session_name))?;

    match format {
        OutputFormat::Json => {
            let output = session_to_json(&session);
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Text => {
            println!("Session: {}", session.name);
            println!("  UDID: {}", session.udid);
            println!("  Platform: {}", session.platform);
            println!("  App: {}", session.app.as_deref().unwrap_or("(none)"));
            println!("  Created: {}", session.created_at.to_rfc3339());
            println!("  Last Activity: {}", session.last_activity.to_rfc3339());
            if let Some(ref snap) = session.last_snapshot {
                println!("  Last Snapshot:");
                println!("    ID: {}", snap.snapshot_id);
                println!("    Refs: {}", snap.ref_count);
                println!("    Time: {}", snap.timestamp.to_rfc3339());
            }
        }
    }

    Ok(())
}

/// Create a new session
async fn create_session(name: String, udid: String) -> CommandResult {
    let state = SessionState::new();

    // Check if session already exists
    if state.session_exists(&name) {
        return Err(format!("Session '{}' already exists", name).into());
    }

    // Auto-detect platform from UDID
    let platform: Platform = crate::helpers::target::detect_platform_from_udid(&udid).await?;

    let now = Utc::now();
    let data = SessionData {
        name: name.clone(),
        udid: udid.clone(),
        platform,
        app: None,
        created_at: now,
        last_activity: now,
        last_snapshot: None,
    };

    state.create_session(data)?;

    println!(
        "Session '{}' created (UDID: {}, platform: {})",
        name, udid, platform
    );
    Ok(())
}

/// Destroy an existing session
async fn destroy_session(name: String) -> CommandResult {
    let state = SessionState::new();

    if !state.session_exists(&name) {
        return Err(format!("Session '{}' does not exist", name).into());
    }

    state.destroy_session(&name)?;

    println!("Session '{}' destroyed", name);
    Ok(())
}
