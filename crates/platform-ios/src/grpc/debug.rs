//! Debug operations for IdbClient.

use std::collections::HashMap;
use std::io::Write;
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;

use crate::proto::idb::log_request::Source as LogSource;
use crate::proto::idb::LogRequest;

use super::client::{
    extract_trace_files, read_dap_message, IdbClient, InstrumentsTimings, XctraceTarget,
};

impl IdbClient {
    /// Tail logs from target or companion (server-side streaming)
    pub async fn log(
        &mut self,
        source: LogSource,
        arguments: Vec<String>,
    ) -> Result<
        tonic::Streaming<crate::proto::idb::LogResponse>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let request = tonic::Request::new(LogRequest {
            arguments,
            source: source.into(),
        });

        let response = self.client.log(request).await?;
        Ok(response.into_inner())
    }

    /// List crash logs
    pub async fn crash_list(
        &mut self,
        since: Option<u64>,
        before: Option<u64>,
        bundle_id: Option<String>,
        name: Option<String>,
    ) -> Result<Vec<crate::proto::idb::CrashLogInfo>, Box<dyn std::error::Error + Send + Sync>>
    {
        let query = crate::proto::idb::CrashLogQuery {
            since: since.unwrap_or(0),
            before: before.unwrap_or(0),
            bundle_id: bundle_id.unwrap_or_default(),
            name: name.unwrap_or_default(),
        };

        let request = tonic::Request::new(query);
        let response = self.client.crash_list(request).await?;
        Ok(response.into_inner().list)
    }

    /// Show crash log contents
    pub async fn crash_show(
        &mut self,
        name: &str,
    ) -> Result<crate::proto::idb::CrashShowResponse, Box<dyn std::error::Error + Send + Sync>>
    {
        let request = tonic::Request::new(crate::proto::idb::CrashShowRequest {
            name: name.to_string(),
        });
        let response = self.client.crash_show(request).await?;
        Ok(response.into_inner())
    }

    /// Delete crash logs
    pub async fn crash_delete(
        &mut self,
        since: Option<u64>,
        before: Option<u64>,
        bundle_id: Option<String>,
        name: Option<String>,
    ) -> Result<Vec<crate::proto::idb::CrashLogInfo>, Box<dyn std::error::Error + Send + Sync>>
    {
        let query = crate::proto::idb::CrashLogQuery {
            since: since.unwrap_or(0),
            before: before.unwrap_or(0),
            bundle_id: bundle_id.unwrap_or_default(),
            name: name.unwrap_or_default(),
        };

        let request = tonic::Request::new(query);
        let response = self.client.crash_delete(request).await?;
        Ok(response.into_inner().list)
    }

    /// Start the debug server for a bundle
    /// Returns LLDB bootstrap commands
    pub async fn debugserver_start(
        &mut self,
        bundle_id: String,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::proto::idb::debug_server_request::{Control, Start};
        use crate::proto::idb::DebugServerRequest;

        let (tx, rx) = mpsc::channel(1);
        let request_stream = ReceiverStream::new(rx);

        // Send start request
        tx.send(DebugServerRequest {
            control: Some(Control::Start(Start {
                bundle_id: bundle_id.clone(),
            })),
        })
        .await?;
        drop(tx); // End the stream

        // Start the bidirectional stream
        let response = self.client.debugserver(request_stream).await?;
        let mut stream = response.into_inner();

        // Get the response with bootstrap commands
        if let Some(resp) = stream.message().await? {
            if let Some(crate::proto::idb::debug_server_response::Control::Status(status)) =
                resp.control
            {
                return Ok(status.lldb_bootstrap_commands);
            }
        }

        Err("No response from debugserver start".into())
    }

    /// Stop the debug server
    pub async fn debugserver_stop(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::proto::idb::debug_server_request::{Control, Stop};
        use crate::proto::idb::DebugServerRequest;

        let (tx, rx) = mpsc::channel(1);
        let request_stream = ReceiverStream::new(rx);

        // Send stop request
        tx.send(DebugServerRequest {
            control: Some(Control::Stop(Stop {})),
        })
        .await?;
        drop(tx);

        // Start the stream and wait for completion
        let response = self.client.debugserver(request_stream).await?;
        let mut stream = response.into_inner();

        // Drain the stream
        while (stream.message().await?).is_some() {}

        Ok(())
    }

    /// Get the status of the debug server
    /// Returns Some(commands) if running, None if not running
    pub async fn debugserver_status(
        &mut self,
    ) -> Result<Option<Vec<String>>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::proto::idb::debug_server_request::{Control, Status};
        use crate::proto::idb::DebugServerRequest;

        let (tx, rx) = mpsc::channel(1);
        let request_stream = ReceiverStream::new(rx);

        // Send status request
        tx.send(DebugServerRequest {
            control: Some(Control::Status(Status {})),
        })
        .await?;
        drop(tx);

        // Start the stream
        let response = self.client.debugserver(request_stream).await?;
        let mut stream = response.into_inner();

        // Get the response
        if let Some(resp) = stream.message().await? {
            if let Some(crate::proto::idb::debug_server_response::Control::Status(status)) =
                resp.control
            {
                let commands = status.lldb_bootstrap_commands;
                if commands.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(commands));
            }
        }

        Ok(None)
    }

    /// Start a DAP debug session
    /// This bridges stdin/stdout to the remote DAP server
    pub async fn dap(
        &mut self,
        pkg_id: String,
        _port: Option<u16>,
        mut stop_rx: watch::Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::proto::idb::dap_request::{Control, Pipe, Start, Stop};
        use crate::proto::idb::DapRequest;
        use tokio::io::{AsyncWriteExt, BufReader};

        let (tx, rx) = mpsc::channel(32);
        let request_stream = ReceiverStream::new(rx);

        // Send start request
        tx.send(DapRequest {
            control: Some(Control::Start(Start {
                debugger_pkg_id: pkg_id,
            })),
        })
        .await?;

        // Start the bidirectional stream
        let response = self.client.dap(request_stream).await?;
        let mut stream = response.into_inner();

        // Wait for started response
        if let Some(resp) = stream.message().await? {
            if resp.output.is_none()
                || !matches!(
                    resp.output,
                    Some(crate::proto::idb::dap_response::Output::Started(_))
                )
            {
                return Err("DAP server failed to start".into());
            }
        } else {
            return Err("No response from DAP server".into());
        }

        eprintln!("DAP server started, bridging stdin/stdout...");

        // Setup stdin reader
        let stdin = tokio::io::stdin();
        let mut stdin_reader = BufReader::new(stdin);
        let mut stdout = tokio::io::stdout();

        // Bridge stdin/stdout to DAP server
        loop {
            tokio::select! {
                // Check for stop signal
                _ = stop_rx.changed() => {
                    if *stop_rx.borrow() {
                        // Send stop request
                        let _ = tx.send(DapRequest {
                            control: Some(Control::Stop(Stop {})),
                        }).await;
                        break;
                    }
                }

                // Read from stdin and send to DAP server
                result = read_dap_message(&mut stdin_reader) => {
                    match result {
                        Ok(Some(data)) => {
                            tx.send(DapRequest {
                                control: Some(Control::Pipe(Pipe { data })),
                            }).await?;
                        }
                        Ok(None) => {
                            // EOF on stdin
                            break;
                        }
                        Err(e) => {
                            eprintln!("Error reading stdin: {}", e);
                            break;
                        }
                    }
                }

                // Read from DAP server and write to stdout
                response = stream.message() => {
                    match response? {
                        Some(resp) => {
                            match resp.output {
                                Some(crate::proto::idb::dap_response::Output::Stdout(pipe)) => {
                                    stdout.write_all(&pipe.data).await?;
                                    stdout.flush().await?;
                                }
                                Some(crate::proto::idb::dap_response::Output::Stopped(_)) => {
                                    eprintln!("DAP server stopped");
                                    break;
                                }
                                _ => {}
                            }
                        }
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }

    /// Run instruments profiling (bidirectional streaming)
    ///
    /// Returns a list of trace file paths written to disk
    #[allow(clippy::too_many_arguments)]
    pub async fn instruments_run(
        &mut self,
        template_name: String,
        app_bundle_id: Option<String>,
        environment: HashMap<String, String>,
        arguments: Vec<String>,
        timings: Option<InstrumentsTimings>,
        post_process_arguments: Vec<String>,
        trace_basename: String,
        mut stop_rx: watch::Receiver<bool>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::proto::idb::instruments_run_request::{self, Control, Start, Stop};
        use crate::proto::idb::instruments_run_response::Output;
        use crate::proto::idb::InstrumentsRunRequest;

        let (tx, rx) = mpsc::channel::<InstrumentsRunRequest>(4);
        let request_stream = ReceiverStream::new(rx);

        // Start the bidirectional stream first
        let response = self.client.instruments_run(request_stream).await?;
        let mut stream = response.into_inner();

        // Build timings message if provided
        let timings_msg = timings.map(|t| instruments_run_request::InstrumentsTimings {
            operation_duration: t.operation_duration.unwrap_or(0.0),
            terminate_timeout: t.terminate_timeout.unwrap_or(0.0),
            launch_retry_timeout: t.launch_retry_timeout.unwrap_or(0.0),
            launch_error_timeout: t.launch_error_timeout.unwrap_or(0.0),
        });

        // Send start request
        tx.send(InstrumentsRunRequest {
            control: Some(Control::Start(Start {
                template_name,
                app_bundle_id: app_bundle_id.unwrap_or_default(),
                environment,
                arguments,
                timings: timings_msg,
                tool_arguments: vec![],
            })),
        })
        .await?;

        // Wait for RUNNING_INSTRUMENTS state, then process logs until stop signal
        let mut payload_data: Vec<u8> = Vec::new();
        let mut received_running = false;

        loop {
            tokio::select! {
                _ = stop_rx.changed() => {
                    if *stop_rx.borrow() {
                        // Send stop request
                        tx.send(InstrumentsRunRequest {
                            control: Some(Control::Stop(Stop {
                                post_process_arguments: post_process_arguments.clone(),
                            })),
                        }).await?;
                    }
                }
                msg = stream.message() => {
                    match msg? {
                        Some(response) => {
                            match response.output {
                                Some(Output::State(state)) => {
                                    use crate::proto::idb::instruments_run_response::State;
                                    match State::try_from(state) {
                                        Ok(State::RunningInstruments) => {
                                            received_running = true;
                                            eprintln!("Instruments running...");
                                        }
                                        Ok(State::PostProcessing) => {
                                            eprintln!("Post-processing...");
                                        }
                                        _ => {}
                                    }
                                }
                                Some(Output::LogOutput(data)) => {
                                    // Write log output to stderr
                                    std::io::stderr().write_all(&data)?;
                                    std::io::stderr().flush()?;
                                }
                                Some(Output::Payload(payload)) => {
                                    // Collect payload data
                                    if let Some(crate::proto::idb::payload::Source::Data(data)) = payload.source {
                                        payload_data.extend(data);
                                    }
                                }
                                None => {}
                            }
                        }
                        None => break, // Stream ended
                    }
                }
            }

            // Check if we should exit (received running state but haven't sent stop yet)
            if received_running && !*stop_rx.borrow() {
                continue;
            }
        }

        // Drop tx to close the request stream
        drop(tx);

        // Extract trace files from payload
        let trace_files = extract_trace_files(&payload_data, &trace_basename)?;

        Ok(trace_files)
    }

    /// Record xctrace (bidirectional streaming)
    ///
    /// Returns a list of trace file paths written to disk
    #[allow(clippy::too_many_arguments)]
    pub async fn xctrace_record(
        &mut self,
        template_name: String,
        time_limit: Option<f64>,
        package: Option<String>,
        target: XctraceTarget,
        stop_timeout: Option<f64>,
        post_args: Vec<String>,
        trace_basename: String,
        mut stop_rx: watch::Receiver<bool>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::proto::idb::xctrace_record_request::{self, Control, Start, Stop, Target};
        use crate::proto::idb::xctrace_record_response::Output;
        use crate::proto::idb::XctraceRecordRequest;

        let (tx, rx) = mpsc::channel::<XctraceRecordRequest>(4);
        let request_stream = ReceiverStream::new(rx);

        // Start the bidirectional stream first
        let response = self.client.xctrace_record(request_stream).await?;
        let mut stream = response.into_inner();

        // Build target message
        let target_msg = match target {
            XctraceTarget::AllProcesses => Target {
                target: Some(xctrace_record_request::target::Target::AllProcesses(true)),
            },
            XctraceTarget::Attach(process) => Target {
                target: Some(xctrace_record_request::target::Target::ProcessToAttach(
                    process,
                )),
            },
            XctraceTarget::Launch {
                process,
                args,
                stdin,
                stdout,
                env,
            } => Target {
                target: Some(xctrace_record_request::target::Target::LaunchProcess(
                    xctrace_record_request::LaunchProcess {
                        process_to_launch: process,
                        launch_args: args,
                        target_stdin: stdin.unwrap_or_default(),
                        target_stdout: stdout.unwrap_or_default(),
                        process_env: env,
                    },
                )),
            },
        };

        // Send start request
        tx.send(XctraceRecordRequest {
            control: Some(Control::Start(Start {
                template_name,
                time_limit: time_limit.unwrap_or(0.0),
                package: package.unwrap_or_default(),
                target: Some(target_msg),
            })),
        })
        .await?;

        // Wait for RUNNING state, then process logs until stop signal
        let mut payload_data: Vec<u8> = Vec::new();
        let mut received_running = false;

        loop {
            tokio::select! {
                _ = stop_rx.changed() => {
                    if *stop_rx.borrow() {
                        // Send stop request
                        tx.send(XctraceRecordRequest {
                            control: Some(Control::Stop(Stop {
                                timeout: stop_timeout.unwrap_or(0.0),
                                args: post_args.clone(),
                            })),
                        }).await?;
                    }
                }
                msg = stream.message() => {
                    match msg? {
                        Some(response) => {
                            match response.output {
                                Some(Output::State(state)) => {
                                    use crate::proto::idb::xctrace_record_response::State;
                                    match State::try_from(state) {
                                        Ok(State::Running) => {
                                            received_running = true;
                                            eprintln!("XCTrace recording...");
                                        }
                                        Ok(State::Processing) => {
                                            eprintln!("Processing...");
                                        }
                                        _ => {}
                                    }
                                }
                                Some(Output::Log(data)) => {
                                    // Write log output to stderr
                                    std::io::stderr().write_all(&data)?;
                                    std::io::stderr().flush()?;
                                }
                                Some(Output::Payload(payload)) => {
                                    // Collect payload data
                                    if let Some(crate::proto::idb::payload::Source::Data(data)) = payload.source {
                                        payload_data.extend(data);
                                    }
                                }
                                None => {}
                            }
                        }
                        None => break, // Stream ended
                    }
                }
            }

            // Check if we should exit
            if received_running && !*stop_rx.borrow() {
                continue;
            }
        }

        // Drop tx to close the request stream
        drop(tx);

        // Extract trace files from payload
        let trace_files = extract_trace_files(&payload_data, &trace_basename)?;

        Ok(trace_files)
    }
}
