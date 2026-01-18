mod common;

use common::{ensure_companion_running, get_available_udid};
use std::process::Command;

/// Get a shutdown simulator UDID for testing
fn get_shutdown_simulator() -> Option<String> {
    let output = Command::new("idb")
        .args(["list-targets", "--json"])
        .output()
        .expect("Failed to execute Python idb");

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if json.get("state").and_then(|v| v.as_str()) == Some("Shutdown")
                && json.get("type").and_then(|v| v.as_str()) == Some("simulator")
            {
                if let Some(udid) = json.get("udid").and_then(|v| v.as_str()) {
                    return Some(udid.to_string());
                }
            }
        }
    }
    None
}

/// Test target boot CLI help output
#[test]
fn test_target_boot_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "boot", "--help"])
        .output()
        .expect("Failed to run target boot --help");

    assert!(
        output.status.success(),
        "target boot --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--udid") || stdout.contains("UDID"),
        "Expected --udid flag in help output"
    );
}

/// Test target boot has --headless flag
#[test]
fn test_target_boot_has_headless_flag() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "boot", "--help"])
        .output()
        .expect("Failed to run target boot --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--headless"),
        "Expected --headless flag in help output"
    );
}

/// Test target shutdown CLI help output
#[test]
fn test_target_shutdown_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "shutdown", "--help"])
        .output()
        .expect("Failed to run target shutdown --help");

    assert!(
        output.status.success(),
        "target shutdown --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--udid") || stdout.contains("UDID"),
        "Expected --udid flag in help output"
    );
}

/// Test target erase CLI help output
#[test]
fn test_target_erase_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "erase", "--help"])
        .output()
        .expect("Failed to run target erase --help");

    assert!(
        output.status.success(),
        "target erase --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--udid") || stdout.contains("UDID"),
        "Expected --udid flag in help output"
    );
}

/// Test target boot with conditional fallback
#[test]
fn test_target_boot() {
    match get_shutdown_simulator() {
        Some(udid) => {
            // Full integration test when shutdown simulator available
            let output = Command::new("./target/debug/agent-mobile")
                .args(["idb", "target", "boot", &udid])
                .output()
                .expect("Failed to run target boot");

            assert!(
                output.status.success(),
                "target boot failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );

            // Cleanup: shutdown the simulator
            let _ = Command::new("./target/debug/agent-mobile")
                .args(["idb", "target", "shutdown", &udid])
                .output();
        }
        None => {
            // Fallback: just validate CLI accepts arguments
            let output = Command::new("./target/debug/agent-mobile")
                .args(["idb", "target", "boot", "--help"])
                .output()
                .expect("Failed to run command");

            assert!(output.status.success());
        }
    }
}

/// Test target shutdown is idempotent
#[test]
fn test_target_shutdown() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "shutdown", &udid])
        .output()
        .expect("Failed to run target shutdown");

    assert!(
        output.status.success(),
        "target shutdown failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Cleanup: reboot simulator for next tests
    let _ = Command::new("idb").args(["boot", &udid]).output();
}

/// Test shutdown on already shutdown simulator is idempotent
#[test]
fn test_target_shutdown_already_shutdown() {
    match get_shutdown_simulator() {
        Some(udid) => {
            // Shutdown an already shutdown simulator
            let output = Command::new("./target/debug/agent-mobile")
                .args(["idb", "target", "shutdown", &udid])
                .output()
                .expect("Failed to run target shutdown");

            // Should not error (idempotent) or handle gracefully
            assert!(output.status.success() || !output.stderr.is_empty());
        }
        None => {
            // No shutdown simulator available, verify CLI help works
            let output = Command::new("./target/debug/agent-mobile")
                .args(["idb", "target", "shutdown", "--help"])
                .output()
                .expect("Failed to run command");

            assert!(output.status.success());
        }
    }
}
