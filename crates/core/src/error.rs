//! Common error types for agent-mobile.
//!
//! Unified, structured error type with context for the codebase.

use thiserror::Error;

/// Common result type alias using the Error enum
pub type Result<T> = std::result::Result<T, Error>;

/// Common error type for agent-mobile operations
#[derive(Debug, Error)]
pub enum Error {
    /// gRPC communication error
    #[error("gRPC error: {0}")]
    Grpc(String),
    /// Connection error (failed to connect to companion)
    #[error("Connection error: {0}")]
    Connection(String),
    /// Companion not found or not available
    #[error("Companion not found: {0}")]
    CompanionNotFound(String),
    /// Target (device/simulator) not found
    #[error("Target not found: {0}")]
    TargetNotFound(String),
    /// Multiple targets found when one was expected
    #[error("Multiple targets found: {0}")]
    MultipleTargets(String),
    /// File operation error
    #[error("File operation error: {0}")]
    FileOperation(String),
    /// Installation error
    #[error("Installation error: {0}")]
    Installation(String),
    /// Invalid argument or parameter
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    /// IO error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Other(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Other(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Grpc("connection refused".to_string());
        assert_eq!(format!("{}", err), "gRPC error: connection refused");
    }

    #[test]
    fn test_error_from_string() {
        let err: Error = "test error".into();
        assert!(matches!(err, Error::Other(_)));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}
