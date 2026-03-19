//! Shared iOS text input helpers.

use std::time::Duration;

use agent_mobile_platform_ios::xcuitest::XCUITestClient;

use crate::helpers::client::CommandResult;

const IOS_TEXT_INPUT_FOCUS_DELAY: Duration = Duration::from_millis(100);

pub(crate) async fn clear_text_input_ios(client: &XCUITestClient, x: f64, y: f64) -> CommandResult {
    client.tap(x, y).await?;
    tokio::time::sleep(IOS_TEXT_INPUT_FOCUS_DELAY).await;
    client.clear_text().await?;
    Ok(())
}

pub(crate) async fn fill_text_input_ios(
    client: &XCUITestClient,
    x: f64,
    y: f64,
    text: &str,
) -> CommandResult {
    clear_text_input_ios(client, x, y).await?;
    client.type_text(text).await?;
    Ok(())
}
