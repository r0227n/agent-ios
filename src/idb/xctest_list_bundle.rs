use crate::helpers::client::{with_client, CommandResult};

pub async fn run(
    bundle_id: String,
    app_path: Option<String>,
    udid: Option<String>,
) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let tests = client.xctest_list_tests(bundle_id, app_path).await?;

        // Output test names (one per line, matching Python idb behavior)
        for test in tests {
            println!("{}", test);
        }

        Ok(())
    })
    .await
}
