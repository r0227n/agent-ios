//! Privacy feature integration tests.
//!
//! Tests for `agent-mobile app grant/revoke/reset` commands.

use crate::common::{
    assert_stdout_contains, assert_success, get_available_udid, get_test_bundle_id,
    run_cli_command_with_udid,
};

/// Test app grant command.
#[test]
fn test_privacy_grant() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    let output =
        run_cli_command_with_udid("app", &["grant", "location", "--bundle", &bundle_id], &udid);

    assert_success(&output, "app grant location");
    assert_stdout_contains(&output, "Granted");
}

/// Test app revoke command.
#[test]
fn test_privacy_revoke() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    // First grant
    let grant_output =
        run_cli_command_with_udid("app", &["grant", "location", "--bundle", &bundle_id], &udid);
    assert_success(&grant_output, "app grant (setup)");

    // Then revoke
    let output = run_cli_command_with_udid(
        "app",
        &["revoke", "location", "--bundle", &bundle_id],
        &udid,
    );

    assert_success(&output, "app revoke location");
    assert_stdout_contains(&output, "Revoked");
}

/// Test app reset command.
#[test]
fn test_privacy_reset() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    let output =
        run_cli_command_with_udid("app", &["reset", "location", "--bundle", &bundle_id], &udid);

    assert_success(&output, "app reset location");
    assert_stdout_contains(&output, "Reset");
}

/// Test app grant camera permission.
#[test]
fn test_privacy_grant_camera() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    let output =
        run_cli_command_with_udid("app", &["grant", "camera", "--bundle", &bundle_id], &udid);

    assert_success(&output, "app grant camera");
    assert_stdout_contains(&output, "Granted");
}
