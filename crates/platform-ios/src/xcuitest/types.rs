use serde::{Deserialize, Serialize};

/// Request for a tap gesture
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TapRequest {
    /// Horizontal tap coordinate.
    pub x: f64,
    /// Vertical tap coordinate.
    pub y: f64,
}

/// Request for a long press gesture
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LongPressRequest {
    /// Horizontal press coordinate.
    pub x: f64,
    /// Vertical press coordinate.
    pub y: f64,
    /// Press duration in seconds.
    pub duration: f64,
}

/// Request for a swipe gesture
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwipeRequest {
    /// Horizontal starting coordinate.
    pub start_x: f64,
    /// Vertical starting coordinate.
    pub start_y: f64,
    /// Horizontal ending coordinate.
    pub end_x: f64,
    /// Vertical ending coordinate.
    pub end_y: f64,
    /// Swipe duration in seconds.
    pub duration: f64,
}

/// Request for typing text input
#[derive(Debug, Serialize)]
pub struct TypeTextRequest {
    /// Text to type into the focused element.
    pub text: String,
}

/// Request for a key press
#[derive(Debug, Serialize)]
pub struct KeyPressRequest {
    /// Logical key name understood by the runner.
    pub key: String,
}

/// Request for a hardware button press
#[derive(Debug, Serialize)]
pub struct ButtonPressRequest {
    /// Hardware button name understood by the runner.
    pub button: String,
}

/// Request for an app operation (specified by bundle ID)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppRequest {
    /// Bundle identifier of the application to operate on.
    pub bundle_id: String,
}

/// Request for app installation
#[derive(Debug, Serialize)]
pub struct InstallRequest {
    /// Local filesystem path to the app bundle to install.
    pub path: String,
}

/// Request for app uninstallation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallRequest {
    /// Bundle identifier of the app to uninstall.
    pub bundle_id: String,
}

/// Request to copy text to the clipboard
#[derive(Debug, Serialize)]
pub struct ClipboardCopyRequest {
    /// Text to place on the simulator clipboard.
    pub text: String,
}

/// Generic response from XCUITest Runner
#[derive(Debug, Deserialize)]
pub struct RunnerResponse {
    /// Whether the request succeeded.
    pub success: Option<bool>,
    /// Human-readable error message when the request fails.
    pub error: Option<String>,
}

/// Response containing clipboard paste content
#[derive(Debug, Deserialize)]
pub struct ClipboardPasteResponse {
    /// Text currently stored in the clipboard.
    pub text: String,
}

/// Response for health check endpoint
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    /// Overall health status returned by the runner.
    pub status: String,
    /// Optional runner version or identifier string.
    pub runner: Option<String>,
}

/// Response containing list of installed apps
#[derive(Debug, Deserialize)]
pub struct ListAppsResponse {
    /// Raw list of applications returned by the runner.
    pub apps: serde_json::Value,
}

/// Simplified app info returned from list-apps endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// Bundle identifier of the installed application.
    pub bundle_id: Option<String>,
    /// Human-readable application name.
    pub name: Option<String>,
    /// Marketing version string.
    pub version: Option<String>,
}
