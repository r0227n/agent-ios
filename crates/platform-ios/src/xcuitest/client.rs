use std::time::Duration;

use super::types::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// HTTP client for the XCUITest Runner.
/// Communicates with the Swift-based XCUITest server running on the simulator.
pub struct XCUITestClient {
    base_url: String,
    /// General-purpose HTTP client (30 s timeout).
    http: reqwest::Client,
    /// HTTP client with a long timeout for full accessibility tree traversal.
    http_long: reqwest::Client,
}

impl XCUITestClient {
    /// Default port for the XCUITest Runner HTTP server.
    pub const DEFAULT_PORT: u16 = 8200;

    /// Create a new client connected to localhost on the given port.
    pub fn new(port: u16) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        let http_long = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create long-timeout HTTP client");

        Self {
            base_url: format!("http://localhost:{}", port),
            http,
            http_long,
        }
    }

    /// Create a client with the default port.
    pub fn default() -> Self {
        Self::new(Self::DEFAULT_PORT)
    }

    /// Check if the runner is healthy and accepting connections.
    pub async fn health_check(&self) -> Result<bool> {
        let resp = self
            .http
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;
        let health: HealthResponse = resp.json().await?;
        Ok(health.status == "ok")
    }

    /// Wait for the runner to become available (with retries).
    pub async fn wait_for_ready(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        let retry_interval = Duration::from_millis(500);

        loop {
            match self.health_check().await {
                Ok(true) => return Ok(()),
                _ => {
                    if start.elapsed() > timeout {
                        return Err("XCUITest Runner did not become ready in time".into());
                    }
                    tokio::time::sleep(retry_interval).await;
                }
            }
        }
    }

    /// Wait for the runner to become available without any timeout.
    ///
    /// Polls the health endpoint every 500ms indefinitely, printing a progress
    /// message every 10 seconds so the user knows the process is still waiting.
    pub async fn wait_for_ready_indefinitely(&self) -> Result<()> {
        let start = std::time::Instant::now();
        let retry_interval = Duration::from_millis(500);
        let progress_interval = Duration::from_secs(10);
        let mut last_progress = std::time::Instant::now();

        loop {
            match self.health_check().await {
                Ok(true) => return Ok(()),
                _ => {
                    if last_progress.elapsed() >= progress_interval {
                        let elapsed = start.elapsed().as_secs();
                        eprintln!("Still waiting for XCUITest Runner ({elapsed}s elapsed)...");
                        last_progress = std::time::Instant::now();
                    }
                    tokio::time::sleep(retry_interval).await;
                }
            }
        }
    }

    /// Tap at screen coordinates.
    pub async fn tap(&self, x: f64, y: f64) -> Result<()> {
        self.post("/tap", &TapRequest { x, y }).await
    }

    /// Long press at screen coordinates.
    pub async fn long_press(&self, x: f64, y: f64, duration: f64) -> Result<()> {
        self.post("/longpress", &LongPressRequest { x, y, duration })
            .await
    }

    /// Swipe from one point to another.
    pub async fn swipe(&self, start: (f64, f64), end: (f64, f64), duration: f64) -> Result<()> {
        self.post(
            "/swipe",
            &SwipeRequest {
                start_x: start.0,
                start_y: start.1,
                end_x: end.0,
                end_y: end.1,
                duration,
            },
        )
        .await
    }

    /// Type text on the focused element.
    pub async fn type_text(&self, text: &str) -> Result<()> {
        self.post(
            "/type",
            &TypeTextRequest {
                text: text.to_string(),
            },
        )
        .await
    }

    /// Press a keyboard key by name (enter, delete, escape, etc.).
    pub async fn key_press(&self, key: &str) -> Result<()> {
        self.post(
            "/keypress",
            &KeyPressRequest {
                key: key.to_string(),
            },
        )
        .await
    }

    /// Clear text in the focused element (Select All + Delete).
    pub async fn clear_text(&self) -> Result<()> {
        let url = format!("{}/clear-text", self.base_url);
        let resp = self
            .http
            .post(&url)
            .json(&serde_json::json!({}))
            .send()
            .await?;
        let status = resp.status();
        let response: RunnerResponse = resp.json().await?;
        if !status.is_success() {
            let err_msg = response.error.unwrap_or_else(|| format!("HTTP {}", status));
            return Err(err_msg.into());
        }
        if let Some(err) = response.error {
            return Err(err.into());
        }
        Ok(())
    }

    /// Press a hardware button (home, etc.).
    pub async fn button_press(&self, button: &str) -> Result<()> {
        self.post(
            "/button",
            &ButtonPressRequest {
                button: button.to_string(),
            },
        )
        .await
    }

    /// Get the accessibility tree as JSON string (idb-compatible format).
    pub async fn accessibility_info(&self, nested: bool) -> Result<String> {
        self.accessibility_info_with_depth(nested, None).await
    }

    /// Get the accessibility tree with an optional depth limit.
    ///
    /// When `max_depth` is `Some(n)`, only `n` levels of children are returned.
    /// This is useful for change-detection where the full tree is unnecessary.
    pub async fn accessibility_info_with_depth(
        &self,
        nested: bool,
        max_depth: Option<u32>,
    ) -> Result<String> {
        let mut url = format!(
            "{}/accessibility?nested={}",
            self.base_url,
            if nested { "true" } else { "false" }
        );
        if let Some(depth) = max_depth {
            url.push_str(&format!("&depth={}", depth));
        }
        // Shallow queries use the standard client (30 s); full traversal needs the long client (120 s).
        let client = if max_depth.is_some() {
            &self.http
        } else {
            &self.http_long
        };
        let resp = client.get(&url).send().await?;
        let body = resp.text().await?;
        Ok(body)
    }

    /// Launch an app by bundle ID.
    pub async fn launch_app(&self, bundle_id: &str) -> Result<()> {
        self.post(
            "/launch",
            &AppRequest {
                bundle_id: bundle_id.to_string(),
            },
        )
        .await
    }

    /// Terminate an app by bundle ID.
    pub async fn terminate_app(&self, bundle_id: &str) -> Result<()> {
        self.post(
            "/terminate",
            &AppRequest {
                bundle_id: bundle_id.to_string(),
            },
        )
        .await
    }

    /// Install an app on the simulator.
    pub async fn install_app(&self, path: &str) -> Result<()> {
        self.post(
            "/install",
            &InstallRequest {
                path: path.to_string(),
            },
        )
        .await
    }

    /// Uninstall an app from the simulator.
    pub async fn uninstall_app(&self, bundle_id: &str) -> Result<()> {
        self.post(
            "/uninstall",
            &UninstallRequest {
                bundle_id: bundle_id.to_string(),
            },
        )
        .await
    }

    /// List installed apps on the simulator.
    pub async fn list_apps(&self) -> Result<Vec<AppInfo>> {
        let url = format!("{}/list-apps", self.base_url);
        let resp = self.http.get(&url).send().await?;
        let body: ListAppsResponse = resp.json().await?;

        // Parse the apps array from the response
        match body.apps {
            serde_json::Value::Array(arr) => {
                let mut apps = Vec::new();
                for item in arr {
                    if let Ok(app) = serde_json::from_value::<AppInfo>(item) {
                        apps.push(app);
                    }
                }
                Ok(apps)
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Capture a screenshot and return PNG bytes.
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        let url = format!("{}/screenshot", self.base_url);
        let resp = self.http.get(&url).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Screenshot failed: HTTP {} - {}", status, body).into());
        }
        Ok(resp.bytes().await?.to_vec())
    }

    /// Set the active app context (for accessibility queries) without launching.
    pub async fn set_app(&self, bundle_id: &str) -> Result<()> {
        self.post(
            "/set-app",
            &AppRequest {
                bundle_id: bundle_id.to_string(),
            },
        )
        .await
    }

    /// Copy text to the device clipboard.
    pub async fn clipboard_copy(&self, text: &str) -> Result<()> {
        self.post(
            "/clipboard/copy",
            &ClipboardCopyRequest {
                text: text.to_string(),
            },
        )
        .await
    }

    /// Paste text from the device clipboard.
    pub async fn clipboard_paste(&self) -> Result<String> {
        let url = format!("{}/clipboard/paste", self.base_url);
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        let body: ClipboardPasteResponse = resp.json().await?;
        if !status.is_success() {
            return Err(format!("Clipboard paste failed: HTTP {}", status).into());
        }
        Ok(body.text)
    }

    /// Clear the device clipboard.
    pub async fn clipboard_clear(&self) -> Result<()> {
        let url = format!("{}/clipboard/clear", self.base_url);
        let resp = self
            .http
            .post(&url)
            .json(&serde_json::json!({}))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("Clipboard clear failed: HTTP {}", status).into());
        }
        Ok(())
    }

    /// Internal: send a POST request and check for success.
    async fn post<T: serde::Serialize>(&self, path: &str, body: &T) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.http.post(&url).json(body).send().await?;

        let status = resp.status();
        let response: RunnerResponse = resp.json().await?;

        if !status.is_success() {
            let err_msg = response.error.unwrap_or_else(|| format!("HTTP {}", status));
            return Err(err_msg.into());
        }

        if let Some(err) = response.error {
            return Err(err.into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = XCUITestClient::new(8200);
        assert_eq!(client.base_url, "http://localhost:8200");
    }

    #[test]
    fn test_default_port() {
        assert_eq!(XCUITestClient::DEFAULT_PORT, 8200);
    }
}
