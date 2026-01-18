mod common;

use common::{ensure_framebuffer_ready, get_available_udid};
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_screenshot_to_file() {
    let udid = get_available_udid();
    ensure_framebuffer_ready(&udid);

    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "screenshot", "--udid", &udid, temp_path])
        .output()
        .expect("Failed to run screenshot command");

    // フレームバッファが初期化されていない場合はテストをスキップ
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No Image available to encode")
            || stderr.contains("framebuffer not ready")
        {
            eprintln!("Skipping test: framebuffer not ready");
            return;
        }
        panic!("Screenshot command failed: {}", stderr);
    }

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
    ensure_framebuffer_ready(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "screenshot", "--udid", &udid, "-"])
        .output()
        .expect("Failed to run screenshot command");

    // フレームバッファが初期化されていない場合はテストをスキップ
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No Image available to encode")
            || stderr.contains("framebuffer not ready")
        {
            eprintln!("Skipping test: framebuffer not ready");
            return;
        }
        panic!("Screenshot command failed: {}", stderr);
    }

    assert!(!output.stdout.is_empty(), "No data written to stdout");
    assert_eq!(
        &output.stdout[0..4],
        &[0x89, 0x50, 0x4E, 0x47],
        "Stdout is not a valid PNG"
    );
}

#[test]
fn test_screenshot_without_udid() {
    // Warm up the default simulator's framebuffer
    let udid = get_available_udid();
    ensure_framebuffer_ready(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "screenshot", "-"])
        .output()
        .expect("Failed to run screenshot command");

    // UDIDなしでもデフォルトのcompanionに接続できる場合、
    // スクリーンショットが成功するか、companion/framebuffer errorが発生する
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No companions available")
                || stderr.contains("Multiple companions available")
                || stderr.contains("No Image available to encode")
                || stderr.contains("framebuffer not ready"),
            "Expected companion or framebuffer error, got: {}",
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
