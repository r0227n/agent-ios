//! accessibility command - Get accessibility information from the device.
//!
//! Retrieves the accessibility tree for UI automation and testing.

use crate::helpers::client::{with_client, CommandResult};

pub async fn describe_all(nested: bool, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let json = client.accessibility_info(None, nested).await?;
        println!("{}", json);
        Ok(())
    })
    .await
}

pub async fn describe_point(x: f64, y: f64, nested: bool, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let json = client.accessibility_info(Some((x, y)), nested).await?;
        println!("{}", json);
        Ok(())
    })
    .await
}
