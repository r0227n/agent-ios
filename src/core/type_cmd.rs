//! type コマンド - テキスト入力 (追記モード)
//!
//! ```bash
//! agent-mobile type "Hello World"
//! ```

use clap::Args;

use agent_mobile_core::Platform;

use crate::helpers::client::{with_client, CommandResult};
use crate::helpers::common_args::DeviceArgs;

use agent_mobile_gateway::DeviceResolver;

/// type コマンド引数
#[derive(Args, Debug)]
pub struct TypeArgs {
    /// Text to type
    pub text: String,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the type command
pub async fn run(args: TypeArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    match platform {
        Platform::Ios => execute_type_ios(args.device.udid.as_deref(), &args.text).await,
        Platform::Android => execute_type_android(args.device.udid.as_deref(), &args.text).await,
    }
}

/// Execute type on iOS
async fn execute_type_ios(udid: Option<&str>, text: &str) -> CommandResult {
    use agent_mobile_platform_ios::hid::events;

    let text = text.to_string();

    with_client(udid, |mut client| async move {
        let text_events = events::text_to_events(&text)?;
        client.hid(text_events).await?;
        Ok(())
    })
    .await
}

/// Execute type on Android
async fn execute_type_android(udid: Option<&str>, text: &str) -> CommandResult {
    use agent_mobile_platform_android::adb::input;

    input::text(udid, text).await?;
    Ok(())
}
