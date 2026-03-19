//! Shared iOS command helpers.

use agent_mobile_platform_ios::xcuitest::XCUITestClient;

use crate::helpers::client::{with_xcuitest, CommandResult};

/// Parse the root accessibility frame into screen dimensions.
fn screen_size_from_accessibility_json(json: &serde_json::Value) -> Option<(f64, f64)> {
    let frame = json.get("frame")?;
    let width = frame.get("width")?.as_f64()?;
    let height = frame.get("height")?.as_f64()?;

    if width > 0.0 && height > 0.0 {
        Some((width, height))
    } else {
        None
    }
}

/// Get iOS screen dimensions from a live XCUITest client.
pub async fn get_ios_screen_size_from_client(client: &XCUITestClient) -> CommandResult<(f64, f64)> {
    let json_str = client.accessibility_info(false).await?;
    let json: serde_json::Value = serde_json::from_str(&json_str)?;

    screen_size_from_accessibility_json(&json)
        .ok_or_else(|| "Could not determine screen size from accessibility info".into())
}

/// Get iOS screen dimensions for the selected simulator.
pub async fn get_ios_screen_size(udid: Option<&str>) -> CommandResult<(f64, f64)> {
    with_xcuitest(udid, |client| async move {
        get_ios_screen_size_from_client(&client).await
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_size_from_accessibility_json_reads_root_frame() {
        let json = serde_json::json!({
            "frame": {
                "x": 0.0,
                "y": 0.0,
                "width": 430.0,
                "height": 932.0
            }
        });

        assert_eq!(
            screen_size_from_accessibility_json(&json),
            Some((430.0, 932.0))
        );
    }

    #[test]
    fn test_screen_size_from_accessibility_json_rejects_zero_size() {
        let json = serde_json::json!({
            "frame": {
                "width": 0.0,
                "height": 932.0
            }
        });

        assert_eq!(screen_size_from_accessibility_json(&json), None);
    }

    #[test]
    fn test_screen_size_from_accessibility_json_missing_frame() {
        let json = serde_json::json!({
            "type": "Application"
        });

        assert_eq!(screen_size_from_accessibility_json(&json), None);
    }

    #[test]
    fn test_screen_size_from_accessibility_json_missing_dimensions() {
        let json = serde_json::json!({
            "frame": {
                "x": 0.0,
                "y": 0.0
            }
        });

        assert_eq!(screen_size_from_accessibility_json(&json), None);
    }
}
