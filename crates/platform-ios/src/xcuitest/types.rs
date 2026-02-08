use serde::{Deserialize, Serialize};

/// Request for a tap gesture
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TapRequest {
    pub x: f64,
    pub y: f64,
}

/// Request for a long press gesture
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LongPressRequest {
    pub x: f64,
    pub y: f64,
    pub duration: f64,
}

/// Request for a swipe gesture
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwipeRequest {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub duration: f64,
}

/// Request for typing text input
#[derive(Debug, Serialize)]
pub struct TypeTextRequest {
    pub text: String,
}

/// Request for a key press
#[derive(Debug, Serialize)]
pub struct KeyPressRequest {
    pub key: String,
}

/// Request for a hardware button press
#[derive(Debug, Serialize)]
pub struct ButtonPressRequest {
    pub button: String,
}

/// Request for an app operation (specified by bundle ID)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppRequest {
    pub bundle_id: String,
}

/// Request for app installation
#[derive(Debug, Serialize)]
pub struct InstallRequest {
    pub path: String,
}

/// Request for app uninstallation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallRequest {
    pub bundle_id: String,
}

/// Request to copy text to the clipboard
#[derive(Debug, Serialize)]
pub struct ClipboardCopyRequest {
    pub text: String,
}

/// Generic response from XCUITest Runner
#[derive(Debug, Deserialize)]
pub struct RunnerResponse {
    pub success: Option<bool>,
    pub error: Option<String>,
}

/// Response containing clipboard paste content
#[derive(Debug, Deserialize)]
pub struct ClipboardPasteResponse {
    pub text: String,
}

/// Response for health check endpoint
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub runner: Option<String>,
}

/// Response containing list of installed apps
#[derive(Debug, Deserialize)]
pub struct ListAppsResponse {
    pub apps: serde_json::Value,
}

/// Simplified app info returned from list-apps endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub bundle_id: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
}
