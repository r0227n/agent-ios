//! Error types for CoreSimulator operations.

/// Errors that can occur during CoreSimulator framework operations.
#[derive(Debug, thiserror::Error)]
pub enum CoreSimError {
    #[error("CoreSimulator framework not found")]
    /// The CoreSimulator framework could not be located on disk.
    FrameworkNotFound,
    #[error("No booted simulator found")]
    /// No simulator is currently booted.
    NoBootedDevice,
    #[error("Device with UDID '{0}' not found")]
    /// The requested simulator UDID does not exist in the current device set.
    DeviceNotFound(String),
    #[error("App install failed: {0}")]
    /// Installing the application bundle failed.
    InstallFailed(String),
    #[error("App launch failed: {0}")]
    /// Launching the application failed.
    LaunchFailed(String),
    #[error("App terminate failed: {0}")]
    /// Terminating the application failed.
    TerminateFailed(String),
    #[error("FFI error: {0}")]
    /// An Objective-C or framework interop call returned an error.
    FfiError(String),
}
