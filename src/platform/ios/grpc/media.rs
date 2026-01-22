//! Media operations for IdbClient.

use std::io::Write;
use tokio::sync::watch;

use crate::platform::ios::proto::idb::payload::Source as PayloadSource;
use crate::platform::ios::proto::idb::{AddMediaRequest, Payload};

use super::client::IdbClient;

impl IdbClient {
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
        let request = tonic::Request::new(crate::platform::ios::proto::idb::PhotosClearRequest {});
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
        use crate::platform::ios::proto::idb::record_request::{Control, Start, Stop};
        use crate::platform::ios::proto::idb::RecordRequest;

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
        format: crate::platform::ios::proto::idb::video_stream_request::Format,
        compression_quality: f64,
        scale_factor: f64,
        mut stop_rx: watch::Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::platform::ios::proto::idb::video_stream_request::{Control, Start, Stop};
        use crate::platform::ios::proto::idb::VideoStreamRequest;

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
                            if let Some(crate::platform::ios::proto::idb::video_stream_response::Output::Payload(payload)) = response.output {
                                if let Some(crate::platform::ios::proto::idb::payload::Source::Data(data)) = payload.source {
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
}
