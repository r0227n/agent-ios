use crate::cli::helpers::{with_client, CommandResult};

pub async fn run(latitude: f64, longitude: f64, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.set_location(latitude, longitude).await?;
        Ok(())
    })
    .await
}
