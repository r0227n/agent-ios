use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct TapRequest {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Serialize)]
pub struct LongPressRequest {
    pub x: f64,
    pub y: f64,
    pub duration: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwipeRequest {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub duration: f64,
}

#[derive(Debug, Serialize)]
pub struct TypeTextRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct KeyPressRequest {
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct ButtonPressRequest {
    pub button: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppRequest {
    pub bundle_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RunnerResponse {
    pub success: Option<bool>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InstallRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallRequest {
    pub bundle_id: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub runner: Option<String>,
}

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
