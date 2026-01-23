//! Companion spawning functionality.
//!
//! This module handles spawning new idb_companion daemon processes.

use agent_mobile_core::types::TargetType;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use thiserror::Error;

const IDB_DIR: &str = "/tmp/idb";

#[derive(Debug, Error)]
pub enum SpawnError {
    #[error("idb_companion not found in PATH")]
    CompanionNotFound,

    #[error("Failed to spawn companion: {0}")]
    SpawnFailed(String),

    #[error("Failed to parse companion output: {0}")]
    ParseError(String),

    #[error("Companion exited unexpectedly: {0}")]
    ProcessExited(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Configuration for spawning a companion
pub struct CompanionSpawnConfig {
    pub udid: String,
    pub target_type: TargetType,
}

/// Response parsed from companion stdout
#[derive(Debug, Deserialize)]
pub struct CompanionSpawnResponse {
    pub grpc_path: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    pub grpc_port: Option<u16>,
}

/// Represents a successfully spawned companion
#[derive(Debug)]
pub struct SpawnedCompanion {
    pub grpc_path: String,
    pub pid: u32,
    pub udid: String,
}

/// Spawns and manages idb_companion processes
pub struct CompanionSpawner {
    companion_path: String,
}

impl CompanionSpawner {
    /// Create a new spawner, finding idb_companion in PATH or common locations
    pub fn new() -> Result<Self, SpawnError> {
        // Check IDB_COMPANION environment variable first
        if let Ok(path) = std::env::var("IDB_COMPANION") {
            if std::path::Path::new(&path).exists() {
                return Ok(Self {
                    companion_path: path,
                });
            }
        }

        // Try to find idb_companion using `which`
        if let Ok(output) = Command::new("which").arg("idb_companion").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Self {
                        companion_path: path,
                    });
                }
            }
        }

        // Check common installation locations
        let common_paths = [
            "/usr/local/bin/idb_companion",
            "/opt/homebrew/bin/idb_companion",
        ];

        for path in &common_paths {
            if std::path::Path::new(path).exists() {
                return Ok(Self {
                    companion_path: path.to_string(),
                });
            }
        }

        Err(SpawnError::CompanionNotFound)
    }

    /// Create spawner with explicit path to idb_companion
    #[allow(dead_code)]
    pub fn with_path(companion_path: String) -> Self {
        Self { companion_path }
    }

    /// Check if a domain socket exists (might be stale)
    pub fn socket_exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    /// Generate socket path for a UDID
    pub fn socket_path_for_udid(udid: &str) -> String {
        format!("{}/{}_companion.sock", IDB_DIR, udid)
    }

    /// Spawn a companion for a UDID with domain socket
    pub fn spawn_domain_socket(
        &self,
        config: CompanionSpawnConfig,
    ) -> Result<SpawnedCompanion, SpawnError> {
        let socket_path = Self::socket_path_for_udid(&config.udid);

        // Ensure /tmp/idb directory exists
        std::fs::create_dir_all(IDB_DIR)?;

        // Remove stale socket if it exists
        if Self::socket_exists(&socket_path) {
            // Try to connect to check if companion is alive
            // If connection fails, socket is stale - remove it
            if std::os::unix::net::UnixStream::connect(&socket_path).is_err() {
                let _ = std::fs::remove_file(&socket_path);
            } else {
                // Socket is active, return it as if spawned
                return Ok(SpawnedCompanion {
                    grpc_path: socket_path,
                    pid: 0, // Unknown PID for existing companion
                    udid: config.udid,
                });
            }
        }

        // Build spawn command
        let mut cmd = Command::new(&self.companion_path);
        cmd.arg("--udid")
            .arg(&config.udid)
            .arg("--grpc-domain-sock")
            .arg(&socket_path);

        // Add --only flag based on target type
        match config.target_type {
            TargetType::Simulator => {
                cmd.arg("--only").arg("simulator");
            }
            TargetType::Device => {
                cmd.arg("--only").arg("device");
            }
            TargetType::Mac => {
                // No --only flag for mac
            }
        }

        // Configure process to become a daemon
        unsafe {
            cmd.pre_exec(|| {
                // Create new session to detach from terminal
                nix::unistd::setsid().map_err(std::io::Error::other)?;
                Ok(())
            });
        }

        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        // Spawn the process
        let mut child = cmd
            .spawn()
            .map_err(|e| SpawnError::SpawnFailed(e.to_string()))?;

        let pid = child.id();

        // Read JSON response from stdout
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| SpawnError::SpawnFailed("Failed to capture stdout".to_string()))?;

        let reader = BufReader::new(stdout);
        let mut grpc_path_from_response: Option<String> = None;

        for line in reader.lines() {
            let line = line.map_err(|e| SpawnError::ParseError(e.to_string()))?;
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse as JSON
            if let Ok(response) = serde_json::from_str::<CompanionSpawnResponse>(&line) {
                if let Some(path) = response.grpc_path {
                    grpc_path_from_response = Some(path);
                    break;
                }
            }
        }

        // Use response path or default to expected path
        let final_path = grpc_path_from_response.unwrap_or_else(|| socket_path.clone());

        // Wait briefly to ensure socket is ready
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Verify socket was created
        if !Self::socket_exists(&final_path) {
            // Check if process is still running
            match child.try_wait() {
                Ok(Some(status)) => {
                    return Err(SpawnError::ProcessExited(format!(
                        "Companion exited with status: {}",
                        status
                    )));
                }
                Ok(None) => {
                    // Process still running but socket not created yet
                    // Wait a bit more
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if !Self::socket_exists(&final_path) {
                        return Err(SpawnError::SpawnFailed(
                            "Socket not created after waiting".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    return Err(SpawnError::SpawnFailed(format!(
                        "Failed to check process status: {}",
                        e
                    )));
                }
            }
        }

        Ok(SpawnedCompanion {
            grpc_path: final_path,
            pid,
            udid: config.udid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_for_udid() {
        let path = CompanionSpawner::socket_path_for_udid("ABC123");
        assert_eq!(path, "/tmp/idb/ABC123_companion.sock");
    }

    #[test]
    fn test_parse_companion_response() {
        let json = r#"{"grpc_path": "/tmp/idb/test.sock"}"#;
        let response: CompanionSpawnResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.grpc_path, Some("/tmp/idb/test.sock".to_string()));
    }

    #[test]
    fn test_parse_companion_response_with_port() {
        let json = r#"{"grpc_path": "/tmp/idb/test.sock", "grpc_port": 9888}"#;
        let response: CompanionSpawnResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.grpc_path, Some("/tmp/idb/test.sock".to_string()));
        assert_eq!(response.grpc_port, Some(9888));
    }
}
