<<<<<<< HEAD
//! Error types for CoreSimulator operations.

/// Errors that can occur during CoreSimulator framework operations.
||||||| parent of 8cef7438 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
=======
<<<<<<< HEAD
>>>>>>> 8cef7438 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
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
||||||| parent of 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
=======
#[derive(Debug, thiserror::Error)]
pub enum CoreSimError {
    #[error("CoreSimulator framework not found")]
    FrameworkNotFound,
    #[error("No booted simulator found")]
    NoBootedDevice,
    #[error("App install failed: {0}")]
    InstallFailed(String),
    #[error("App launch failed: {0}")]
    LaunchFailed(String),
    #[error("App terminate failed: {0}")]
    TerminateFailed(String),
    #[error("FFI error: {0}")]
    FfiError(String),
}
>>>>>>> 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
