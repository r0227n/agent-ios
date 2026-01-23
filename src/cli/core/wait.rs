//! wait コマンド - 要素待機
//!
//! ```bash
//! agent-mobile wait visible @e1 --timeout 10s
//! agent-mobile wait gone @e1
//! agent-mobile wait idle
//! ```

use std::time::{Duration, Instant};

use clap::Args;

use agent_mobile_core::Platform;

use crate::cli::helpers::{CommandResult, DeviceArgs};

use super::ref_resolver::{self, Target};
use super::tap::{resolve_platform, take_snapshot};

/// wait コマンド引数
#[derive(Args, Debug)]
pub struct WaitArgs {
    /// Condition to wait for: visible, gone, idle
    pub condition: String,

    /// Element ref (@eN) or "text" (not required for "idle")
    pub target: Option<String>,

    /// Timeout (e.g., "10s", "1m", "30")
    #[arg(long, default_value = "30s")]
    pub timeout: String,

    /// Polling interval in milliseconds
    #[arg(long, default_value = "500")]
    pub interval: u64,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the wait command
pub async fn run(args: WaitArgs) -> CommandResult {
    let platform = resolve_platform(args.device.platform.as_deref()).await?;
    let timeout = parse_timeout(&args.timeout)?;
    let interval = Duration::from_millis(args.interval);

    match args.condition.to_lowercase().as_str() {
        "visible" | "exists" => {
            let target_str = args
                .target
                .as_ref()
                .ok_or("Target required for 'visible' condition")?;
            wait_visible(
                platform,
                args.device.udid.as_deref(),
                target_str,
                timeout,
                interval,
            )
            .await
        }
        "gone" | "invisible" | "hidden" => {
            let target_str = args
                .target
                .as_ref()
                .ok_or("Target required for 'gone' condition")?;
            wait_gone(
                platform,
                args.device.udid.as_deref(),
                target_str,
                timeout,
                interval,
            )
            .await
        }
        "idle" => wait_idle(platform, args.device.udid.as_deref(), timeout, interval).await,
        _ => Err(format!(
            "Unknown condition: {}. Valid conditions: visible, gone, idle",
            args.condition
        )
        .into()),
    }
}

/// Wait for an element to become visible
async fn wait_visible(
    platform: Platform,
    udid: Option<&str>,
    target_str: &str,
    timeout: Duration,
    interval: Duration,
) -> CommandResult {
    let target = Target::parse(target_str);
    let deadline = Instant::now() + timeout;

    loop {
        // Take a fresh snapshot
        let snapshot = take_snapshot(platform, udid).await?;

        // Try to find element
        if ref_resolver::resolve_from_snapshot(&snapshot, &target).is_ok() {
            println!("Found: {}", target_str);
            return Ok(());
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "Timeout waiting for '{}' to become visible (waited {:?})",
                target_str, timeout
            )
            .into());
        }

        tokio::time::sleep(interval).await;
    }
}

/// Wait for an element to disappear
async fn wait_gone(
    platform: Platform,
    udid: Option<&str>,
    target_str: &str,
    timeout: Duration,
    interval: Duration,
) -> CommandResult {
    let target = Target::parse(target_str);
    let deadline = Instant::now() + timeout;

    loop {
        // Take a fresh snapshot
        let snapshot = take_snapshot(platform, udid).await?;

        // Check if element is gone
        if ref_resolver::resolve_from_snapshot(&snapshot, &target).is_err() {
            println!("Gone: {}", target_str);
            return Ok(());
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "Timeout waiting for '{}' to disappear (waited {:?})",
                target_str, timeout
            )
            .into());
        }

        tokio::time::sleep(interval).await;
    }
}

/// Wait for the UI to become idle (no changes between snapshots)
async fn wait_idle(
    platform: Platform,
    udid: Option<&str>,
    timeout: Duration,
    interval: Duration,
) -> CommandResult {
    let deadline = Instant::now() + timeout;
    let mut last_element_count: Option<usize> = None;
    let mut stable_count = 0;
    const REQUIRED_STABLE_CHECKS: usize = 3;

    loop {
        // Take snapshot
        let snapshot = take_snapshot(platform, udid).await?;
        let current_count = snapshot.elements.len();

        // Check stability
        if let Some(last_count) = last_element_count {
            if last_count == current_count {
                stable_count += 1;
                if stable_count >= REQUIRED_STABLE_CHECKS {
                    println!("UI is idle");
                    return Ok(());
                }
            } else {
                stable_count = 0;
            }
        }

        last_element_count = Some(current_count);

        if Instant::now() >= deadline {
            return Err(format!(
                "Timeout waiting for UI to become idle (waited {:?})",
                timeout
            )
            .into());
        }

        tokio::time::sleep(interval).await;
    }
}

/// Parse timeout string (e.g., "10s", "1m", "30")
fn parse_timeout(s: &str) -> CommandResult<Duration> {
    let s = s.trim().to_lowercase();

    let secs = if let Some(secs_str) = s.strip_suffix('s') {
        secs_str
            .parse::<f64>()
            .map_err(|_| format!("Invalid timeout: {}", s))?
    } else if let Some(mins_str) = s.strip_suffix('m') {
        let mins: f64 = mins_str
            .parse()
            .map_err(|_| format!("Invalid timeout: {}", s))?;
        mins * 60.0
    } else {
        // Default: parse as seconds
        s.parse().map_err(|_| format!("Invalid timeout: {}", s))?
    };

    // Validate: must be finite and non-negative
    if !secs.is_finite() || secs < 0.0 {
        return Err(format!(
            "Invalid timeout value: {}. Must be a non-negative finite number.",
            s
        )
        .into());
    }

    Ok(Duration::from_secs_f64(secs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timeout_seconds() {
        assert_eq!(parse_timeout("10s").unwrap(), Duration::from_secs(10));
        assert_eq!(parse_timeout("5.5s").unwrap(), Duration::from_secs_f64(5.5));
    }

    #[test]
    fn test_parse_timeout_minutes() {
        assert_eq!(parse_timeout("1m").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_timeout("2.5m").unwrap(), Duration::from_secs(150));
    }

    #[test]
    fn test_parse_timeout_plain_number() {
        assert_eq!(parse_timeout("30").unwrap(), Duration::from_secs(30));
    }

    #[test]
    fn test_parse_timeout_negative_value() {
        assert!(parse_timeout("-5s").is_err());
        assert!(parse_timeout("-1m").is_err());
        assert!(parse_timeout("-10").is_err());
    }

    #[test]
    fn test_parse_timeout_invalid_value() {
        assert!(parse_timeout("abc").is_err());
        assert!(parse_timeout("").is_err());
    }
}
