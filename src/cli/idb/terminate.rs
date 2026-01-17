use crate::companion::CompanionResolver;

pub async fn run(
    bundle_id: String,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Connect to companion (with auto-spawning if needed)
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    // 2. Call terminate RPC
    client.terminate(&bundle_id).await?;

    // 3. Success (no output for terminate command, same as Python idb)
    Ok(())
}
