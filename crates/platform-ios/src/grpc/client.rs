//! gRPC client for iOS companion communication.
//!
//! This module provides the IdbClient which handles all gRPC communication
//! with the idb_companion daemon.

use agent_mobile_core::types::Address;
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::time::sleep;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use crate::proto::idb::companion_service_client::CompanionServiceClient;

/// Configuration for launching an application
pub struct LaunchConfig {
    pub bundle_id: String,
    pub app_args: Vec<String>,
    pub env: HashMap<String, String>,
    pub foreground_if_running: bool,
    pub wait_for_debugger: bool,
}

pub struct IdbClient {
    pub(crate) client: CompanionServiceClient<Channel>,
    pub(crate) address: Address,
}

impl IdbClient {
    /// Connection timeout (seconds)
    const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
    /// Request timeout (seconds)
    const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
    /// Maximum retry count
    const MAX_RETRIES: u32 = 3;

    /// Connect to idb_companion via Unix Domain Socket (with retry)
    pub async fn connect_uds(
        socket_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error = None;

        for attempt in 0..Self::MAX_RETRIES {
            if attempt > 0 {
                // Exponential backoff: 1s, 2s, 4s...
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

    /// Actual UDS connection attempt
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
                // Exponential backoff: 1s, 2s, 4s...
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

    /// Actual TCP connection attempt
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

    /// Connect to idb_companion via Unix Domain Socket for streaming (no request timeout)
    ///
    /// Use this method for long-running streaming RPCs (e.g., log, video)
    /// where the default 30-second timeout would cause transport errors.
    pub async fn connect_uds_streaming(
        socket_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error = None;

        for attempt in 0..Self::MAX_RETRIES {
            if attempt > 0 {
                let delay = Duration::from_secs(1 << (attempt - 1));
                sleep(delay).await;
            }

            match Self::try_connect_uds_streaming(socket_path).await {
                Ok(client) => return Ok(client),
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "Connection failed".into()))
    }

    /// Actual UDS connection attempt (for streaming, no timeout)
    async fn try_connect_uds_streaming(
        socket_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let socket_path_owned = socket_path.to_string();

        // For streaming: no request timeout
        let endpoint =
            Endpoint::try_from("http://[::]:50051")?.connect_timeout(Self::DEFAULT_CONNECT_TIMEOUT);

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

    /// Connect to idb_companion via TCP for streaming (no request timeout)
    ///
    /// Use this method for long-running streaming RPCs (e.g., log, video)
    /// where the default 30-second timeout would cause transport errors.
    pub async fn connect_tcp_streaming(
        host: &str,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error = None;

        for attempt in 0..Self::MAX_RETRIES {
            if attempt > 0 {
                let delay = Duration::from_secs(1 << (attempt - 1));
                sleep(delay).await;
            }

            match Self::try_connect_tcp_streaming(host, port).await {
                Ok(client) => return Ok(client),
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "Connection failed".into()))
    }

    /// Actual TCP connection attempt (for streaming, no timeout)
    async fn try_connect_tcp_streaming(
        host: &str,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("http://{}:{}", host, port);
        // For streaming: no request timeout
        let channel = Channel::from_shared(addr)?
            .connect_timeout(Self::DEFAULT_CONNECT_TIMEOUT)
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
