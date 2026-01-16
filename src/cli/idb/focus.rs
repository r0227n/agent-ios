use crate::companion::CompanionResolver;

pub async fn run(udid: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    client.focus().await?;

    Ok(())
}
