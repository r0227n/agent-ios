//! iOS device API for programmatic access.
//!
//! This module provides a simple, high-level API for controlling iOS devices.

use agent_mobile_core::Address;
use agent_mobile_platform_ios::companion::CompanionResolver;
use agent_mobile_platform_ios::grpc::{IdbClient, LaunchConfig};
use agent_mobile_platform_ios::hid::events::{swipe_to_events, tap_to_events, text_to_events};
use agent_mobile_platform_ios::simctl::management as simctl;
use agent_mobile_platform_ios::ImageFormat;
use std::collections::HashMap;
use tokio::sync::watch;

/// Result type for iOS API operations.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// High-level iOS device API.
///
/// This struct wraps the low-level gRPC client and provides
/// a simplified interface for common mobile automation tasks.
pub struct IosDevice {
    client: IdbClient,
    udid: String,
}

impl IosDevice {
    /// Connect to an iOS device.
    ///
    /// If `udid` is provided, connects to that specific device.
    /// Otherwise, auto-selects an available device.
    ///
    /// # Arguments
    ///
    /// * `udid` - Optional device UDID to connect to
    ///
    /// # Returns
    ///
    /// An `IosDevice` instance connected to the target device.
    pub async fn connect(udid: Option<&str>) -> Result<Self> {
        let resolver = CompanionResolver::new();
        let resolved = resolver.resolve(udid)?;
        let client = match &resolved.address {
            Address::DomainSocket { path } => IdbClient::connect_uds(path).await?,
            Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await?,
        };
        Ok(Self {
            client,
            udid: resolved.udid,
        })
    }

    /// Take a screenshot of the device screen.
    ///
    /// # Returns
    ///
    /// PNG image data as bytes.
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        let udid = self.udid.clone();
        let data = tokio::task::spawn_blocking(move || {
            simctl::io_screenshot_bytes(&udid, ImageFormat::Png)
        })
        .await??;
        Ok(data)
    }

    /// Tap at screen coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    pub async fn tap(&mut self, x: f64, y: f64) -> Result<()> {
        self.client.hid(tap_to_events(x, y, None)).await
    }

    /// Long press at screen coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `duration` - Press duration in seconds
    pub async fn long_press(&mut self, x: f64, y: f64, duration: f64) -> Result<()> {
        self.client.hid(tap_to_events(x, y, Some(duration))).await
    }

    /// Swipe from one point to another.
    ///
    /// # Arguments
    ///
    /// * `start` - Starting coordinates (x, y)
    /// * `end` - Ending coordinates (x, y)
    /// * `duration` - Swipe duration in seconds (optional)
    pub async fn swipe(
        &mut self,
        start: (f64, f64),
        end: (f64, f64),
        duration: Option<f64>,
    ) -> Result<()> {
        self.client
            .hid(swipe_to_events(start, end, duration, None))
            .await
    }

    /// Type text on the device.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to type
    pub async fn type_text(&mut self, text: &str) -> Result<()> {
        let events = text_to_events(text)?;
        self.client.hid(events).await
    }

    /// Launch an application.
    ///
    /// # Arguments
    ///
    /// * `bundle_id` - Bundle ID of the app to launch
    pub async fn launch(&mut self, bundle_id: &str) -> Result<Option<u64>> {
        let config = LaunchConfig {
            bundle_id: bundle_id.to_string(),
            app_args: vec![],
            env: HashMap::new(),
            foreground_if_running: true,
            wait_for_debugger: false,
        };

        let (_stop_tx, stop_rx) = watch::channel(false);
        self.client.launch(config, false, stop_rx).await
    }

    /// Terminate an application.
    ///
    /// # Arguments
    ///
    /// * `bundle_id` - Bundle ID of the app to terminate
    pub async fn terminate(&mut self, bundle_id: &str) -> Result<()> {
        self.client.terminate(bundle_id).await
    }

    /// Install an application.
    ///
    /// # Arguments
    ///
    /// * `bundle_path` - Path to the .app or .ipa bundle
    ///
    /// # Returns
    ///
    /// The installed app's bundle ID.
    pub async fn install(&mut self, bundle_path: &str) -> Result<String> {
        use tokio_stream::StreamExt;

        let mut stream = self.client.install(bundle_path, false, false, None).await?;

        let mut bundle_id = String::new();
        while let Some(response) = stream.next().await {
            if let Ok(resp) = response {
                if !resp.name.is_empty() {
                    bundle_id = resp.name;
                }
            }
        }

        Ok(bundle_id)
    }

    /// Uninstall an application.
    ///
    /// # Arguments
    ///
    /// * `bundle_id` - Bundle ID of the app to uninstall
    pub async fn uninstall(&mut self, bundle_id: &str) -> Result<()> {
        self.client.uninstall(bundle_id).await
    }

    /// Get the accessibility tree for the current screen.
    ///
    /// # Arguments
    ///
    /// * `nested` - Whether to include nested elements
    ///
    /// # Returns
    ///
    /// JSON string containing the accessibility tree.
    pub async fn accessibility(&mut self, nested: bool) -> Result<String> {
        self.client.accessibility_info(None, nested).await
    }

    /// Get the underlying gRPC client for advanced operations.
    ///
    /// This provides access to the full set of gRPC methods
    /// for operations not covered by the high-level API.
    pub fn client(&mut self) -> &mut IdbClient {
        &mut self.client
    }
}
