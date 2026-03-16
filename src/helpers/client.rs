//! Client connection helpers for CLI commands
//!
//! This module provides utilities for connecting to the XCUITest Runner,
//! reducing boilerplate across command implementations.

use agent_mobile_platform_ios::xcuitest::types::HealthResponse;
use agent_mobile_platform_ios::xcuitest::XCUITestClient;

/// Standard result type for CLI commands
pub type CommandResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Policy for restoring the persisted iOS app context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppContextPolicy {
    /// Leave the runner's current app context untouched.
    PreserveCurrent,
    /// Restore the saved app only when the runner has no active app context.
    RestoreIfUnset,
}

/// Prepare a runner client and optionally synchronize the app context.
pub async fn prepare_xcuitest_with_policy(
    target_udid: Option<&str>,
    policy: AppContextPolicy,
) -> CommandResult<(String, XCUITestClient, HealthResponse)> {
    use agent_mobile_platform_ios::xcuitest::ensure_runner_started;

    let resolved_udid = ensure_runner_started(XCUITestClient::DEFAULT_PORT, target_udid).await?;
    let client = XCUITestClient::default();
    let mut ready = client.ready_status().await?;

    if let Some(bundle_id) = crate::session::app_context::get_active_app(&resolved_udid)? {
        if should_restore_app_context(ready.active_bundle_id.as_deref(), policy) {
            client.set_app(&bundle_id).await?;
            ready = client.ready_status().await?;
        }
    }

    Ok((resolved_udid, client, ready))
}

/// Prepare a runner client without forcing any stored app context.
pub async fn prepare_xcuitest(
    target_udid: Option<&str>,
) -> CommandResult<(String, XCUITestClient, HealthResponse)> {
    prepare_xcuitest_with_policy(target_udid, AppContextPolicy::PreserveCurrent).await
}

fn should_restore_app_context(active_bundle_id: Option<&str>, policy: AppContextPolicy) -> bool {
    match policy {
        AppContextPolicy::PreserveCurrent => false,
        AppContextPolicy::RestoreIfUnset => active_bundle_id.is_none(),
    }
}

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
///     with_xcuitest(None, |client| async move {
///         client.tap(100.0, 200.0).await?;
///         Ok(())
///     }).await
/// }
/// ```
pub async fn with_xcuitest<F, Fut, T>(target_udid: Option<&str>, f: F) -> CommandResult<T>
where
    F: FnOnce(XCUITestClient) -> Fut,
    Fut: std::future::Future<Output = CommandResult<T>>,
{
    let (_, client, _) = prepare_xcuitest(target_udid).await?;
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

    #[test]
    fn test_should_restore_app_context_preserve_current() {
        assert!(!should_restore_app_context(
            None,
            AppContextPolicy::PreserveCurrent
        ));
        assert!(!should_restore_app_context(
            Some("com.apple.springboard"),
            AppContextPolicy::PreserveCurrent
        ));
    }

    #[test]
    fn test_should_restore_app_context_restore_if_unset() {
        assert!(should_restore_app_context(
            None,
            AppContextPolicy::RestoreIfUnset
        ));
        assert!(!should_restore_app_context(
            Some("com.example.app"),
            AppContextPolicy::RestoreIfUnset
        ));
    }
}
