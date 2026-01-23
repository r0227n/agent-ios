//! Common error types for agent-mobile.
//!
//! This module defines a unified error type that can be used across
//! all components of the codebase.

use std::fmt;

/// Common result type alias using the Error enum
pub type Result<T> = std::result::Result<T, Error>;

/// Common error type for agent-mobile operations
#[derive(Debug)]
pub enum Error {
    /// gRPC communication error
    Grpc(String),
    /// Connection error (failed to connect to companion)
    Connection(String),
    /// Companion not found or not available
    CompanionNotFound(String),
    /// Target (device/simulator) not found
    TargetNotFound(String),
    /// Multiple targets found when one was expected
    MultipleTargets(String),
    /// File operation error
    FileOperation(String),
    /// Installation error
    Installation(String),
    /// Invalid argument or parameter
    InvalidArgument(String),
    /// IO error
    Io(std::io::Error),
    /// Generic error with message
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Grpc(msg) => write!(f, "gRPC error: {}", msg),
            Error::Connection(msg) => write!(f, "Connection error: {}", msg),
            Error::CompanionNotFound(msg) => write!(f, "Companion not found: {}", msg),
            Error::TargetNotFound(msg) => write!(f, "Target not found: {}", msg),
            Error::MultipleTargets(msg) => write!(f, "Multiple targets found: {}", msg),
            Error::FileOperation(msg) => write!(f, "File operation error: {}", msg),
            Error::Installation(msg) => write!(f, "Installation error: {}", msg),
            Error::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
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

// tonic error conversions (only when tonic feature is enabled)
#[cfg(feature = "tonic")]
impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Self {
        Error::Grpc(status.message().to_string())
    }
}

#[cfg(feature = "tonic")]
impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        Error::Connection(err.to_string())
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
