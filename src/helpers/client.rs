//! Client connection helpers for CLI commands
//!
//! This module provides utilities for connecting to idb_companion,
//! reducing boilerplate across command implementations.

use agent_mobile_platform_ios::companion::CompanionResolver;
use agent_mobile_platform_ios::grpc::IdbClient;

/// Standard result type for CLI commands
pub type CommandResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Execute a command with an IdbClient connection.
///
/// This helper function handles the common pattern of:
/// 1. Creating a CompanionResolver
/// 2. Connecting to a companion (with optional UDID)
/// 3. Executing the provided closure with the client
///
/// # UDID Resolution Priority
///
/// The UDID parameter passed here follows this priority order (enforced at CLI layer):
/// 1. **Session UDID** (highest priority)
///    - Set via `apply_session_udid_option!` macro in main.rs
///    - SessionResolver converts session name → UDID
///    - Main.rs macro applies resolved UDID before calling command
/// 2. **Explicit UDID** (mid priority)
///    - From --udid flag or -u short form
///    - Passed directly to this helper
/// 3. **Auto-detection** (lowest priority)
///    - If no UDID: CompanionResolver auto-selects single companion
///    - If multiple companions exist: error, user must specify UDID
///
/// # Example
///
/// ```ignore
/// use crate::helpers::client::{with_client, CommandResult};
///
/// pub async fn run(udid: Option<&str>) -> CommandResult {
///     with_client(udid, |mut client| async move {
///         client.focus().await?;
///         Ok(())
///     }).await
/// }
/// ```
///
/// # See Also
///
/// - `apply_session_udid_option!` macro in main.rs: Applies session UDID before command execution
/// - `CompanionResolver`: Handles actual UDID resolution and companion connection
pub async fn with_client<F, Fut, T>(udid: Option<&str>, f: F) -> CommandResult<T>
where
    F: FnOnce(IdbClient) -> Fut,
    Fut: std::future::Future<Output = CommandResult<T>>,
{
    let resolver = CompanionResolver::new();
    let client = resolver.connect(udid).await?;
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
