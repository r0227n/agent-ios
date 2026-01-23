use crate::helpers::{with_client, CommandResult};

pub async fn run(latitude: f64, longitude: f64, udid: Option<String>) -> CommandResult {
    // Validate coordinate ranges
    if !(-90.0..=90.0).contains(&latitude) {
        return Err(format!(
            "Invalid latitude: {}. Must be between -90.0 and 90.0",
            latitude
        )
        .into());
    }

    if !(-180.0..=180.0).contains(&longitude) {
        return Err(format!(
            "Invalid longitude: {}. Must be between -180.0 and 180.0",
            longitude
        )
        .into());
    }

    with_client(udid.as_deref(), |mut client| async move {
        client.set_location(latitude, longitude).await?;
        Ok(())
    })
    .await
}
