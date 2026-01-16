use crate::types::{Address, CompanionInfo, Compression, TargetDescription, TargetType};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::io::Write;
use tokio::net::UnixStream;
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use super::idb::companion_service_client::CompanionServiceClient;
use super::idb::install_request::{self, Destination};
use super::idb::launch_request::{self, Control};
use super::idb::log_request::Source as LogSource;
use super::idb::payload::Source as PayloadSource;
use super::idb::process_output::Interface;
use super::idb::tail_request::{self, Control as TailControl};
use super::idb::{
    InstallRequest, InstallResponse, LaunchRequest, LogRequest, Payload, ScreenshotRequest,
    TailRequest, TargetDescriptionRequest,
};

/// Configuration for launching an application
pub struct LaunchConfig {
    pub bundle_id: String,
    pub app_args: Vec<String>,
    pub env: HashMap<String, String>,
    pub foreground_if_running: bool,
    pub wait_for_debugger: bool,
}

pub struct IdbClient {
    client: CompanionServiceClient<Channel>,
    address: Address,
}

impl IdbClient {
    /// Connect to idb_companion via Unix Domain Socket
    pub async fn connect_uds(
        socket_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let socket_path_owned = socket_path.to_string();

        // For Unix Domain Sockets, we need a dummy URI but real socket connection
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(move |_: Uri| {
                let path = socket_path_owned.clone();
                async move {
                    let stream = UnixStream::connect(path).await?;
                    Ok::<_, std::io::Error>(TokioIo::new(stream))
                }
            }))
            .await?;

        let client = CompanionServiceClient::new(channel);

        Ok(Self {
            client,
            address: Address::DomainSocket {
                path: socket_path.to_string(),
            },
        })
    }

    /// Connect to idb_companion via TCP
    pub async fn connect_tcp(
        host: &str,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("http://{}:{}", host, port);
        let channel = Channel::from_shared(addr)?.connect().await?;
        let client = CompanionServiceClient::new(channel);

        Ok(Self {
            client,
            address: Address::Tcp {
                host: host.to_string(),
                port,
            },
        })
    }

    /// Call describe RPC to get target information
    pub async fn describe(
        &mut self,
        fetch_diagnostics: bool,
    ) -> Result<TargetDescription, Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(TargetDescriptionRequest { fetch_diagnostics });

        let response = self.client.describe(request).await?;
        let inner = response.into_inner();

        self.target_from_response(inner)
    }

    fn target_from_response(
        &self,
        response: super::idb::TargetDescriptionResponse,
    ) -> Result<TargetDescription, Box<dyn std::error::Error + Send + Sync>> {
        let target = response
            .target_description
            .ok_or("Missing target_description in response")?;

        let companion = response.companion;

        let companion_info = companion.map(|c| CompanionInfo {
            udid: c.udid,
            is_local: c.is_local,
            pid: None,
            address: self.address.clone(),
        });

        Ok(TargetDescription {
            name: target.name,
            udid: target.udid,
            state: if target.state.is_empty() {
                None
            } else {
                Some(target.state)
            },
            target_type: TargetType::from_proto_string(&target.target_type),
            os_version: if target.os_version.is_empty() {
                None
            } else {
                Some(target.os_version)
            },
            architecture: if target.architecture.is_empty() {
                None
            } else {
                Some(target.architecture)
            },
            companion_info,
        })
    }

    /// Launch an application via bidirectional streaming gRPC
    pub async fn launch(
        &mut self,
        config: LaunchConfig,
        wait_for: bool,
        mut stop_rx: watch::Receiver<bool>,
    ) -> Result<Option<u64>, Box<dyn std::error::Error + Send + Sync>> {
        // Create the initial Start request
        let start_request = LaunchRequest {
            control: Some(Control::Start(launch_request::Start {
                bundle_id: config.bundle_id,
                env: config.env,
                app_args: config.app_args,
                foreground_if_running: config.foreground_if_running,
                wait_for,
                wait_for_debugger: config.wait_for_debugger,
            })),
        };

        // Create channel for additional requests (like Stop)
        let (tx, rx) = mpsc::channel::<LaunchRequest>(4);

        // Create a stream that starts with the initial request, then chains additional requests
        let initial_stream = tokio_stream::once(start_request);
        let additional_stream = ReceiverStream::new(rx);
        let request_stream = tokio_stream::StreamExt::chain(initial_stream, additional_stream);

        // Start the bidirectional stream
        let response = self.client.launch(request_stream).await?;
        let mut response_stream = response.into_inner();

        let mut pid: Option<u64> = None;

        if wait_for {
            // Handle responses and stop signal concurrently when waiting for app to exit
            loop {
                tokio::select! {
                    // Check for stop signal (Ctrl+C)
                    _ = stop_rx.changed() => {
                        if *stop_rx.borrow() {
                            // Send Stop request
                            let stop_request = LaunchRequest {
                                control: Some(Control::Stop(launch_request::Stop {})),
                            };
                            let _ = tx.send(stop_request).await;
                            break;
                        }
                    }

                    // Process response stream
                    response = response_stream.message() => {
                        match response? {
                            Some(launch_response) => {
                                // Handle ProcessOutput
                                if let Some(output) = launch_response.output {
                                    let data = &output.data;
                                    match output.interface() {
                                        Interface::Stdout => {
                                            std::io::stdout().write_all(data)?;
                                            std::io::stdout().flush()?;
                                        }
                                        Interface::Stderr => {
                                            std::io::stderr().write_all(data)?;
                                            std::io::stderr().flush()?;
                                        }
                                    }
                                }

                                // Handle DebuggerInfo
                                if let Some(debugger) = launch_response.debugger {
                                    // Output PID as JSON (matching Python idb behavior)
                                    println!("{{\"pid\": {}}}", debugger.pid);
                                    pid = Some(debugger.pid);
                                }
                            }
                            None => break, // Stream ended
                        }
                    }
                }
            }
        } else {
            // Close the send side immediately to signal we're done sending
            drop(tx);

            // Drain responses without waiting for stop signal
            while let Some(launch_response) = response_stream.message().await? {
                // Handle ProcessOutput
                if let Some(output) = launch_response.output {
                    let data = &output.data;
                    match output.interface() {
                        Interface::Stdout => {
                            std::io::stdout().write_all(data)?;
                            std::io::stdout().flush()?;
                        }
                        Interface::Stderr => {
                            std::io::stderr().write_all(data)?;
                            std::io::stderr().flush()?;
                        }
                    }
                }

                // Handle DebuggerInfo
                if let Some(debugger) = launch_response.debugger {
                    // Output PID as JSON (matching Python idb behavior)
                    println!("{{\"pid\": {}}}", debugger.pid);
                    pid = Some(debugger.pid);
                }
            }
        }

        Ok(pid)
    }

    /// Take a screenshot and return the image data as PNG bytes
    pub async fn screenshot(
        &mut self,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(ScreenshotRequest {});
        let response = self.client.screenshot(request).await?;
        let inner = response.into_inner();
        Ok(inner.image_data)
    }

    /// Bring simulator window to front
    pub async fn focus(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::FocusRequest {});
        let response = self.client.focus(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Uninstall an application
    pub async fn uninstall(
        &mut self,
        bundle_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::UninstallRequest {
            bundle_id: bundle_id.to_string(),
        });
        let response = self.client.uninstall(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Tail logs from target or companion (server-side streaming)
    pub async fn log(
        &mut self,
        source: LogSource,
        arguments: Vec<String>,
    ) -> Result<tonic::Streaming<super::idb::LogResponse>, Box<dyn std::error::Error + Send + Sync>>
    {
        let request = tonic::Request::new(LogRequest {
            arguments,
            source: source.into(),
        });

        let response = self.client.log(request).await?;
        Ok(response.into_inner())
    }

    /// Install an application via bidirectional streaming gRPC
    pub async fn install(
        &mut self,
        bundle_path: &str,
        make_debuggable: bool,
        override_mtime: bool,
        compression: Option<Compression>,
    ) -> Result<tonic::Streaming<InstallResponse>, Box<dyn std::error::Error + Send + Sync>> {
        // Build the sequence of install requests
        let mut requests: Vec<InstallRequest> = Vec::new();

        // 1. Destination request (APP)
        requests.push(InstallRequest {
            value: Some(install_request::Value::Destination(Destination::App as i32)),
        });

        // 2. Payload with file path
        requests.push(InstallRequest {
            value: Some(install_request::Value::Payload(Payload {
                source: Some(PayloadSource::FilePath(bundle_path.to_string())),
            })),
        });

        // 3. Optional: make_debuggable
        if make_debuggable {
            requests.push(InstallRequest {
                value: Some(install_request::Value::MakeDebuggable(true)),
            });
        }

        // 4. Optional: override_modification_time
        if override_mtime {
            requests.push(InstallRequest {
                value: Some(install_request::Value::OverrideModificationTime(true)),
            });
        }

        // 5. Optional: compression
        if let Some(comp) = compression {
            let compression_enum = match comp {
                Compression::Gzip => super::idb::payload::Compression::Gzip,
                Compression::Zstd => super::idb::payload::Compression::Zstd,
            };
            requests.push(InstallRequest {
                value: Some(install_request::Value::Payload(Payload {
                    source: Some(PayloadSource::Compression(compression_enum as i32)),
                })),
            });
        }

        // Create stream from requests
        let request_stream = tokio_stream::iter(requests);

        // Start the bidirectional stream
        let response = self.client.install(request_stream).await?;
        Ok(response.into_inner())
    }

    /// Tail a file on the target device via bidirectional streaming
    pub async fn tail(
        &mut self,
        path: String,
        container: Option<super::idb::FileContainer>,
        mut stop_rx: watch::Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create the initial Start request
        let start_request = TailRequest {
            control: Some(TailControl::Start(tail_request::Start { container, path })),
        };

        // Create channel for additional requests (like Stop)
        let (tx, rx) = mpsc::channel::<TailRequest>(4);

        // Create a stream that starts with the initial request, then chains additional requests
        let initial_stream = tokio_stream::once(start_request);
        let additional_stream = ReceiverStream::new(rx);
        let request_stream = tokio_stream::StreamExt::chain(initial_stream, additional_stream);

        // Start the bidirectional stream
        let response = self.client.tail(request_stream).await?;
        let mut response_stream = response.into_inner();

        // Handle responses and stop signal concurrently
        loop {
            tokio::select! {
                // Check for stop signal (Ctrl+C)
                _ = stop_rx.changed() => {
                    if *stop_rx.borrow() {
                        // Send Stop request
                        let stop_request = TailRequest {
                            control: Some(TailControl::Stop(tail_request::Stop {})),
                        };
                        let _ = tx.send(stop_request).await;
                        break;
                    }
                }

                // Process response stream
                response = response_stream.message() => {
                    match response? {
                        Some(tail_response) => {
                            // Write data directly to stdout
                            std::io::stdout().write_all(&tail_response.data)?;
                            std::io::stdout().flush()?;
                        }
                        None => break, // Stream ended
                    }
                }
            }
        }

        Ok(())
    }

    /// Pull a file from the target via server streaming
    pub async fn pull(
        &mut self,
        src_path: String,
        container: Option<super::idb::FileContainer>,
    ) -> Result<tonic::Streaming<super::idb::PullResponse>, Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::PullRequest;

        let request = tonic::Request::new(PullRequest {
            src_path,
            dst_path: String::new(),
            container,
        });

        let response = self.client.pull(request).await?;
        Ok(response.into_inner())
    }

    /// Push a file to the target via client streaming
    pub async fn push(
        &mut self,
        src_path: String,
        dst_path: String,
        container: Option<super::idb::FileContainer>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::push_request::{self, Value};
        use super::idb::PushRequest;

        // Create channel for streaming requests
        let (tx, rx) = mpsc::channel::<PushRequest>(32);

        // Send initial Inner request with metadata
        let inner_request = PushRequest {
            value: Some(Value::Inner(push_request::Inner {
                dst_path,
                container,
            })),
        };
        tx.send(inner_request).await?;

        // Clone src_path for the spawned task
        let src_path_clone = src_path.clone();
        let file_tx = tx.clone();

        // Spawn task to read file and send chunks
        tokio::spawn(async move {
            match tokio::fs::File::open(&src_path_clone).await {
                Ok(mut file) => {
                    use tokio::io::AsyncReadExt;
                    let mut buffer = vec![0u8; 64 * 1024]; // 64KB chunks

                    loop {
                        match file.read(&mut buffer).await {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                let chunk = PushRequest {
                                    value: Some(Value::Payload(Payload {
                                        source: Some(PayloadSource::Data(buffer[..n].to_vec())),
                                    })),
                                };
                                if file_tx.send(chunk).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
                Err(_) => {
                    // File open failed, channel will be dropped
                }
            }
            drop(file_tx);
        });

        // Drop the original tx to signal we won't send more from this context
        drop(tx);

        // Create request stream from receiver
        let request_stream = ReceiverStream::new(rx);

        // Send the streaming request
        let _response = self.client.push(request_stream).await?;

        Ok(())
    }

    /// Run XCTest in logic mode
    pub async fn xctest_run_logic(
        &mut self,
        test_bundle_id: String,
        tests_to_run: Vec<String>,
    ) -> Result<
        tonic::Streaming<super::idb::XctestRunResponse>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        use super::idb::xctest_run_request;

        let request = super::idb::XctestRunRequest {
            mode: Some(xctest_run_request::Mode {
                mode: Some(xctest_run_request::mode::Mode::Logic(
                    xctest_run_request::Logic {},
                )),
            }),
            test_bundle_id,
            tests_to_run,
            tests_to_skip: vec![],
            arguments: vec![],
            environment: HashMap::new(),
            timeout: 0,
            report_activities: false,
            collect_coverage: false,
            report_attachments: false,
            collect_logs: false,
            wait_for_debugger: false,
            code_coverage: None,
            collect_result_bundle: false,
        };

        let response = self.client.xctest_run(request).await?;
        Ok(response.into_inner())
    }

    /// List crash logs
    pub async fn crash_list(
        &mut self,
        since: u64,
        before: u64,
        bundle_id: String,
        name: String,
    ) -> Result<Vec<super::idb::CrashLogInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let query = super::idb::CrashLogQuery {
            since,
            before,
            bundle_id,
            name,
        };

        let request = tonic::Request::new(query);
        let response = self.client.crash_list(request).await?;
        Ok(response.into_inner().list)
    }

    /// Show crash log contents
    pub async fn crash_show(
        &mut self,
        name: String,
    ) -> Result<super::idb::CrashShowResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::CrashShowRequest { name });
        let response = self.client.crash_show(request).await?;
        Ok(response.into_inner())
    }

    /// Delete crash logs
    pub async fn crash_delete(
        &mut self,
        since: u64,
        before: u64,
        bundle_id: String,
        name: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = super::idb::CrashLogQuery {
            since,
            before,
            bundle_id,
            name,
        };

        let request = tonic::Request::new(query);
        let _response = self.client.crash_delete(request).await?;
        Ok(())
    }

    /// Set a device setting
    pub async fn set_setting(
        &mut self,
        setting: super::idb::setting_request::Setting,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::SettingRequest;

        let request = tonic::Request::new(SettingRequest {
            setting: Some(setting),
        });

        let _response = self.client.setting(request).await?;
        Ok(())
    }

    /// Get a device setting value
    pub async fn get_setting(
        &mut self,
        setting: super::idb::Setting,
        name: String,
        domain: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::GetSettingRequest;

        let request = tonic::Request::new(GetSettingRequest {
            setting: setting as i32,
            name,
            domain,
        });

        let response = self.client.get_setting(request).await?;
        Ok(response.into_inner().value)
    }

    /// List available settings
    pub async fn list_settings(
        &mut self,
        setting: super::idb::Setting,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::ListSettingRequest;

        let request = tonic::Request::new(ListSettingRequest {
            setting: setting as i32,
        });

        let response = self.client.list_settings(request).await?;
        Ok(response.into_inner().values)
    }
}
