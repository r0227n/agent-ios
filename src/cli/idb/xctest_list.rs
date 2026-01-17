use crate::companion::CompanionResolver;

/// List installed XCTest bundles
pub async fn run(udid: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    // Call xctest_list_bundles
    let bundles = client.xctest_list_bundles().await?;

    // Print results in human-readable format (matching Python idb format)
    // Format: bundle_id | name | architectures
    for bundle in bundles {
        let name = if bundle.name.is_empty() {
            "no bundle name available"
        } else {
            &bundle.name
        };

        let archs = if bundle.architectures.is_empty() {
            "no archs available".to_string()
        } else {
            bundle.architectures.join(", ")
        };

        println!("{} | {} | {}", bundle.bundle_id, name, archs);
    }

    Ok(())
}
