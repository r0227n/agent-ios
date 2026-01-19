//! gRPC client for iOS companion communication.
//!
//! This module provides the IdbClient which handles all gRPC communication
//! with the idb_companion daemon.

use crate::types::Address;
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::time::sleep;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use crate::platform::ios::proto::idb::companion_service_client::CompanionServiceClient;

/// Configuration for launching an application
pub struct LaunchConfig {
    pub bundle_id: String,
    pub app_args: Vec<String>,
    pub env: HashMap<String, String>,
    pub foreground_if_running: bool,
    pub wait_for_debugger: bool,
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

pub struct IdbClient {
    pub(crate) client: CompanionServiceClient<Channel>,
    pub(crate) address: Address,
}

impl IdbClient {
    /// 接続タイムアウト（秒）
    const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
    /// リクエストタイムアウト（秒）
    const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
    /// 最大リトライ回数
    const MAX_RETRIES: u32 = 3;

    /// Connect to idb_companion via Unix Domain Socket (with retry)
    pub async fn connect_uds(
        socket_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error = None;

        for attempt in 0..Self::MAX_RETRIES {
            if attempt > 0 {
                // 指数バックオフ: 1秒, 2秒, 4秒...
                let delay = Duration::from_secs(1 << (attempt - 1));
                sleep(delay).await;
            }

            match Self::try_connect_uds(socket_path).await {
                Ok(client) => return Ok(client),
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "Connection failed".into()))
    }

    /// UDS接続の実際の試行
    async fn try_connect_uds(
        socket_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let socket_path_owned = socket_path.to_string();

        let endpoint = Endpoint::try_from("http://[::]:50051")?
            .connect_timeout(Self::DEFAULT_CONNECT_TIMEOUT)
            .timeout(Self::DEFAULT_REQUEST_TIMEOUT);

        let channel = endpoint
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

    /// Connect to idb_companion via TCP (with retry)
    pub async fn connect_tcp(
        host: &str,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error = None;

        for attempt in 0..Self::MAX_RETRIES {
            if attempt > 0 {
                // 指数バックオフ: 1秒, 2秒, 4秒...
                let delay = Duration::from_secs(1 << (attempt - 1));
                sleep(delay).await;
            }

            match Self::try_connect_tcp(host, port).await {
                Ok(client) => return Ok(client),
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "Connection failed".into()))
    }

    /// TCP接続の実際の試行
    async fn try_connect_tcp(
        host: &str,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("http://{}:{}", host, port);
        let channel = Channel::from_shared(addr)?
            .connect_timeout(Self::DEFAULT_CONNECT_TIMEOUT)
            .timeout(Self::DEFAULT_REQUEST_TIMEOUT)
            .connect()
            .await?;
        let client = CompanionServiceClient::new(channel);

        Ok(Self {
            client,
            address: Address::Tcp {
                host: host.to_string(),
                port,
            },
        })
    }
}

/// Extract trace files from tar payload data
pub(crate) fn extract_trace_files(
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
pub(crate) async fn read_dap_message<R: tokio::io::AsyncBufRead + tokio::io::AsyncRead + Unpin>(
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
