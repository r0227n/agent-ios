//! {COMMAND_NAME} コマンド - {DESCRIPTION}
//!
//! ```bash
//! agent-mobile {COMMAND_NAME} [OPTIONS]
//! ```

use clap::Args;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::cli::helpers::{with_client, CommandResult, DeviceArgs};

/// {CommandName} コマンド引数
#[derive(Args, Debug)]
pub struct {CommandName}Args {
    // TODO: Add command-specific arguments here
    // Example:
    // /// Target: @eN ref, "text", or coordinates
    // pub target: String,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the {COMMAND_NAME} command
pub async fn run(args: {CommandName}Args) -> CommandResult {
    let platform = DeviceResolver::resolve_platform(args.device.platform.as_deref()).await?;

    match platform {
        Platform::Ios => run_ios(args.device.udid.as_deref()).await,
        Platform::Android => run_android(args.device.udid.as_deref()).await,
    }
}

/// iOS implementation
async fn run_ios(udid: Option<&str>) -> CommandResult {
    with_client(udid, |mut client| async move {
        // TODO: Implement iOS logic using idb gRPC
        // Example:
        // client.focus().await?;

        println!("iOS {COMMAND_NAME} executed");
        Ok(())
    })
    .await
}

/// Android implementation
async fn run_android(udid: Option<&str>) -> CommandResult {
    // TODO: Implement Android logic using adb
    // Example:
    // use agent_mobile_platform_android::adb::input;
    // input::tap(udid, 100.0, 200.0).await?;

    println!("Android {COMMAND_NAME} executed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_{COMMAND_NAME}_args() {
        // TODO: Add unit tests for argument parsing
        // Example:
        // let args = {CommandName}Args {
        //     device: DeviceArgs::default(),
        // };
        // assert!(args.device.udid.is_none());
    }
}
