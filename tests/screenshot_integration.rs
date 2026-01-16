use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

fn get_available_udid() -> String {
    let output = Command::new("idb")
        .args(["list-targets", "--json"])
        .output()
        .expect("Failed to execute Python idb - ensure idb is installed and in PATH");

    assert!(
        output.status.success(),
        "Python idb list-targets failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(udid) = json.get("udid").and_then(|v| v.as_str()) {
                // Prefer Booted simulator
                if json.get("state").and_then(|v| v.as_str()) == Some("Booted") {
                    return udid.to_string();
                }
            }
        }
    }
    panic!("No booted simulator with companion available");
}

#[test]
fn test_screenshot_to_file() {
    let udid = get_available_udid();

    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "screenshot", "--udid", &udid, temp_path])
        .output()
        .expect("Failed to run screenshot command");

    assert!(
        output.status.success(),
        "Screenshot command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let file_content = fs::read(temp_path).expect("Failed to read screenshot file");
    assert!(!file_content.is_empty(), "Screenshot file is empty");

    // Verify PNG signature (89 50 4E 47)
    assert_eq!(
        &file_content[0..4],
        &[0x89, 0x50, 0x4E, 0x47],
        "File is not a valid PNG"
    );
}

#[test]
fn test_screenshot_to_stdout() {
    let udid = get_available_udid();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "screenshot", "--udid", &udid, "-"])
        .output()
        .expect("Failed to run screenshot command");

    assert!(
        output.status.success(),
        "Screenshot command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(!output.stdout.is_empty(), "No data written to stdout");
    assert_eq!(
        &output.stdout[0..4],
        &[0x89, 0x50, 0x4E, 0x47],
        "Stdout is not a valid PNG"
    );
}

#[test]
fn test_screenshot_without_udid() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "screenshot", "-"])
        .output()
        .expect("Failed to run screenshot command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No companions available"),
            "Expected companion error, got: {}",
            stderr
        );
    } else {
        // If succeeded, verify PNG output
        assert_eq!(&output.stdout[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }
}

#[test]
fn test_screenshot_invalid_udid() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "screenshot", "--udid", "INVALID_UDID_12345", "-"])
        .output()
        .expect("Failed to run screenshot command");

    assert!(!output.status.success(), "Should fail with invalid UDID");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No companion found for UDID"),
        "Expected UDID error, got: {}",
        stderr
    );
}
