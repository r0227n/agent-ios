//! Platform-specific implementations for mobile device communication.
//!
//! This module contains platform-specific code for communicating with
//! mobile devices. Currently supports iOS, with Android support planned.

pub mod ios;

// Future: Android support
// pub mod android;

use async_trait::async_trait;
use serde::Serialize;

/// Suggested device information for AI agents.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceSuggestion {
    pub name: String,
    pub udid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub platform: String,
    pub score: f64,
    pub reasons: Vec<String>,
}

/// Device suggester trait (common interface for iOS/Android).
#[async_trait]
pub trait DeviceSuggester: Send + Sync {
    /// Get suggested devices.
    async fn suggest(
        &self,
        count: usize,
    ) -> Result<Vec<DeviceSuggestion>, Box<dyn std::error::Error + Send + Sync>>;

    /// Return the platform name.
    fn platform_name(&self) -> &'static str;
}
