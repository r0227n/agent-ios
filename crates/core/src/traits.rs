//! Platform abstraction traits for mobile device operations.
//!
//! This module defines common traits that can be implemented by different
//! platform backends (iOS, Android, etc.) to provide a unified interface
//! for device operations.

use crate::error::Result;
use async_trait::async_trait;

/// Information about an installed application
#[derive(Debug, Clone)]
pub struct AppInfo {
    pub bundle_id: String,
    pub name: String,
    pub install_type: Option<String>,
    pub architectures: Vec<String>,
    pub process_state: Option<String>,
    pub debuggable: Option<bool>,
}

/// Information about a file or directory
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
}

/// A point on the screen
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new point at the given coordinates.
    ///
    /// # Example
    ///
    /// ```
    /// use agent_mobile_core::traits::Point;
    ///
    /// let p = Point::new(10.0, 20.0);
    /// assert_eq!(p.x, 10.0);
    /// assert_eq!(p.y, 20.0);
    /// ```
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Common device operations trait.
///
/// This trait defines the core operations that can be performed on any
/// mobile device, regardless of platform (iOS or Android).
#[async_trait]
pub trait DeviceOperations: Send + Sync {
    /// Take a screenshot and return the image data
    async fn screenshot(&mut self) -> Result<Vec<u8>>;

    /// Tap at the specified coordinates
    async fn tap(&mut self, point: Point) -> Result<()>;

    /// Swipe from one point to another
    async fn swipe(&mut self, from: Point, to: Point, duration: Option<f64>) -> Result<()>;

    /// Input text
    async fn input_text(&mut self, text: &str) -> Result<()>;
}

/// Application management operations
#[async_trait]
pub trait AppOperations: Send + Sync {
    /// Install an application from the specified path
    async fn install(&mut self, path: &str) -> Result<String>;

    /// Launch an application by its bundle ID
    async fn launch(&mut self, bundle_id: &str) -> Result<u64>;

    /// Terminate a running application
    async fn terminate(&mut self, bundle_id: &str) -> Result<()>;

    /// Uninstall an application
    async fn uninstall(&mut self, bundle_id: &str) -> Result<()>;

    /// List installed applications
    async fn list_apps(&mut self) -> Result<Vec<AppInfo>>;
}

/// File system operations
#[async_trait]
pub trait FileOperations: Send + Sync {
    /// List files in a directory
    async fn list_files(&mut self, path: &str) -> Result<Vec<FileInfo>>;

    /// Pull a file from the device
    async fn pull(&mut self, remote_path: &str, local_path: &str) -> Result<()>;

    /// Push a file to the device
    async fn push(&mut self, local_path: &str, remote_path: &str) -> Result<()>;

    /// Remove a file or directory
    async fn remove(&mut self, path: &str) -> Result<()>;

    /// Create a directory
    async fn mkdir(&mut self, path: &str) -> Result<()>;
}

/// Combined trait for full device capabilities
///
/// A type implementing this trait has access to all standard device operations.
pub trait MobileDevice: DeviceOperations + AppOperations + FileOperations {}

// Blanket implementation: any type implementing all three traits is a MobileDevice
impl<T> MobileDevice for T where T: DeviceOperations + AppOperations + FileOperations {}
