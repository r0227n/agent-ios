//! Error types for CoreSimulator operations.

/// Errors that can occur during CoreSimulator framework operations.
#[derive(Debug, thiserror::Error)]
pub enum CoreSimError {
    #[error("CoreSimulator framework not found")]
    FrameworkNotFound,
    #[error("No booted simulator found")]
    NoBootedDevice,
    #[error("Device with UDID '{0}' not found")]
    DeviceNotFound(String),
    #[error("App install failed: {0}")]
    InstallFailed(String),
    #[error("App launch failed: {0}")]
    LaunchFailed(String),
    #[error("App terminate failed: {0}")]
    TerminateFailed(String),
    #[error("FFI error: {0}")]
    FfiError(String),
}
