//! instruments command implementation
//!
//! Run Instruments profiling on the device

use crate::helpers::{setup_ctrl_c_handler, with_client, CommandResult};
use std::collections::HashMap;

/// Run instruments profiling on the device
#[allow(clippy::too_many_arguments)]
pub async fn run(
    template: String,
    app_bundle_id: Option<String>,
    app_args: Vec<String>,
    app_env: Vec<(String, String)>,
    output: Option<String>,
    post_args: Vec<String>,
    operation_duration: Option<f64>,
    terminate_timeout: Option<f64>,
    launch_retry_timeout: Option<f64>,
    launch_error_timeout: Option<f64>,
    udid: Option<String>,
) -> CommandResult {
    // Collect IDB_ prefixed environment variables
    let mut env_map: HashMap<String, String> = std::env::vars()
        .filter(|(k, _)| k.starts_with("IDB_"))
        .collect();

    // Add user-provided environment variables
    for (key, value) in app_env {
        env_map.insert(key, value);
    }

    // Determine output path
    let output_path = output.unwrap_or_else(|| "trace".to_string());

    eprintln!("Running instruments with template '{}' until ^C", template);

    let stop_rx = setup_ctrl_c_handler();

    // Build timings if any timing option is provided
    let has_timings = operation_duration.is_some()
        || terminate_timeout.is_some()
        || launch_retry_timeout.is_some()
        || launch_error_timeout.is_some();

    let timings = if has_timings {
        Some(
            agent_mobile_platform_ios::grpc::client::InstrumentsTimings {
                operation_duration,
                terminate_timeout,
                launch_retry_timeout,
                launch_error_timeout,
            },
        )
    } else {
        None
    };

    with_client(udid.as_deref(), |mut client| async move {
        let trace_files = client
            .instruments_run(
                template,
                app_bundle_id,
                env_map,
                app_args,
                timings,
                post_args,
                output_path,
                stop_rx,
            )
            .await?;

        eprintln!("\nInstruments stopped");
        for file in trace_files {
            println!("{}", file);
        }
        Ok(())
    })
    .await
}
