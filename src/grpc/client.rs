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
use super::idb::{
    InstallRequest, InstallResponse, LaunchRequest, LogRequest, Payload, RmRequest,
    ScreenshotRequest, TargetDescriptionRequest,
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
}
