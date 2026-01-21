//! Privacy feature integration tests.
//!
//! Tests for `agent-mobile privacy` commands.

use crate::common::{
    assert_stdout_contains, assert_success, get_available_udid, get_test_bundle_id,
    run_cli_command_with_udid,
};

/// Test privacy grant command.
#[test]
fn test_privacy_grant() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    let output =
        run_cli_command_with_udid("privacy", &["grant", "location", "-b", &bundle_id], &udid);

    assert_success(&output, "privacy grant location");
    assert_stdout_contains(&output, "Granted");
}

/// Test privacy revoke command.
#[test]
fn test_privacy_revoke() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    // First grant
    let grant_output =
        run_cli_command_with_udid("privacy", &["grant", "location", "-b", &bundle_id], &udid);
    assert_success(&grant_output, "privacy grant (setup)");

    // Then revoke
    let output =
        run_cli_command_with_udid("privacy", &["revoke", "location", "-b", &bundle_id], &udid);

    assert_success(&output, "privacy revoke location");
    assert_stdout_contains(&output, "Revoked");
}

/// Test privacy reset command.
#[test]
fn test_privacy_reset() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    let output =
        run_cli_command_with_udid("privacy", &["reset", "location", "-b", &bundle_id], &udid);

    assert_success(&output, "privacy reset location");
    assert_stdout_contains(&output, "Reset");
}

/// Test privacy grant camera permission.
#[test]
fn test_privacy_grant_camera() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    let output =
        run_cli_command_with_udid("privacy", &["grant", "camera", "-b", &bundle_id], &udid);

    assert_success(&output, "privacy grant camera");
    assert_stdout_contains(&output, "Granted");
}
