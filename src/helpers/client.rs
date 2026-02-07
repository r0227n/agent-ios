//! Client connection helpers for CLI commands
//!
//! This module provides utilities for connecting to the XCUITest Runner,
//! reducing boilerplate across command implementations.

use agent_mobile_platform_ios::xcuitest::XCUITestClient;

/// Standard result type for CLI commands
pub type CommandResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Execute a command with an XCUITestClient connection.
///
/// This helper function handles the common pattern of:
/// 1. Ensuring the XCUITest Runner is started (auto-start if needed)
/// 2. Creating an XCUITestClient
/// 3. Executing the provided closure with the client
///
/// The Runner is started as a daemon process that persists after the CLI exits.
/// If the Runner is already running, the health check returns immediately
/// with minimal overhead.
///
/// # Example
///
/// ```ignore
/// use agent_mobile::helpers::client::{with_xcuitest, CommandResult};
///
/// pub async fn run() -> CommandResult {
///     with_xcuitest(|client| async move {
///         client.tap(100.0, 200.0).await?;
///         Ok(())
///     }).await
/// }
/// ```
pub async fn with_xcuitest<F, Fut, T>(f: F) -> CommandResult<T>
where
    F: FnOnce(XCUITestClient) -> Fut,
    Fut: std::future::Future<Output = CommandResult<T>>,
{
    use agent_mobile_platform_ios::xcuitest::ensure_runner_started;

    // Ensure the Runner is started (no-op if already running)
    ensure_runner_started(XCUITestClient::DEFAULT_PORT).await?;

    let client = XCUITestClient::default();
    f(client).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_type() {
        // Test that CommandResult can hold different types
        let _: CommandResult<()> = Ok(());
        let _: CommandResult<i32> = Ok(42);
        let _: CommandResult<String> = Ok("test".to_string());
    }
}
