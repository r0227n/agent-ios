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
use super::idb::push_request;
use super::idb::tail_request::{self, Control as TailControl};
use super::idb::xctest_run_request::{self, Mode};
use super::idb::AddMediaRequest;
use super::idb::{
    InstallRequest, InstallResponse, LaunchRequest, LogRequest, Payload, PullRequest, PushRequest,
    RmRequest, ScreenshotRequest, TailRequest, TargetDescriptionRequest, XctestRunRequest,
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

    /// Add media files (photos/videos) to the device
    pub async fn add_media(
        &mut self,
        file_paths: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Build requests for each file
        let requests: Vec<AddMediaRequest> = file_paths
            .into_iter()
            .map(|path| AddMediaRequest {
                payload: Some(Payload {
                    source: Some(PayloadSource::FilePath(path)),
                }),
            })
            .collect();

        // Send the request stream
        let request_stream = tokio_stream::iter(requests);
        let response = self.client.add_media(request_stream).await?;
        let _inner = response.into_inner();

        Ok(())
    }

    /// Clear all photos from the device
    pub async fn photos_clear(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::PhotosClearRequest {});
        let response = self.client.photos_clear(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Record video to a file (bidirectional streaming)
    /// The recording continues until stop_rx receives a signal
    /// Note: format and fps parameters are accepted for CLI compatibility but not used by gRPC
    pub async fn record_video(
        &mut self,
        output_file: String,
        _format: String,
        _fps: Option<u64>,
        mut stop_rx: watch::Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::record_request::{Control, Start, Stop};
        use super::idb::RecordRequest;

        // Create channel for sending requests
        let (tx, rx) = tokio::sync::mpsc::channel::<RecordRequest>(4);
        let request_stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        // Start the bidirectional stream first
        let response = self.client.record(request_stream).await?;
        let mut stream = response.into_inner();

        // Now send the start request
        tx.send(RecordRequest {
            control: Some(Control::Start(Start {
                file_path: output_file.clone(),
            })),
        })
        .await?;

        // Wait for stop signal while draining responses
        loop {
            tokio::select! {
                _ = stop_rx.changed() => {
                    // Stop signal received, send stop request
                    tx.send(RecordRequest {
                        control: Some(Control::Stop(Stop {})),
                    }).await?;
                    // Drop tx to close the request stream
                    drop(tx);
                    // Wait for the final response (companion confirms recording saved)
                    while let Ok(Some(_)) = stream.message().await {}
                    break;
                }
                msg = stream.message() => {
                    match msg {
                        Ok(Some(_response)) => {
                            // Continue receiving (log_output or payload)
                        }
                        Ok(None) => break, // Stream ended
                        Err(e) => return Err(e.into()),
                    }
                }
            }
        }

        Ok(())
    }

    /// Stream video data (bidirectional streaming)
    pub async fn video_stream(
        &mut self,
        output_file: Option<String>,
        fps: Option<u64>,
        format: super::idb::video_stream_request::Format,
        compression_quality: f64,
        scale_factor: f64,
        mut stop_rx: watch::Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::video_stream_request::{Control, Start, Stop};
        use super::idb::VideoStreamRequest;
        use std::io::Write;

        // Create channel for sending requests
        let (tx, rx) = tokio::sync::mpsc::channel::<VideoStreamRequest>(4);
        let request_stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        // Start the bidirectional stream first
        let response = self.client.video_stream(request_stream).await?;
        let mut stream = response.into_inner();

        // Now send start request
        tx.send(VideoStreamRequest {
            control: Some(Control::Start(Start {
                file_path: output_file.clone().unwrap_or_default(),
                fps: fps.unwrap_or(0),
                format: format.into(),
                compression_quality,
                scale_factor,
                avg_bitrate: 0.0,
                key_frame_rate: 0.0,
            })),
        })
        .await?;

        // Create output: file or stdout
        let mut file_output: Option<std::fs::File> = output_file
            .as_ref()
            .map(std::fs::File::create)
            .transpose()?;

        // Stream data until stop signal
        loop {
            tokio::select! {
                _ = stop_rx.changed() => {
                    // Stop signal received
                    let _ = tx.send(VideoStreamRequest {
                        control: Some(Control::Stop(Stop {})),
                    }).await;
                    drop(tx);
                    break;
                }
                msg = stream.message() => {
                    match msg {
                        Ok(Some(response)) => {
                            if let Some(super::idb::video_stream_response::Output::Payload(payload)) = response.output {
                                if let Some(super::idb::payload::Source::Data(data)) = payload.source {
                                    if let Some(ref mut file) = file_output {
                                        file.write_all(&data)?;
                                    } else {
                                        std::io::stdout().write_all(&data)?;
                                        std::io::stdout().flush()?;
                                    }
                                }
                            }
                        }
                        Ok(None) => break, // Stream ended
                        Err(e) => return Err(e.into()),
                    }
                }
            }
        }

        // Drain remaining messages
        while let Ok(Some(_)) = stream.message().await {}

        if let Some(ref mut file) = file_output {
            file.flush()?;
        }

        Ok(())
    }

    /// Get accessibility information
    pub async fn accessibility_info(
        &mut self,
        point: Option<(f64, f64)>,
        nested: bool,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::accessibility_info_request::Format;
        use super::idb::Point;

        let request = tonic::Request::new(super::idb::AccessibilityInfoRequest {
            point: point.map(|(x, y)| Point { x, y }),
            format: if nested {
                Format::Nested as i32
            } else {
                Format::Legacy as i32
            },
        });
        let response = self.client.accessibility_info(request).await?;
        Ok(response.into_inner().json)
    }

    /// Update contacts database
    pub async fn contacts_update(
        &mut self,
        db_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::ContactsUpdateRequest {
            payload: Some(Payload {
                source: Some(PayloadSource::FilePath(db_path.to_string())),
            }),
        });
        let response = self.client.contacts_update(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Clear all contacts
    pub async fn contacts_clear(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::ContactsClearRequest {});
        let response = self.client.contacts_clear(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Clear keychain
    pub async fn clear_keychain(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::ClearKeychainRequest {});
        let response = self.client.clear_keychain(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Simulate memory warning
    pub async fn simulate_memory_warning(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::SimulateMemoryWarningRequest {});
        let response = self.client.simulate_memory_warning(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Bring simulator window to front
    pub async fn focus(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::FocusRequest {});
        let response = self.client.focus(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Approve (grant) app permissions
    pub async fn approve(
        &mut self,
        bundle_id: &str,
        permissions: Vec<i32>,
        scheme: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::ApproveRequest {
            bundle_id: bundle_id.to_string(),
            permissions,
            scheme: scheme.unwrap_or_default(),
        });
        let response = self.client.approve(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Open a URL on the device
    pub async fn open_url(
        &mut self,
        url: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::OpenUrlRequest {
            url: url.to_string(),
        });
        let response = self.client.open_url(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Revoke app permissions
    pub async fn revoke(
        &mut self,
        bundle_id: &str,
        permissions: Vec<i32>,
        scheme: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::RevokeRequest {
            bundle_id: bundle_id.to_string(),
            permissions,
            scheme: scheme.unwrap_or_default(),
        });
        let response = self.client.revoke(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Send a push notification to the device
    pub async fn send_notification(
        &mut self,
        bundle_id: &str,
        json_payload: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::SendNotificationRequest {
            bundle_id: bundle_id.to_string(),
            json_payload: json_payload.to_string(),
        });
        let response = self.client.send_notification(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Set device location
    pub async fn set_location(
        &mut self,
        latitude: f64,
        longitude: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::SetLocationRequest {
            location: Some(super::idb::Location {
                latitude,
                longitude,
            }),
        });
        let response = self.client.set_location(request).await?;
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

    /// List installed applications
    pub async fn list_apps(
        &mut self,
    ) -> Result<Vec<super::idb::InstalledAppInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::ListAppsRequest {
            suppress_process_state: false,
        });
        let response = self.client.list_apps(request).await?;
        let inner = response.into_inner();
        Ok(inner.apps)
    }

    /// Terminate a running application
    pub async fn terminate(
        &mut self,
        bundle_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::TerminateRequest {
            bundle_id: bundle_id.to_string(),
        });
        let response = self.client.terminate(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Create a directory on the target
    pub async fn mkdir(
        &mut self,
        path: &str,
        container: super::idb::FileContainer,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::MkdirRequest {
            path: path.to_string(),
            container: Some(container),
        });
        let response = self.client.mkdir(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Move files/directories on the target
    pub async fn mv(
        &mut self,
        src_paths: Vec<String>,
        dst_path: &str,
        container: super::idb::FileContainer,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::MvRequest {
            src_paths,
            dst_path: dst_path.to_string(),
            container: Some(container),
        });
        let response = self.client.mv(request).await?;
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
    pub async fn install_xctest(
        &mut self,
        bundle_path: &str,
        skip_signing: bool,
        compression: Option<Compression>,
    ) -> Result<tonic::Streaming<InstallResponse>, Box<dyn std::error::Error + Send + Sync>> {
        // Build the sequence of install requests for XCTest
        let mut requests: Vec<InstallRequest> = Vec::new();

        // 1. Destination request (XCTEST)
        requests.push(InstallRequest {
            value: Some(install_request::Value::Destination(
                Destination::Xctest as i32,
            )),
        });

        // 2. Payload with file path
        requests.push(InstallRequest {
            value: Some(install_request::Value::Payload(Payload {
                source: Some(PayloadSource::FilePath(bundle_path.to_string())),
            })),
        });

        // 3. Optional: skip_signing_bundles
        if skip_signing {
            requests.push(InstallRequest {
                value: Some(install_request::Value::SkipSigningBundles(true)),
            });
        }

        // 4. Optional: compression
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

        // Send the request stream
        let request_stream = tokio_stream::iter(requests);
        let response = self.client.install(request_stream).await?;
        Ok(response.into_inner())
    }

    /// Install a dSYM bundle
    pub async fn install_dsym(
        &mut self,
        dsym_path: &str,
        bundle_id: Option<String>,
        compression: Option<Compression>,
    ) -> Result<tonic::Streaming<InstallResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let mut requests: Vec<InstallRequest> = Vec::new();

        // 1. Destination request (DSYM)
        requests.push(InstallRequest {
            value: Some(install_request::Value::Destination(
                Destination::Dsym as i32,
            )),
        });

        // 2. Payload with file path
        requests.push(InstallRequest {
            value: Some(install_request::Value::Payload(Payload {
                source: Some(PayloadSource::FilePath(dsym_path.to_string())),
            })),
        });

        // 3. Optional: bundle_id (link to app container)
        if let Some(bid) = bundle_id {
            requests.push(InstallRequest {
                value: Some(install_request::Value::NameHint(bid)),
            });
        }

        // 4. Optional: compression
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

        let request_stream = tokio_stream::iter(requests);
        let response = self.client.install(request_stream).await?;
        Ok(response.into_inner())
    }

    /// Install a dylib
    pub async fn install_dylib(
        &mut self,
        dylib_path: &str,
    ) -> Result<tonic::Streaming<InstallResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let requests: Vec<InstallRequest> = vec![
            InstallRequest {
                value: Some(install_request::Value::Destination(
                    Destination::Dylib as i32,
                )),
            },
            InstallRequest {
                value: Some(install_request::Value::Payload(Payload {
                    source: Some(PayloadSource::FilePath(dylib_path.to_string())),
                })),
            },
        ];

        let request_stream = tokio_stream::iter(requests);
        let response = self.client.install(request_stream).await?;
        Ok(response.into_inner())
    }

    /// Install a framework
    pub async fn install_framework(
        &mut self,
        framework_path: &str,
    ) -> Result<tonic::Streaming<InstallResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let requests: Vec<InstallRequest> = vec![
            InstallRequest {
                value: Some(install_request::Value::Destination(
                    Destination::Framework as i32,
                )),
            },
            InstallRequest {
                value: Some(install_request::Value::Payload(Payload {
                    source: Some(PayloadSource::FilePath(framework_path.to_string())),
                })),
            },
        ];

        let request_stream = tokio_stream::iter(requests);
        let response = self.client.install(request_stream).await?;
        Ok(response.into_inner())
    }

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

    /// Remove files or directories inside a container
    pub async fn rm(
        &mut self,
        paths: Vec<String>,
        container: Option<super::idb::FileContainer>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(RmRequest { paths, container });
        let response = self.client.rm(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// List files on the target device/simulator
    pub async fn ls(
        &mut self,
        path: String,
        paths: Vec<String>,
        container: Option<super::idb::FileContainer>,
    ) -> Result<super::idb::LsResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::LsRequest {
            path,
            container,
            paths,
        });
        let response = self.client.ls(request).await?;
        Ok(response.into_inner())
    }

    /// List installed XCTest bundles
    pub async fn xctest_list_bundles(
        &mut self,
    ) -> Result<
        Vec<super::idb::xctest_list_bundles_response::Bundles>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let request = tonic::Request::new(super::idb::XctestListBundlesRequest {});
        let response = self.client.xctest_list_bundles(request).await?;
        Ok(response.into_inner().bundles)
    }

    /// List tests inside an installed test bundle
    pub async fn xctest_list_tests(
        &mut self,
        bundle_name: String,
        app_path: Option<String>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::XctestListTestsRequest {
            bundle_name,
            app_path: app_path.unwrap_or_default(),
        });
        let response = self.client.xctest_list_tests(request).await?;
        Ok(response.into_inner().names)
    }

    /// List crash logs
    pub async fn crash_list(
        &mut self,
        since: Option<u64>,
        before: Option<u64>,
        bundle_id: Option<String>,
        name: Option<String>,
    ) -> Result<Vec<super::idb::CrashLogInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let query = super::idb::CrashLogQuery {
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
    ) -> Result<super::idb::CrashShowResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(super::idb::CrashShowRequest {
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
    ) -> Result<Vec<super::idb::CrashLogInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let query = super::idb::CrashLogQuery {
            since: since.unwrap_or(0),
            before: before.unwrap_or(0),
            bundle_id: bundle_id.unwrap_or_default(),
            name: name.unwrap_or_default(),
        };

        let request = tonic::Request::new(query);
        let response = self.client.crash_delete(request).await?;
        Ok(response.into_inner().list)
    }

    // ========== Phase 2: Streaming File Operations ==========

    /// Pull a file from the target device (server-side streaming)
    pub async fn pull(
        &mut self,
        src_path: String,
        container: Option<super::idb::FileContainer>,
    ) -> Result<tonic::Streaming<super::idb::PullResponse>, Box<dyn std::error::Error + Send + Sync>>
    {
        let request = tonic::Request::new(PullRequest {
            src_path,
            dst_path: String::new(),
            container,
        });

        let response = self.client.pull(request).await?;
        Ok(response.into_inner())
    }

    /// Pull a file to a local destination path (for local companions)
    pub async fn pull_to_file(
        &mut self,
        src_path: String,
        dst_path: String,
        container: Option<super::idb::FileContainer>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(PullRequest {
            src_path,
            dst_path,
            container,
        });

        let response = self.client.pull(request).await?;
        let mut stream = response.into_inner();

        // Drain the stream (companion writes to disk, we just need to consume responses)
        while (stream.message().await?).is_some() {}

        Ok(())
    }

    /// Push a file to the target device (client-side streaming)
    pub async fn push(
        &mut self,
        src_path: String,
        dst_path: String,
        container: Option<super::idb::FileContainer>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let requests: Vec<PushRequest> = vec![
            // 1. Send Inner with dst_path and container
            PushRequest {
                value: Some(push_request::Value::Inner(push_request::Inner {
                    dst_path,
                    container,
                })),
            },
            // 2. Send Payload with file path
            PushRequest {
                value: Some(push_request::Value::Payload(Payload {
                    source: Some(PayloadSource::FilePath(src_path)),
                })),
            },
        ];

        let request_stream = tokio_stream::iter(requests);
        let response = self.client.push(request_stream).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Tail a file on the target device (bidirectional streaming)
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
                    match response {
                        Ok(Some(tail_response)) => {
                            // Write tail data to stdout
                            std::io::stdout().write_all(&tail_response.data)?;
                            std::io::stdout().flush()?;
                        }
                        Ok(None) => break, // Stream ended normally
                        Err(e) => {
                            // gRPC error - likely file not found or permission denied
                            eprintln!("Error: {}", e.message());
                            return Err(e.into());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // ========== Phase 2: XCTest Execution ==========

    /// Run XCTest tests in logic mode (server-side streaming)
    pub async fn xctest_run_logic(
        &mut self,
        test_bundle_id: String,
        tests_to_run: Vec<String>,
    ) -> Result<
        tonic::Streaming<super::idb::XctestRunResponse>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let request = tonic::Request::new(XctestRunRequest {
            mode: Some(Mode {
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
        });

        let response = self.client.xctest_run(request).await?;
        Ok(response.into_inner())
    }

    // ========== Settings Operations ==========

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
        name: Option<String>,
        domain: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::GetSettingRequest;

        let request = tonic::Request::new(GetSettingRequest {
            setting: setting as i32,
            name: name.unwrap_or_default(),
            domain: domain.unwrap_or_default(),
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

    // ========== HID (Human Interface Device) ==========

    /// Send HID events to the device (client-side streaming)
    pub async fn hid(
        &mut self,
        events: Vec<super::idb::HidEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request_stream = tokio_stream::iter(events);
        let response = self.client.hid(request_stream).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    // ========== Debug Server ==========

    /// Start the debug server for a bundle
    /// Returns LLDB bootstrap commands
    pub async fn debugserver_start(
        &mut self,
        bundle_id: String,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::debug_server_request::{Control, Start};
        use super::idb::DebugServerRequest;

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
            if let Some(super::idb::debug_server_response::Control::Status(status)) = resp.control {
                return Ok(status.lldb_bootstrap_commands);
            }
        }

        Err("No response from debugserver start".into())
    }

    /// Stop the debug server
    pub async fn debugserver_stop(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::debug_server_request::{Control, Stop};
        use super::idb::DebugServerRequest;

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
        use super::idb::debug_server_request::{Control, Status};
        use super::idb::DebugServerRequest;

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
            if let Some(super::idb::debug_server_response::Control::Status(status)) = resp.control {
                let commands = status.lldb_bootstrap_commands;
                if commands.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(commands));
            }
        }

        Ok(None)
    }

    // ========== DAP (Debug Adapter Protocol) ==========

    /// Start a DAP debug session
    /// This bridges stdin/stdout to the remote DAP server
    pub async fn dap(
        &mut self,
        pkg_id: String,
        _port: Option<u16>,
        mut stop_rx: watch::Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use super::idb::dap_request::{Control, Pipe, Start, Stop};
        use super::idb::DapRequest;
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
                    Some(super::idb::dap_response::Output::Started(_))
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
                                Some(super::idb::dap_response::Output::Stdout(pipe)) => {
                                    stdout.write_all(&pipe.data).await?;
                                    stdout.flush().await?;
                                }
                                Some(super::idb::dap_response::Output::Stopped(_)) => {
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

    // ========== Instruments Profiling ==========

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
        use super::idb::instruments_run_request::{self, Control, Start, Stop};
        use super::idb::instruments_run_response::Output;
        use super::idb::InstrumentsRunRequest;
        use std::io::Write;

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
                                    use super::idb::instruments_run_response::State;
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
                                    if let Some(super::idb::payload::Source::Data(data)) = payload.source {
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

    // ========== XCTrace Recording ==========

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
        use super::idb::xctrace_record_request::{self, Control, Start, Stop, Target};
        use super::idb::xctrace_record_response::Output;
        use super::idb::XctraceRecordRequest;
        use std::io::Write;

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
                                    use super::idb::xctrace_record_response::State;
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
                                    if let Some(super::idb::payload::Source::Data(data)) = payload.source {
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

/// Timings configuration for instruments
pub struct InstrumentsTimings {
    pub operation_duration: Option<f64>,
    pub terminate_timeout: Option<f64>,
    pub launch_retry_timeout: Option<f64>,
    pub launch_error_timeout: Option<f64>,
}

/// Target specification for xctrace
pub enum XctraceTarget {
    AllProcesses,
    Attach(String),
    Launch {
        process: String,
        args: Vec<String>,
        stdin: Option<String>,
        stdout: Option<String>,
        env: HashMap<String, String>,
    },
}

/// Extract trace files from tar payload data
fn extract_trace_files(
    payload_data: &[u8],
    trace_basename: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    use flate2::read::GzDecoder;
    use std::fs;
    use std::io::Cursor;
    use tar::Archive;

    if payload_data.is_empty() {
        return Ok(vec![]);
    }

    // Create temp dir to extract to
    let temp_dir = tempfile::tempdir()?;

    // Try to extract as gzipped tar
    let cursor = Cursor::new(payload_data);
    let gz = GzDecoder::new(cursor);
    let mut archive = Archive::new(gz);

    // Extract all files
    archive.unpack(temp_dir.path())?;

    // Look for trace files or instrument_data directory
    let mut trace_files = Vec::new();

    // Check if there's an instrument_data directory (Instruments format)
    let instrument_data_path = temp_dir.path().join("instrument_data");
    if instrument_data_path.exists() {
        // Copy as .trace directory
        let trace_path = format!("{}.trace", trace_basename);
        fs::rename(&instrument_data_path, &trace_path)?;
        trace_files.push(trace_path);
    } else {
        // Look for individual trace files
        for entry in fs::read_dir(temp_dir.path())? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name() {
                let dest = format!("{}", name.to_string_lossy());
                fs::rename(&path, &dest)?;
                trace_files.push(dest);
            }
        }
    }

    Ok(trace_files)
}

/// Read a DAP protocol message from a reader
/// DAP messages have format: "Content-Length: <length>\r\n\r\n<body>"
async fn read_dap_message<R: tokio::io::AsyncBufRead + tokio::io::AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::{AsyncBufReadExt, AsyncReadExt};

    // Read headers until empty line
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Ok(None); // EOF
        }
        headers.push_str(&line);
        if line == "\r\n" || line == "\n" {
            break;
        }
    }

    // Parse Content-Length header
    let content_length = headers
        .lines()
        .find(|line| line.to_lowercase().starts_with("content-length:"))
        .and_then(|line| line.split(':').nth(1))
        .and_then(|s| s.trim().parse::<usize>().ok())
        .ok_or("Missing or invalid Content-Length header")?;

    // Read body
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).await?;

    // Return full message (headers + body)
    let mut message = headers.into_bytes();
    message.extend(body);
    Ok(Some(message))
}
