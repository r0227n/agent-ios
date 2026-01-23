//! File operations for IdbClient.

use std::io::Write;
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;

use crate::proto::idb::payload::Source as PayloadSource;
use crate::proto::idb::push_request;
use crate::proto::idb::tail_request::{self, Control as TailControl};
use crate::proto::idb::{Payload, PullRequest, PushRequest, RmRequest, TailRequest};

use super::client::IdbClient;

impl IdbClient {
    /// List files on the target device/simulator
    pub async fn ls(
        &mut self,
        path: String,
        paths: Vec<String>,
        container: Option<crate::proto::idb::FileContainer>,
    ) -> Result<crate::proto::idb::LsResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(crate::proto::idb::LsRequest {
            path,
            container,
            paths,
        });
        let response = self.client.ls(request).await?;
        Ok(response.into_inner())
    }

    /// Create a directory on the target
    pub async fn mkdir(
        &mut self,
        path: &str,
        container: crate::proto::idb::FileContainer,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(crate::proto::idb::MkdirRequest {
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
        container: crate::proto::idb::FileContainer,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(crate::proto::idb::MvRequest {
            src_paths,
            dst_path: dst_path.to_string(),
            container: Some(container),
        });
        let response = self.client.mv(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Remove files or directories inside a container
    pub async fn rm(
        &mut self,
        paths: Vec<String>,
        container: Option<crate::proto::idb::FileContainer>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(RmRequest { paths, container });
        let response = self.client.rm(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Pull a file from the target device (server-side streaming)
    pub async fn pull(
        &mut self,
        src_path: String,
        container: Option<crate::proto::idb::FileContainer>,
    ) -> Result<
        tonic::Streaming<crate::proto::idb::PullResponse>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
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
        container: Option<crate::proto::idb::FileContainer>,
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
        container: Option<crate::proto::idb::FileContainer>,
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
        container: Option<crate::proto::idb::FileContainer>,
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
}
