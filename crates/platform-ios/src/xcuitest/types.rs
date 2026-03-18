use serde::{Deserialize, Serialize};

use agent_mobile_core::snapshot::Frame;

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
    /// Optional simulator UDID hosting the runner.
    pub udid: Option<String>,
    /// Optional active app bundle identifier for the current runner context.
    pub active_bundle_id: Option<String>,
    /// Monotonic generation that advances after UI-affecting commands.
    pub snapshot_generation: Option<u64>,
}

/// Query request for the fast snapshot/query endpoints.
#[derive(Debug, Serialize)]
pub struct QueryRequest {
    /// Locator type: text, label, placeholder, type, or element_id.
    pub locator: String,
    /// Locator value to match.
    pub value: String,
    /// Whether the match must be exact.
    pub exact: bool,
    /// Whether the comparison should be case-sensitive.
    #[serde(rename = "caseSensitive")]
    pub case_sensitive: bool,
    /// Restrict results to visible elements.
    #[serde(rename = "visibleOnly")]
    pub visible_only: bool,
    /// Optional maximum depth when the runner uses depth-bounded traversal.
    #[serde(rename = "maxDepth", skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
}

/// Snapshot element payload returned by the fast runner endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct RunnerSnapshotElement {
    /// Stable-ish runner-side element identifier.
    pub element_id: String,
    /// Platform-normalized element type.
    #[serde(rename = "type")]
    pub element_type: String,
    /// Accessibility label, when available.
    pub label: Option<String>,
    /// Current value, when available.
    pub value: Option<String>,
    /// Placeholder text for editable elements.
    pub placeholder: Option<String>,
    /// Element frame in screen coordinates.
    pub frame: Frame,
    /// Whether the element is enabled.
    pub enabled: bool,
    /// Whether the runner considers the element interactive.
    pub interactive: bool,
    /// Optional exact depth from bounded traversal.
    pub depth: Option<u32>,
}

/// Response for the fast snapshot endpoint.
#[derive(Debug, Deserialize)]
pub struct RunnerSnapshotResponse {
    /// Snapshot identifier assigned by the runner.
    pub snapshot_id: String,
    /// Active bundle identifier for the snapshot context.
    pub active_bundle_id: Option<String>,
    /// Runner generation at capture time.
    pub snapshot_generation: u64,
    /// Flat element payloads.
    pub elements: Vec<RunnerSnapshotElement>,
}

/// Response for query/first.
#[derive(Debug, Deserialize)]
pub struct QueryFirstResponse {
    /// Whether a matching element was found.
    pub found: bool,
    /// Active bundle identifier for the current context.
    pub active_bundle_id: Option<String>,
    /// Runner generation at query time.
    pub snapshot_generation: Option<u64>,
    /// Matching element payload, if any.
    pub element: Option<RunnerSnapshotElement>,
}

/// Response for query/exists.
#[derive(Debug, Deserialize)]
pub struct QueryExistsResponse {
    /// Whether a matching element currently exists.
    pub exists: bool,
    /// Active bundle identifier for the current context.
    pub active_bundle_id: Option<String>,
    /// Runner generation at query time.
    pub snapshot_generation: Option<u64>,
}

/// Response for ui-hash.
#[derive(Debug, Deserialize)]
pub struct UiHashResponse {
    /// Hash of the current UI state.
    pub hash: String,
    /// Hash source used by the runner.
    pub source: String,
    /// Active bundle identifier for the current context.
    pub active_bundle_id: Option<String>,
    /// Runner generation at hash time.
    pub snapshot_generation: Option<u64>,
}
