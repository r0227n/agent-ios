//! Application management operations for IdbClient.

use std::io::Write;
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;

use crate::platform::ios::proto::idb::install_request::{self, Destination};
use crate::platform::ios::proto::idb::launch_request::{self, Control};
use crate::platform::ios::proto::idb::payload::Source as PayloadSource;
use crate::platform::ios::proto::idb::process_output::Interface;
use crate::platform::ios::proto::idb::{InstallRequest, InstallResponse, LaunchRequest, Payload};
use crate::types::Compression;

use super::client::{IdbClient, LaunchConfig};

impl IdbClient {
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
                Compression::Gzip => crate::platform::ios::proto::idb::payload::Compression::Gzip,
                Compression::Zstd => crate::platform::ios::proto::idb::payload::Compression::Zstd,
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

    /// Uninstall an application
    pub async fn uninstall(
        &mut self,
        bundle_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(crate::platform::ios::proto::idb::UninstallRequest {
            bundle_id: bundle_id.to_string(),
        });
        let response = self.client.uninstall(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// List installed applications
    pub async fn list_apps(
        &mut self,
    ) -> Result<
        Vec<crate::platform::ios::proto::idb::InstalledAppInfo>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let request = tonic::Request::new(crate::platform::ios::proto::idb::ListAppsRequest {
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
        let request = tonic::Request::new(crate::platform::ios::proto::idb::TerminateRequest {
            bundle_id: bundle_id.to_string(),
        });
        let response = self.client.terminate(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }
}
