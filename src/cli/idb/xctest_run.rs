use agent_mobile_platform_ios::proto::idb::xctest_run_response::test_run_info::Status;

use crate::cli::helpers::{with_client, CommandResult};

pub async fn run(
    test_bundle_id: String,
    tests_to_run: Vec<String>,
    udid: Option<String>,
) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let mut stream = client
            .xctest_run_logic(test_bundle_id, tests_to_run)
            .await?;

        while let Some(response) = stream.message().await? {
            // Print test results
            for test_result in &response.results {
                let status_str = match Status::try_from(test_result.status) {
                    Ok(Status::Passed) => "PASSED",
                    Ok(Status::Failed) => "FAILED",
                    Ok(Status::Crashed) => "CRASHED",
                    _ => "UNKNOWN",
                };

                println!(
                    "{} | {} | {} | {:.2}s",
                    status_str,
                    test_result.class_name,
                    test_result.method_name,
                    test_result.duration
                );

                if let Some(failure_info) = &test_result.failure_info {
                    eprintln!("  Failure: {}", failure_info.failure_message);
                }
            }

            // Print log output
            for log_line in &response.log_output {
                println!("{}", log_line);
            }
        }

        Ok(())
    })
    .await
}
