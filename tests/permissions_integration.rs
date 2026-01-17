mod common;

use common::{ensure_companion_running, get_available_udid, get_test_bundle_id};
use std::process::Command;

#[test]
fn test_approve_photos() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", "photos", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run approve");

    assert!(
        output.status.success(),
        "approve photos failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_approve_camera() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", "camera", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run approve");

    assert!(output.status.success());
}

#[test]
fn test_approve_contacts() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", "contacts", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run approve");

    assert!(output.status.success());
}

#[test]
fn test_approve_location() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", "location", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run approve");

    assert!(output.status.success());
}

#[test]
fn test_approve_notifications() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "approve",
            "notifications",
            &bundle_id,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run approve");

    assert!(output.status.success());
}

#[test]
fn test_approve_url_with_scheme() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb", "approve", "url", &bundle_id, "--scheme", "https", "--udid", &udid,
        ])
        .output()
        .expect("Failed to run approve");

    assert!(output.status.success());
}

#[test]
fn test_approve_multiple_permissions() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Approve photos
    let output1 = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", "photos", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run approve");
    assert!(output1.status.success());

    // Approve camera
    let output2 = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", "camera", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run approve");
    assert!(output2.status.success());
}

#[test]
fn test_revoke_photos() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // First approve
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", "photos", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run approve");

    // Then revoke
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "revoke", "photos", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run revoke");

    assert!(
        output.status.success(),
        "revoke photos failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_revoke_camera() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "revoke", "camera", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run revoke");

    assert!(output.status.success());
}

#[test]
fn test_revoke_multiple_permissions() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Approve multiple permissions first
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", "photos", &bundle_id, "--udid", &udid])
        .output();
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", "camera", &bundle_id, "--udid", &udid])
        .output();

    // Revoke photos
    let output1 = Command::new("./target/debug/agent-mobile")
        .args(["idb", "revoke", "photos", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run revoke");
    assert!(output1.status.success());

    // Revoke camera
    let output2 = Command::new("./target/debug/agent-mobile")
        .args(["idb", "revoke", "camera", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run revoke");
    assert!(output2.status.success());
}

#[test]
fn test_approve_invalid_permission() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "approve",
            "invalid_permission",
            &bundle_id,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run approve");

    // 無効な権限タイプでエラーになるべき
    assert!(!output.status.success());
}

#[test]
fn test_revoke_invalid_permission() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "revoke",
            "invalid_permission",
            &bundle_id,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run revoke");

    // 無効な権限タイプでエラーになるべき
    assert!(!output.status.success());
}

#[test]
fn test_approve_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Python idb
    let python_output = Command::new("idb")
        .args(["approve", "photos", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", "photos", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );
}

#[test]
fn test_revoke_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Python idb
    let python_output = Command::new("idb")
        .args(["revoke", "photos", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "revoke", "photos", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );
}
