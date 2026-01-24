use agent_mobile_platform_ios::proto::idb::hid_event::HidButtonType;

use crate::helpers::client::{with_client, CommandResult};
use agent_mobile_platform_ios::hid::events;

pub async fn run(button: String, duration: Option<f64>, udid: Option<String>) -> CommandResult {
    // Parse button type
    let button_type = match button.to_uppercase().as_str() {
        "APPLE_PAY" => HidButtonType::ApplePay,
        "HOME" => HidButtonType::Home,
        "LOCK" => HidButtonType::Lock,
        "SIDE_BUTTON" => HidButtonType::SideButton,
        "SIRI" => HidButtonType::Siri,
        _ => {
            return Err(format!(
                "Invalid button type: {}. Valid types: APPLE_PAY, HOME, LOCK, SIDE_BUTTON, SIRI",
                button
            )
            .into())
        }
    };

    with_client(udid.as_deref(), |mut client| async move {
        let events = events::button_to_events(button_type, duration);
        client.hid(events).await?;
        Ok(())
    })
    .await
}
