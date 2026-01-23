//! xctrace command implementation
//!
//! Record xctrace data on the device

use agent_mobile_platform_ios::grpc::XctraceTarget;

use crate::cli::helpers::{
    formatted_time_to_seconds, setup_ctrl_c_handler, with_client, CommandResult,
};
use std::collections::HashMap;

/// Record xctrace on the device
#[allow(clippy::too_many_arguments)]
pub async fn record(
    template: String,
    all_processes: bool,
    attach: Option<String>,
    launch: Option<String>,
    launch_args: Vec<String>,
    output: Option<String>,
    time_limit: Option<String>,
    package: Option<String>,
    target_stdin: Option<String>,
    target_stdout: Option<String>,
    env: Vec<(String, String)>,
    stop_timeout: Option<String>,
    post_args: Vec<String>,
    udid: Option<String>,
) -> CommandResult {
    // Validate mutually exclusive target options
    let target_count = [all_processes, attach.is_some(), launch.is_some()]
        .iter()
        .filter(|&&b| b)
        .count();

    if target_count == 0 {
        return Err("One of --all-processes, --attach, or --launch must be specified".into());
    }
    if target_count > 1 {
        return Err("--all-processes, --attach, and --launch are mutually exclusive".into());
    }

    // Validate stdin/stdout options
    if let Some(ref stdin) = target_stdin {
        if stdin != "-" {
            return Err("--target-stdin only supports '-' (pipe from stdin)".into());
        }
    }
    if let Some(ref stdout) = target_stdout {
        if stdout != "-" {
            return Err("--target-stdout only supports '-' (pipe to stdout)".into());
        }
    }

    // Parse time limit
    let time_limit_secs = time_limit
        .as_ref()
        .map(|t| formatted_time_to_seconds(t))
        .transpose()?;

    // Parse stop timeout
    let stop_timeout_secs = stop_timeout
        .as_ref()
        .map(|t| formatted_time_to_seconds(t))
        .transpose()?;

    // Build target
    let target = if all_processes {
        XctraceTarget::AllProcesses
    } else if let Some(process) = attach {
        XctraceTarget::Attach(process)
    } else if let Some(process) = launch {
        // Collect IDB_ prefixed environment variables
        let mut env_map: HashMap<String, String> = std::env::vars()
            .filter(|(k, _)| k.starts_with("IDB_"))
            .collect();

        // Add user-provided environment variables
        for (key, value) in env {
            env_map.insert(key, value);
        }

        XctraceTarget::Launch {
            process,
            args: launch_args,
            stdin: target_stdin,
            stdout: target_stdout,
            env: env_map,
        }
    } else {
        unreachable!()
    };

    // Determine output path
    let output_path = output.unwrap_or_else(|| "trace".to_string());

    eprintln!("Recording xctrace with template '{}' until ^C", template);

    let stop_rx = setup_ctrl_c_handler();

    with_client(udid.as_deref(), |mut client| async move {
        let trace_files = client
            .xctrace_record(
                template,
                time_limit_secs,
                package,
                target,
                stop_timeout_secs,
                post_args,
                output_path,
                stop_rx,
            )
            .await?;

        eprintln!("\nXCTrace recording stopped");
        for file in trace_files {
            println!("{}", file);
        }
        Ok(())
    })
    .await
}
