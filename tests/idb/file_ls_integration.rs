/// Integration tests for file ls command
/// Compares Python idb output with agent-mobile output to ensure compatibility

#[test]
fn test_ls_basic_equivalence() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    // Test listing /tmp directory
    let python_output = crate::common::run_idb_file_command(&["ls", "/tmp", "--udid", &udid]);
    let rust_output =
        crate::common::run_agent_mobile_file_command(&["ls", "/tmp", "--udid", &udid]);

    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
fn test_ls_multiple_paths_equivalence() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    // Test listing multiple paths
    let python_output =
        crate::common::run_idb_file_command(&["ls", "/tmp", "/var", "--udid", &udid]);
    let rust_output =
        crate::common::run_agent_mobile_file_command(&["ls", "/tmp", "/var", "--udid", &udid]);

    // Use order-independent comparison for multi-path ls
    // (gRPC response order may differ from input order)
    crate::common::compare_file_ls_multi_path_outputs(&python_output, &rust_output);
}

#[test]
fn test_ls_with_bundle_id_equivalence() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);
    let bundle_id = crate::common::get_test_bundle_id();

    // Test listing app container with bundle ID
    let python_output = crate::common::run_idb_file_command(&[
        "ls",
        "/",
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);
    let rust_output = crate::common::run_agent_mobile_file_command(&[
        "ls",
        "/",
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
fn test_ls_nonexistent_path_error() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    let nonexistent_path = "/this/path/does/not/exist/12345";

    // Both should fail with error
    let python_output =
        crate::common::run_idb_file_command(&["ls", nonexistent_path, "--udid", &udid]);
    let rust_output =
        crate::common::run_agent_mobile_file_command(&["ls", nonexistent_path, "--udid", &udid]);

    // Both should have non-zero exit codes
    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());

    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
fn test_ls_without_udid() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    // When only one target is available, both should work without --udid
    let python_output = crate::common::run_idb_file_command(&["ls", "/tmp"]);
    let rust_output = crate::common::run_agent_mobile_file_command(&["ls", "/tmp"]);

    // Both should succeed (assuming only one booted simulator)
    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
fn test_ls_root_directory() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    // Test listing root directory
    let python_output = crate::common::run_idb_file_command(&["ls", "/", "--udid", &udid]);
    let rust_output = crate::common::run_agent_mobile_file_command(&["ls", "/", "--udid", &udid]);

    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}
