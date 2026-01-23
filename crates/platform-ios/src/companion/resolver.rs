//! Companion resolution functionality.
//!
//! This module handles finding and connecting to the appropriate companion daemon.

use crate::grpc::IdbClient;
use agent_mobile_core::types::{Address, TargetType};

use super::spawner::{CompanionSpawnConfig, CompanionSpawner, SpawnError};
use super::state::{CompanionState, StoredCompanion};

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("No companion found for UDID: {0}")]
    NotFound(String),

    #[error("No companions available and no UDID specified")]
    NoCompanions,

    #[error("Multiple companions available, please specify --udid")]
    MultipleCompanions,

    #[error("Spawn error: {0}")]
    SpawnFailed(#[from] SpawnError),

    #[error("State file error: {0}")]
    StateError(#[from] std::io::Error),

    #[allow(dead_code)]
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Companion has no valid address")]
    NoAddress,
}

/// Represents a successfully resolved companion
#[derive(Debug)]
#[allow(dead_code)]
pub struct ResolvedCompanion {
    pub address: Address,
    pub udid: String,
    pub was_spawned: bool,
}

/// Unified companion resolution with optional auto-spawning
///
/// # Responsibilities
///
/// CompanionResolver handles three core tasks:
/// 1. **UDID Resolution**: Determines which companion to use based on priority:
///    - If a Session UDID is available (set via `apply_session_udid_option!` macro in main.rs)
///    - If an explicit UDID is provided via command-line
///    - If only one companion exists, auto-select it
///    - Otherwise, error asking user to specify UDID
///
/// 2. **Companion Spawning**: Auto-spawns the companion if not already running
///    - For unspecified UDID with single companion: auto-spawn and connect
///    - For explicit UDID: spawn if needed, then connect
///
/// 3. **Connection Management**: Establishes gRPC connection to the companion
///    - Uses `connect()` method to create IdbClient
///    - Handles connection errors appropriately
///
/// # UDID Resolution Priority
///
/// The priority order is enforced at the CLI layer (main.rs):
/// 1. Session UDID (highest priority) - Set via `apply_session_udid_option!` macro
/// 2. Explicit UDID (--udid flag or -u short form)
/// 3. Auto-detection (lowest priority) - Single companion or error
///
/// Note: SessionResolver handles converting session names to UDIDs.
/// This resolver only consumes the already-resolved UDID.
pub struct CompanionResolver {
    state: CompanionState,
    spawner: Option<CompanionSpawner>,
}

impl Default for CompanionResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl CompanionResolver {
    /// Create resolver with automatic spawning enabled
    pub fn new() -> Self {
        let spawner = CompanionSpawner::new().ok();
        Self {
            state: CompanionState::default(),
            spawner,
        }
    }

    /// Create resolver without spawning capability
    #[allow(dead_code)]
    pub fn without_spawning() -> Self {
        Self {
            state: CompanionState::default(),
            spawner: None,
        }
    }

    /// Determine if a UDID is a simulator, device, or mac
    /// Returns None if the UDID is not valid (not found in any known sources)
    fn determine_target_type(&self, udid: &str) -> Option<TargetType> {
        // Special case for "mac"
        if udid == "mac" {
            return Some(TargetType::Mac);
        }

        // Without simctl, we cannot validate simulator UDIDs
        // Return None to indicate we cannot verify this UDID
        None
    }

    /// Resolve companion, spawning if necessary
    pub fn resolve(&self, udid: Option<&str>) -> Result<ResolvedCompanion, ResolveError> {
        let companions = self.state.get_companions();
        let companion_map: HashMap<&str, &StoredCompanion> =
            companions.iter().map(|c| (c.udid.as_str(), c)).collect();

        match udid {
            // Case 1: UDID specified and companion exists
            Some(u) if companion_map.contains_key(u) => {
                let companion = companion_map[u];
                Ok(ResolvedCompanion {
                    address: companion.address().ok_or(ResolveError::NoAddress)?,
                    udid: u.to_string(),
                    was_spawned: false,
                })
            }

            // Case 2: UDID specified but no companion in state - try fallback methods
            Some(u) => {
                // Fallback 1: Try direct socket path
                // When idb_companion is started directly (not via Python idb), the state file
                // is not updated. Check if the socket file exists directly.
                let socket_path = format!("/tmp/idb/{}_companion.sock", u);
                if std::path::Path::new(&socket_path).exists() {
                    return Ok(ResolvedCompanion {
                        address: Address::DomainSocket { path: socket_path },
                        udid: u.to_string(),
                        was_spawned: false,
                    });
                }

                // Fallback 2: Try to spawn if UDID is valid (exists in simctl)
                // Only spawn for valid UDIDs to provide clear error messages
                if let Some(target_type) = self.determine_target_type(u) {
                    if let Some(spawner) = &self.spawner {
                        let spawned = spawner.spawn_domain_socket(CompanionSpawnConfig {
                            udid: u.to_string(),
                            target_type,
                        })?;

                        // Update state file with new companion
                        self.state.add_companion(StoredCompanion {
                            udid: spawned.udid.clone(),
                            is_local: true,
                            pid: Some(spawned.pid),
                            path: Some(spawned.grpc_path.clone()),
                            host: None,
                            port: None,
                        })?;

                        Ok(ResolvedCompanion {
                            address: Address::DomainSocket {
                                path: spawned.grpc_path,
                            },
                            udid: spawned.udid,
                            was_spawned: true,
                        })
                    } else {
                        // No spawner available but UDID is valid - report not found
                        Err(ResolveError::NotFound(u.to_string()))
                    }
                } else {
                    // UDID is not valid (not found in simctl) - report not found
                    Err(ResolveError::NotFound(u.to_string()))
                }
            }

            // Case 3: No UDID, single companion available
            None if companions.len() == 1 => {
                let companion = &companions[0];
                Ok(ResolvedCompanion {
                    address: companion.address().ok_or(ResolveError::NoAddress)?,
                    udid: companion.udid.clone(),
                    was_spawned: false,
                })
            }

            // Case 4: No UDID, no companions
            None if companions.is_empty() => Err(ResolveError::NoCompanions),

            // Case 5: No UDID, multiple companions
            None => Err(ResolveError::MultipleCompanions),
        }
    }

    /// Connect to the resolved companion
    pub async fn connect(
        &self,
        udid: Option<&str>,
    ) -> Result<IdbClient, Box<dyn std::error::Error + Send + Sync>> {
        let resolved = self.resolve(udid)?;

        match &resolved.address {
            Address::DomainSocket { path } => IdbClient::connect_uds(path).await,
            Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await,
        }
    }

    /// Connect to the resolved companion for streaming (no request timeout)
    ///
    /// Use this method for long-running streaming RPCs (e.g., log, video)
    /// where the default 30-second timeout would cause transport errors.
    pub async fn connect_streaming(
        &self,
        udid: Option<&str>,
    ) -> Result<IdbClient, Box<dyn std::error::Error + Send + Sync>> {
        let resolved = self.resolve(udid)?;

        match &resolved.address {
            Address::DomainSocket { path } => IdbClient::connect_uds_streaming(path).await,
            Address::Tcp { host, port } => IdbClient::connect_tcp_streaming(host, *port).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_resolver(state_file_path: &str) -> CompanionResolver {
        CompanionResolver {
            state: CompanionState::new(state_file_path),
            spawner: None, // No spawning in tests
        }
    }

    #[test]
    fn test_resolve_single_companion_no_udid() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[{{"udid": "ABC123", "is_local": true, "path": "/tmp/idb/abc.sock"}}]"#
        )
        .unwrap();

        let resolver = create_test_resolver(file.path().to_str().unwrap());
        let result = resolver.resolve(None);

        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.udid, "ABC123");
        assert!(!resolved.was_spawned);
    }

    #[test]
    fn test_resolve_with_udid() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[
                {{"udid": "ABC123", "is_local": true, "path": "/tmp/idb/abc.sock"}},
                {{"udid": "XYZ789", "is_local": true, "path": "/tmp/idb/xyz.sock"}}
            ]"#
        )
        .unwrap();

        let resolver = create_test_resolver(file.path().to_str().unwrap());
        let result = resolver.resolve(Some("XYZ789"));

        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.udid, "XYZ789");
    }

    #[test]
    fn test_resolve_no_companions() {
        let file = NamedTempFile::new().unwrap();
        let resolver = create_test_resolver(file.path().to_str().unwrap());

        let result = resolver.resolve(None);
        assert!(matches!(result, Err(ResolveError::NoCompanions)));
    }

    #[test]
    fn test_resolve_multiple_companions_no_udid() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[
                {{"udid": "ABC123", "is_local": true, "path": "/tmp/idb/abc.sock"}},
                {{"udid": "XYZ789", "is_local": true, "path": "/tmp/idb/xyz.sock"}}
            ]"#
        )
        .unwrap();

        let resolver = create_test_resolver(file.path().to_str().unwrap());
        let result = resolver.resolve(None);

        assert!(matches!(result, Err(ResolveError::MultipleCompanions)));
    }

    #[test]
    fn test_resolve_udid_not_found_no_spawner() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[{{"udid": "ABC123", "is_local": true, "path": "/tmp/idb/abc.sock"}}]"#
        )
        .unwrap();

        let resolver = create_test_resolver(file.path().to_str().unwrap());
        let result = resolver.resolve(Some("NOTEXIST"));

        assert!(matches!(result, Err(ResolveError::NotFound(_))));
    }
}
