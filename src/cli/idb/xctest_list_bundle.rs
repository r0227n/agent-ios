use crate::companion::CompanionResolver;

pub async fn run(
    bundle_id: String,
    app_path: Option<String>,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Connect to companion
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    // 2. List tests in the bundle
    let tests = client.xctest_list_tests(bundle_id, app_path).await?;

    // 3. Output test names (one per line, matching Python idb behavior)
    for test in tests {
        println!("{}", test);
    }

    Ok(())
}
