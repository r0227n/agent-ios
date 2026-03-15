//! wait command - Wait for element
//!
//! ```bash
//! agent-mobile wait visible @e1 --timeout 10s
//! agent-mobile wait gone @e1
//! agent-mobile wait idle
//! ```

use std::time::{Duration, Instant};

use clap::Args;

use agent_mobile_core::Platform;

use crate::helpers::client::CommandResult;
use crate::helpers::common_args::DeviceArgs;

use super::ref_resolver::ElementTarget;
use super::tap::{current_ui_hash_ios, query_exists_ios, resolve_element, take_snapshot};
use agent_mobile_gateway::DeviceResolver;

/// Arguments for the wait command
#[derive(Args, Debug)]
pub struct WaitArgs {
    /// Condition to wait for: visible, gone, idle, text
    pub condition: String,

    /// Element ref (@eN) or "text" (not required for "idle")
    pub target: Option<String>,

    /// Timeout (e.g., "10s", "1m", "30")
    #[arg(long, default_value = "30s")]
    pub timeout: String,

    /// Polling interval in milliseconds
    #[arg(long, default_value = "500")]
    pub interval: u64,

    /// Device selection options.
    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the wait command
pub async fn run(args: WaitArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };
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
        "text" => {
            let text = args
                .target
                .as_ref()
                .ok_or("Text required for 'text' condition")?;
            wait_text(
                platform,
                args.device.udid.as_deref(),
                text,
                timeout,
                interval,
            )
            .await
        }
        _ => Err(format!(
            "Unknown condition: {}. Valid conditions: visible, gone, idle, text",
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
    let target = ElementTarget::parse(target_str);
    let deadline = Instant::now() + timeout;

    loop {
        let found = match (&platform, &target) {
            (Platform::Ios, ElementTarget::Text(text)) => {
                query_exists_ios(udid, "text", text, false, false, None).await?
            }
            _ => resolve_element(&target, platform, udid).await.is_ok(),
        };

        if found {
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
    let target = ElementTarget::parse(target_str);
    let deadline = Instant::now() + timeout;

    loop {
        let gone = match (&platform, &target) {
            (Platform::Ios, ElementTarget::Text(text)) => {
                !query_exists_ios(udid, "text", text, false, false, None).await?
            }
            _ => resolve_element(&target, platform, udid).await.is_err(),
        };

        if gone {
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
    let mut last_state: Option<String> = None;
    let mut stable_count = 0;
    const REQUIRED_STABLE_CHECKS: usize = 3;

    loop {
        let current_state = match platform {
            Platform::Ios => current_ui_hash_ios(udid, Some("screenshot"), Some(1)).await?,
            Platform::Android => {
                let snapshot = take_snapshot(platform, udid).await?;
                snapshot.elements.len().to_string()
            }
        };

        if let Some(last_state) = &last_state {
            if last_state == &current_state {
                stable_count += 1;
                if stable_count >= REQUIRED_STABLE_CHECKS {
                    println!("UI is idle");
                    return Ok(());
                }
            } else {
                stable_count = 0;
            }
        }

        last_state = Some(current_state);

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

/// Wait for text to appear on screen
async fn wait_text(
    platform: Platform,
    udid: Option<&str>,
    text: &str,
    timeout: Duration,
    interval: Duration,
) -> CommandResult {
    let deadline = Instant::now() + timeout;
    let text_lower = text.to_lowercase();

    loop {
        let found = match platform {
            Platform::Ios => query_exists_ios(udid, "text", text, false, false, None).await?,
            Platform::Android => {
                let snapshot = take_snapshot(platform, udid).await?;
                snapshot.elements.iter().any(|e| {
                    e.label
                        .as_ref()
                        .map(|l| l.to_lowercase().contains(&text_lower))
                        .unwrap_or(false)
                        || e.value
                            .as_ref()
                            .map(|v| v.to_lowercase().contains(&text_lower))
                            .unwrap_or(false)
                })
            }
        };

        if found {
            println!("Found text: {}", text);
            return Ok(());
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "Timeout waiting for text '{}' to appear (waited {:?})",
                text, timeout
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
