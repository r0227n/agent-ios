//! Client connection helpers for CLI commands
//!
//! This module provides utilities for connecting to idb_companion,
//! reducing boilerplate across command implementations.

use crate::companion::CompanionResolver;
use crate::grpc::IdbClient;

/// Standard result type for CLI commands
pub type CommandResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Execute a command with an IdbClient connection.
///
/// This helper function handles the common pattern of:
/// 1. Creating a CompanionResolver
/// 2. Connecting to a companion (with optional UDID)
/// 3. Executing the provided closure with the client
///
/// # Example
///
/// ```ignore
/// use crate::cli::helpers::{with_client, CommandResult};
///
/// pub async fn run(udid: Option<String>) -> CommandResult {
///     with_client(udid.as_deref(), |mut client| async move {
///         client.focus().await?;
///         Ok(())
///     }).await
/// }
/// ```
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
