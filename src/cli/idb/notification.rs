use crate::companion::CompanionResolver;

pub async fn run(
    bundle_id: String,
    json_payload: String,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    client.send_notification(&bundle_id, &json_payload).await?;

    Ok(())
}
