use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Get a valid UDID from idb list-targets (prefers Booted simulator)
fn get_available_udid() -> Option<String> {
    let output = Command::new("idb")
        .args(["list-targets", "--json"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(udid) = json.get("udid").and_then(|v| v.as_str()) {
                // Prefer Booted simulator
                if json.get("state").and_then(|v| v.as_str()) == Some("Booted") {
                    return Some(udid.to_string());
                }
            }
        }
    }
    None
}

#[test]
fn test_log_basic_execution() {
    let udid = match get_available_udid() {
        Some(u) => u,
        None => {
            eprintln!("Skipping test: no booted simulator available");
            return;
        }
    };

    // Start log command
    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "log", "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start agent-mobile");

    // Wait a short time and then terminate
    thread::sleep(Duration::from_secs(2));

    // Send SIGTERM to gracefully stop
    #[cfg(unix)]
    {
        let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);
    }

    let output = child.wait_with_output().expect("Failed to wait for child");

    // Basic sanity check - no panic or crash
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic") && !stderr.contains("SIGSEGV"),
        "Log command crashed: stderr={}",
        stderr
    );
}

#[test]
fn test_log_with_source_companion() {
    let udid = match get_available_udid() {
        Some(u) => u,
        None => {
            eprintln!("Skipping test: no booted simulator available");
            return;
        }
    };

    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "log", "--udid", &udid, "--source", "companion"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start agent-mobile");

    thread::sleep(Duration::from_secs(2));

    #[cfg(unix)]
    {
        let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);
    }

    let output = child.wait_with_output().expect("Failed to wait for child");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("panic"),
        "Log command with --source companion crashed"
    );
}

#[test]
fn test_log_invalid_udid() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "log", "--udid", "INVALID-UDID-12345"])
        .output();

    match output {
        Ok(result) => {
            assert!(
                !result.status.success(),
                "log with invalid UDID should fail"
            );
        }
        Err(e) => {
            panic!("Command execution failed: {}", e);
        }
    }
}

#[test]
fn test_log_help() {
    // Test that help output includes log command
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "log", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "help command should succeed");
    assert!(
        stdout.contains("target") || stdout.contains("companion"),
        "help should mention source options"
    );
    assert!(
        stdout.contains("--udid"),
        "help should mention --udid option"
    );
}
