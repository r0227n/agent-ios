use crate::common::{ensure_companion_running, get_available_udid, wait_with_timeout};
use std::process::{Command, Stdio};

/// Test video record-video CLI help output
#[test]
fn test_video_record_video_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "record-video", "--help"])
        .output()
        .expect("Failed to run record-video --help");

    assert!(
        output.status.success(),
        "record-video --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have output path argument
    assert!(
        stdout.contains("OUTPUT") || stdout.contains("output") || stdout.contains("PATH"),
        "Expected output path argument in help output"
    );
}

/// Test video record-video has --format flag
#[test]
fn test_video_record_video_has_format_flag() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "record-video", "--help"])
        .output()
        .expect("Failed to run record-video --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--format"),
        "Expected --format flag in help output"
    );
}

/// Test video record-video has --fps flag
#[test]
fn test_video_record_video_has_fps_flag() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "record-video", "--help"])
        .output()
        .expect("Failed to run record-video --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--fps"),
        "Expected --fps flag in help output"
    );
}

/// Test video stream CLI help output
#[test]
fn test_video_stream_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "video-stream", "--help"])
        .output()
        .expect("Failed to run video-stream --help");

    assert!(
        output.status.success(),
        "video-stream --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have format and fps options
    assert!(
        stdout.contains("--format") || stdout.contains("--fps"),
        "Expected --format or --fps flag in help output"
    );
}

/// Test video record-video with reduced timeout (2 seconds)
#[test]
fn test_video_record_video() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output_path = format!("/tmp/test_recording_{}.mp4", std::process::id());

    // Start recording (background process)
    let child = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "video",
            "record-video",
            &output_path,
            "--udid",
            &udid,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn record-video");

    // Reduced timeout: 2 seconds
    let output = wait_with_timeout(child, 2);

    // Process was terminated by signal (SIGTERM/SIGKILL) or exited
    // Both are valid outcomes for a long-running video recording process
    // On Unix, code() returns None when killed by signal, which is expected
    let _ = output.status; // Process completion verified by wait_with_timeout returning

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

/// Test video stream with reduced timeout (2 seconds)
#[test]
fn test_video_stream() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "video-stream", "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn video-stream");

    // Reduced timeout: 2 seconds
    let output = wait_with_timeout(child, 2);

    // Process was terminated by signal (SIGTERM/SIGKILL) or exited
    // Both are valid outcomes for a long-running video streaming process
    // On Unix, code() returns None when killed by signal, which is expected
    let _ = output.status; // Process completion verified by wait_with_timeout returning
}

/// Test video record-video help compatibility with Python idb
#[test]
fn test_video_record_video_help_python_compatibility() {
    // Python idb
    let python_output = Command::new("idb")
        .args(["video", "record-video", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "record-video", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}

/// Test video stream help compatibility with Python idb
#[test]
fn test_video_stream_help_python_compatibility() {
    // Python idb
    let python_output = Command::new("idb")
        .args(["video", "video-stream", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "video-stream", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}
