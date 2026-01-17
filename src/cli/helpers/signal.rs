//! Signal handling helpers for streaming commands
//!
//! This module provides utilities for handling Ctrl+C (SIGINT)
//! in long-running commands like `log`, `launch`, and `file tail`.

use tokio::sync::watch;

/// Receiver for stop signals
pub type StopReceiver = watch::Receiver<bool>;

/// Setup Ctrl+C signal handler.
///
/// Returns a receiver that will receive `true` when Ctrl+C is pressed.
/// This is used for graceful shutdown of streaming commands.
///
/// # Example
///
/// ```ignore
/// use crate::cli::helpers::setup_ctrl_c_handler;
///
/// let stop_rx = setup_ctrl_c_handler();
///
/// loop {
///     tokio::select! {
///         _ = stop_rx.changed() => {
///             if *stop_rx.borrow() {
///                 break;
///             }
///         }
///         response = stream.message() => {
///             // handle response
///         }
///     }
/// }
/// ```
pub fn setup_ctrl_c_handler() -> StopReceiver {
    let (stop_tx, stop_rx) = watch::channel(false);

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        let _ = stop_tx.send(true);
    });

    stop_rx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stop_receiver_initial_value() {
        let stop_rx = setup_ctrl_c_handler();
        // Initial value should be false
        assert!(!*stop_rx.borrow());
    }
}
