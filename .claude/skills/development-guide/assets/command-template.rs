//! {COMMAND_NAME} コマンド - {DESCRIPTION}
//!
//! ```bash
//! agent-mobile {COMMAND_NAME} [OPTIONS]
//! ```

use clap::Args;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::client::{with_xcuitest, CommandResult};
use crate::helpers::common_args::DeviceArgs;

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
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    match platform {
        Platform::Ios => run_ios().await,
        Platform::Android => run_android().await,
    }
}

/// iOS implementation
async fn run_ios() -> CommandResult {
    with_xcuitest(|client| async move {
        // TODO: Implement iOS logic using XCUITest Runner (HTTP)
        // Example:
        // client.focus().await?;

        println!("iOS {COMMAND_NAME} executed");
        Ok(())
    })
    .await
}

/// Android implementation
async fn run_android() -> CommandResult {
    // TODO: Implement Android logic using ADB native protocol
    // Example:
    // use agent_mobile_platform_android::adb;
    // adb::commands::shell_command(serial, "input tap 100 200").await?;

    println!("Android {COMMAND_NAME} executed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_{COMMAND_NAME}_args() {
        assert_eq!(
            std::mem::size_of::<{CommandName}Args>(),
            std::mem::size_of::<{CommandName}Args>()
        );

        // TODO: Add unit tests for argument parsing
        // Example:
        // let args = {CommandName}Args {
        //     device: DeviceArgs { udid: None },
        // };
        // assert!(args.device.udid.is_none());
    }
}
