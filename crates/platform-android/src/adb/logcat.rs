//! ADB native protocol logcat streaming.
//!
//! Implements logcat streaming over the ADB wire protocol (TCP :5037),
//! eliminating the need for `adb logcat` CLI process.

use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::commands::{AdbError, Result};

/// Default ADB server address.
const ADB_ADDR: &str = "127.0.0.1:5037";

/// Connection timeout for ADB server.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Stream of logcat output over ADB native protocol.
pub struct LogcatStream {
    reader: Option<BufReader<TcpStream>>,
}

impl LogcatStream {
    /// Open a logcat stream to the device.
    ///
    /// Connects to ADB server via TCP, performs the ADB wire protocol
    /// handshake (transport selection + shell:logcat), then returns
    /// a line-oriented async stream.
    ///
    /// If `serial` is `None`, connects to whichever single device is available
    /// (`host:transport-any`).
    pub async fn open(serial: Option<&str>) -> Result<Self> {
        let mut stream = timeout(CONNECT_TIMEOUT, TcpStream::connect(ADB_ADDR))
            .await
            .map_err(|_| {
                AdbError::ConnectionError(format!(
                    "connection to ADB server at {} timed out",
                    ADB_ADDR
                ))
            })?
            .map_err(|e| {
                AdbError::ConnectionError(format!(
                    "failed to connect to ADB server at {}: {}",
                    ADB_ADDR, e
                ))
            })?;

        // 1. Select the target device
        let transport_cmd = match serial {
            Some(s) => format!("host:transport:{}", s),
            None => "host:transport-any".to_string(),
        };
        send_command(&mut stream, &transport_cmd).await?;
        read_okay(&mut stream).await?;

        // 2. Start logcat via shell
        send_command(&mut stream, "shell:logcat").await?;
        read_okay(&mut stream).await?;

        // 3. Wrap in a line-based reader for streaming
        let reader = BufReader::new(stream);

        Ok(Self {
            reader: Some(reader),
        })
    }

    /// Read the next line from the logcat stream.
    pub async fn next_line(&mut self) -> Result<Option<String>> {
        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| AdbError::ConnectionError("stream already closed".to_string()))?;

        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => Ok(None), // EOF
            Ok(_) => Ok(Some(line.trim_end().to_string())),
            Err(e) => Err(AdbError::ConnectionError(format!(
                "logcat read error: {}",
                e
            ))),
        }
    }

    /// Stop the logcat stream by shutting down the TCP connection.
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(reader) = self.reader.take() {
            let mut stream = reader.into_inner();
            stream
                .shutdown()
                .await
                .map_err(|e| AdbError::ConnectionError(format!("failed to shutdown: {}", e)))?;
        }
        Ok(())
    }
}

/// Send an ADB command with the 4-byte hex length prefix.
///
/// ADB wire protocol format: `{len:04X}{command}` where the prefix
/// is the ASCII hex length of the command body.
async fn send_command(stream: &mut TcpStream, command: &str) -> Result<()> {
    let msg = format!("{:04X}{}", command.len(), command);
    stream.write_all(msg.as_bytes()).await.map_err(|e| {
        AdbError::ConnectionError(format!("failed to send command '{}': {}", command, e))
    })
}

/// Read and verify an OKAY response from the ADB server.
///
/// ADB responds with "OKAY" on success, or "FAIL" followed by
/// a 4-byte hex length and an error message on failure.
async fn read_okay(stream: &mut TcpStream) -> Result<()> {
    let mut status = [0u8; 4];
    stream
        .read_exact(&mut status)
        .await
        .map_err(|e| AdbError::ConnectionError(format!("failed to read ADB response: {}", e)))?;

    match &status {
        b"OKAY" => Ok(()),
        b"FAIL" => {
            // Read error message length (4 hex digits)
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await.map_err(|e| {
                AdbError::ConnectionError(format!("failed to read error length: {}", e))
            })?;
            let len_str = std::str::from_utf8(&len_buf).unwrap_or("0000");
            let len = usize::from_str_radix(len_str, 16).unwrap_or(0);

            // Read error message
            let mut msg_buf = vec![0u8; len];
            stream.read_exact(&mut msg_buf).await.map_err(|e| {
                AdbError::ConnectionError(format!("failed to read error message: {}", e))
            })?;
            let msg = String::from_utf8_lossy(&msg_buf);
            Err(AdbError::ConnectionError(format!("ADB server: {}", msg)))
        }
        other => {
            let s = String::from_utf8_lossy(other);
            Err(AdbError::ConnectionError(format!(
                "unexpected ADB response: {}",
                s
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_adb_command_format() {
        let cmd = "host:transport-any";
        let msg = format!("{:04X}{}", cmd.len(), cmd);
        assert_eq!(msg, "0012host:transport-any");

        let cmd = "shell:logcat";
        let msg = format!("{:04X}{}", cmd.len(), cmd);
        assert_eq!(msg, "000Cshell:logcat");
    }

    #[test]
    fn test_transport_command_with_serial() {
        let serial = "emulator-5554";
        let cmd = format!("host:transport:{}", serial);
        assert_eq!(cmd, "host:transport:emulator-5554");
    }
}
