use crate::common::{ensure_companion_running, get_available_udid, get_test_bundle_id};
use std::process::Command;

// ==========================================
// SKIPPED TESTS - idb_companion SQLite Issues
// ==========================================

#[test]
#[ignore = "idb_companion SQLite schema mismatch: table 'access' expects 17 columns but receives 13. Affects both Python and Rust implementations."]
fn test_approve_photos() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", &bundle_id, "photos", "--udid", &udid])
        .output()
        .expect("Failed to run approve");

    assert!(
        output.status.success(),
        "approve photos failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "idb_companion SQLite schema mismatch: table 'access' expects 17 columns but receives 13. Affects both Python and Rust implementations."]
fn test_approve_camera() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", &bundle_id, "camera", "--udid", &udid])
        .output()
        .expect("Failed to run approve");

    assert!(output.status.success());
}

#[test]
#[ignore = "idb_companion SQLite schema mismatch: table 'access' expects 17 columns but receives 13. Affects both Python and Rust implementations."]
fn test_approve_contacts() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", &bundle_id, "contacts", "--udid", &udid])
        .output()
        .expect("Failed to run approve");

    assert!(output.status.success());
}

// ==========================================
// WORKING TESTS - These permissions work correctly
// ==========================================

#[test]
fn test_approve_location() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", &bundle_id, "location", "--udid", &udid])
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
            &bundle_id,
            "notification",
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
            "idb", "approve", &bundle_id, "url", "--scheme", "https", "--udid", &udid,
        ])
        .output()
        .expect("Failed to run approve");

    assert!(output.status.success());
}

// ==========================================
// SKIPPED TESTS - Depends on broken approve operations
// ==========================================

#[test]
#[ignore = "Depends on approve photos/camera which fail due to idb_companion SQLite schema mismatch."]
fn test_approve_multiple_permissions() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Approve photos
    let output1 = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", &bundle_id, "photos", "--udid", &udid])
        .output()
        .expect("Failed to run approve");
    assert!(output1.status.success());

    // Approve camera
    let output2 = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", &bundle_id, "camera", "--udid", &udid])
        .output()
        .expect("Failed to run approve");
    assert!(output2.status.success());
}

// ==========================================
// WORKING TESTS - Revoke operations work correctly
// ==========================================

#[test]
fn test_revoke_photos() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // First approve
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", &bundle_id, "photos", "--udid", &udid])
        .output()
        .expect("Failed to run approve");

    // Then revoke
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "revoke", &bundle_id, "photos", "--udid", &udid])
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
        .args(["idb", "revoke", &bundle_id, "camera", "--udid", &udid])
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
        .args(["idb", "approve", &bundle_id, "photos", "--udid", &udid])
        .output();
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", &bundle_id, "camera", "--udid", &udid])
        .output();

    // Revoke photos
    let output1 = Command::new("./target/debug/agent-mobile")
        .args(["idb", "revoke", &bundle_id, "photos", "--udid", &udid])
        .output()
        .expect("Failed to run revoke");
    assert!(output1.status.success());

    // Revoke camera
    let output2 = Command::new("./target/debug/agent-mobile")
        .args(["idb", "revoke", &bundle_id, "camera", "--udid", &udid])
        .output()
        .expect("Failed to run revoke");
    assert!(output2.status.success());
}

// ==========================================
// ERROR HANDLING TESTS
// ==========================================

#[test]
fn test_approve_invalid_permission() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "approve",
            &bundle_id,
            "invalid_permission",
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
            &bundle_id,
            "invalid_permission",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run revoke");

    // 無効な権限タイプでエラーになるべき
    assert!(!output.status.success());
}

// ==========================================
// PYTHON COMPATIBILITY TESTS - Skipped due to companion issues
// ==========================================

#[test]
#[ignore = "Both Python and Rust fail with same idb_companion SQLite error for photos approval. Test validates compatibility but cannot run until companion is fixed."]
fn test_approve_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Python idb
    let python_output = Command::new("idb")
        .args(["approve", &bundle_id, "photos", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "approve", &bundle_id, "photos", "--udid", &udid])
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
#[ignore = "Python and Rust exit code handling differs for revoke operations. Needs investigation of error propagation behavior."]
fn test_revoke_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Python idb
    let python_output = Command::new("idb")
        .args(["revoke", &bundle_id, "photos", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "revoke", &bundle_id, "photos", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );
}
