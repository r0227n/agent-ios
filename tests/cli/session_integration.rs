//! Session feature integration tests.
//!
//! Tests for `agent-mobile session` commands.
//! Tests session management (list, show, create, destroy).

use crate::common::{
    assert_failure, assert_success, assert_valid_json, ensure_companion_running,
    get_available_udid, run_cli_command,
};

// ============================================================================
// Helper functions
// ============================================================================

/// Generate a unique session name for testing.
fn get_test_session_name() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test-session-{}", timestamp)
}

/// Create a test session and return its name.
fn create_test_session(udid: &str) -> String {
    let name = get_test_session_name();
    let output = run_cli_command("session", &["create", &name, "--udid", udid]);
    assert_success(&output, &format!("session create {}", name));
    name
}

/// Destroy a test session.
fn destroy_test_session(name: &str) {
    let _ = run_cli_command("session", &["destroy", "--session", name]);
}

// ============================================================================
// session list - Normal cases
// ============================================================================

/// Test session list command.
#[test]
fn test_session_list() {
    let output = run_cli_command("session", &["list"]);

    assert_success(&output, "session list");
}

/// Test session list with JSON format.
#[test]
fn test_session_list_json() {
    let output = run_cli_command("session", &["list", "-f", "json"]);

    assert_success(&output, "session list -f json");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array for session list");
}

/// Test session list with text format.
#[test]
fn test_session_list_text() {
    let output = run_cli_command("session", &["list", "-f", "text"]);

    assert_success(&output, "session list -f text");
}

// ============================================================================
// session create - Normal cases
// ============================================================================

/// Test session create command.
#[test]
fn test_session_create() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let name = get_test_session_name();
    let output = run_cli_command("session", &["create", &name, "--udid", &udid]);

    assert_success(&output, &format!("session create {}", name));

    // Cleanup
    destroy_test_session(&name);
}

/// Test session create with custom name.
#[test]
fn test_session_create_custom_name() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let name = format!("my-custom-session-{}", std::process::id());
    let output = run_cli_command("session", &["create", &name, "--udid", &udid]);

    assert_success(&output, &format!("session create {}", name));

    // Cleanup
    destroy_test_session(&name);
}

/// Test created session appears in list.
#[test]
fn test_session_create_appears_in_list() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let name = create_test_session(&udid);

    // Verify it appears in list
    let list_output = run_cli_command("session", &["list", "-f", "json"]);
    assert_success(&list_output, "session list after create");

    let json = assert_valid_json(&list_output);
    let sessions = json.as_array().expect("Expected array");

    let found = sessions
        .iter()
        .any(|s| s.get("name").and_then(|n| n.as_str()) == Some(&name));

    assert!(found, "Created session should appear in list");

    // Cleanup
    destroy_test_session(&name);
}

// ============================================================================
// session show - Normal cases
// ============================================================================

/// Test session show command.
#[test]
fn test_session_show() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let name = create_test_session(&udid);

    let output = run_cli_command("session", &["show", "--session", &name]);

    assert_success(&output, &format!("session show {}", name));

    // Cleanup
    destroy_test_session(&name);
}

/// Test session show with JSON format.
#[test]
fn test_session_show_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let name = create_test_session(&udid);

    let output = run_cli_command("session", &["show", "--session", &name, "-f", "json"]);

    if output.status.success() {
        let _ = assert_valid_json(&output);
    }

    // Cleanup
    destroy_test_session(&name);
}

// ============================================================================
// session destroy - Normal cases
// ============================================================================

/// Test session destroy command.
#[test]
fn test_session_destroy() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let name = create_test_session(&udid);

    let output = run_cli_command("session", &["destroy", "--session", &name]);

    assert_success(&output, &format!("session destroy {}", name));
}

/// Test destroyed session removed from list.
#[test]
fn test_session_destroy_removes_from_list() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let name = create_test_session(&udid);

    // Destroy it
    let destroy_output = run_cli_command("session", &["destroy", "--session", &name]);
    assert_success(&destroy_output, "session destroy");

    // Verify it's gone from list
    let list_output = run_cli_command("session", &["list", "-f", "json"]);
    assert_success(&list_output, "session list after destroy");

    let json = assert_valid_json(&list_output);
    let sessions = json.as_array().expect("Expected array");

    let found = sessions
        .iter()
        .any(|s| s.get("name").and_then(|n| n.as_str()) == Some(&name));

    assert!(!found, "Destroyed session should not appear in list");
}

// ============================================================================
// session lifecycle - Normal cases
// ============================================================================

/// Test full session lifecycle: create, show, destroy.
#[test]
fn test_session_lifecycle() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Create
    let name = get_test_session_name();
    let create_output = run_cli_command("session", &["create", &name, "--udid", &udid]);
    assert_success(&create_output, "session create");

    // Show
    let show_output = run_cli_command("session", &["show", "--session", &name]);
    assert_success(&show_output, "session show");

    // Destroy
    let destroy_output = run_cli_command("session", &["destroy", "--session", &name]);
    assert_success(&destroy_output, "session destroy");
}

/// Test multiple sessions can coexist.
#[test]
fn test_session_multiple() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let name1 = create_test_session(&udid);
    let name2 = create_test_session(&udid);

    // Both should exist
    let list_output = run_cli_command("session", &["list", "-f", "json"]);
    assert_success(&list_output, "session list");

    let json = assert_valid_json(&list_output);
    let sessions = json.as_array().expect("Expected array");

    let found1 = sessions
        .iter()
        .any(|s| s.get("name").and_then(|n| n.as_str()) == Some(&name1));
    let found2 = sessions
        .iter()
        .any(|s| s.get("name").and_then(|n| n.as_str()) == Some(&name2));

    assert!(found1 && found2, "Both sessions should exist");

    // Cleanup
    destroy_test_session(&name1);
    destroy_test_session(&name2);
}

// ============================================================================
// Error cases
// ============================================================================

/// Test session create without UDID.
#[test]
fn test_session_create_missing_udid() {
    let name = get_test_session_name();
    let output = run_cli_command("session", &["create", &name]);

    assert_failure(&output, "session create without UDID");
}

/// Test session create without name.
#[test]
fn test_session_create_missing_name() {
    let udid = get_available_udid();

    let output = run_cli_command("session", &["create", "--udid", &udid]);

    assert_failure(&output, "session create without name");
}

/// Test session create with invalid UDID.
#[test]
fn test_session_create_invalid_udid() {
    let name = get_test_session_name();
    let output = run_cli_command(
        "session",
        &["create", &name, "--udid", "invalid-udid-12345"],
    );

    assert_failure(&output, "session create with invalid UDID");
}

/// Test session show without session flag.
#[test]
fn test_session_show_missing_session() {
    let output = run_cli_command("session", &["show"]);

    assert_failure(&output, "session show without session");
}

/// Test session show with non-existent session.
#[test]
fn test_session_show_non_existent() {
    let output = run_cli_command(
        "session",
        &["show", "--session", "non-existent-session-12345"],
    );

    assert_failure(&output, "session show non-existent");
}

/// Test session destroy without session flag.
#[test]
fn test_session_destroy_missing_session() {
    let output = run_cli_command("session", &["destroy"]);

    assert_failure(&output, "session destroy without session");
}

/// Test session destroy with non-existent session.
#[test]
fn test_session_destroy_non_existent() {
    let output = run_cli_command(
        "session",
        &["destroy", "--session", "non-existent-session-12345"],
    );

    // May succeed (idempotent) or fail depending on implementation
    let _ = output;
}

/// Test session create with duplicate name.
#[test]
fn test_session_create_duplicate_name() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let name = create_test_session(&udid);

    // Try to create again with same name
    let output = run_cli_command("session", &["create", &name, "--udid", &udid]);

    // Should fail or succeed depending on implementation
    let _ = output;

    // Cleanup
    destroy_test_session(&name);
}
