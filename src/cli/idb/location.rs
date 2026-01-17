use crate::companion::CompanionResolver;

pub async fn run(
    latitude: f64,
    longitude: f64,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    client.set_location(latitude, longitude).await?;

    Ok(())
}
