/// Integration tests for file rm command
/// Compares Python idb output with agent-mobile output to ensure compatibility

#[test]
fn test_rm_single_file_equivalence() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let test_file = format!("/tmp/test_rm_single_{}", pid);

    // Create test file and remove with Python idb
    let _create =
        crate::common::run_idb_file_command(&["push", "/dev/null", &test_file, "--udid", &udid]);
    let python_output = crate::common::run_idb_file_command(&["rm", &test_file, "--udid", &udid]);

    // Create test file and remove with agent-mobile
    let _create =
        crate::common::run_idb_file_command(&["push", "/dev/null", &test_file, "--udid", &udid]);
    let rust_output =
        crate::common::run_agent_mobile_file_command(&["rm", &test_file, "--udid", &udid]);

    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
fn test_rm_multiple_files_equivalence() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let file1 = format!("/tmp/test_rm_multi1_{}", pid);
    let file2 = format!("/tmp/test_rm_multi2_{}", pid);

    // Create test files and remove with Python idb
    let _create1 =
        crate::common::run_idb_file_command(&["push", "/dev/null", &file1, "--udid", &udid]);
    let _create2 =
        crate::common::run_idb_file_command(&["push", "/dev/null", &file2, "--udid", &udid]);
    let python_output =
        crate::common::run_idb_file_command(&["rm", &file1, &file2, "--udid", &udid]);

    // Create test files and remove with agent-mobile
    let _create1 =
        crate::common::run_idb_file_command(&["push", "/dev/null", &file1, "--udid", &udid]);
    let _create2 =
        crate::common::run_idb_file_command(&["push", "/dev/null", &file2, "--udid", &udid]);
    let rust_output =
        crate::common::run_agent_mobile_file_command(&["rm", &file1, &file2, "--udid", &udid]);

    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
fn test_rm_directory_recursive_equivalence() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let test_dir = format!("/tmp/test_rm_dir_{}", pid);
    let test_file = format!("{}/test_file", test_dir);

    // Create directory with file and remove with Python idb
    let _mkdir = crate::common::run_idb_file_command(&["mkdir", &test_dir, "--udid", &udid]);
    let _create =
        crate::common::run_idb_file_command(&["push", "/dev/null", &test_file, "--udid", &udid]);
    let python_output = crate::common::run_idb_file_command(&["rm", &test_dir, "--udid", &udid]);

    // Create directory with file and remove with agent-mobile
    let _mkdir = crate::common::run_idb_file_command(&["mkdir", &test_dir, "--udid", &udid]);
    let _create =
        crate::common::run_idb_file_command(&["push", "/dev/null", &test_file, "--udid", &udid]);
    let rust_output =
        crate::common::run_agent_mobile_file_command(&["rm", &test_dir, "--udid", &udid]);

    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
fn test_rm_nonexistent_file_error() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    let nonexistent_file = "/tmp/this_file_does_not_exist_12345";

    // Both should fail
    let python_output =
        crate::common::run_idb_file_command(&["rm", nonexistent_file, "--udid", &udid]);
    let rust_output =
        crate::common::run_agent_mobile_file_command(&["rm", nonexistent_file, "--udid", &udid]);

    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());

    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
fn test_rm_with_bundle_id_equivalence() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);
    let bundle_id = crate::common::get_test_bundle_id();

    let pid = std::process::id();
    let test_file = format!("/tmp/test_rm_bundle_{}", pid);

    // Create test file in app container and remove with Python idb
    let _create = crate::common::run_idb_file_command(&[
        "push",
        "/dev/null",
        &test_file,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);
    let python_output = crate::common::run_idb_file_command(&[
        "rm",
        &test_file,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    // Create test file in app container and remove with agent-mobile
    let _create = crate::common::run_idb_file_command(&[
        "push",
        "/dev/null",
        &test_file,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);
    let rust_output = crate::common::run_agent_mobile_file_command(&[
        "rm",
        &test_file,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
fn test_rm_empty_directory_equivalence() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let test_dir = format!("/tmp/test_rm_empty_dir_{}", pid);

    // Create empty directory and remove with Python idb
    let _mkdir = crate::common::run_idb_file_command(&["mkdir", &test_dir, "--udid", &udid]);
    let python_output = crate::common::run_idb_file_command(&["rm", &test_dir, "--udid", &udid]);

    // Create empty directory and remove with agent-mobile
    let _mkdir = crate::common::run_idb_file_command(&["mkdir", &test_dir, "--udid", &udid]);
    let rust_output =
        crate::common::run_agent_mobile_file_command(&["rm", &test_dir, "--udid", &udid]);

    crate::common::compare_file_command_outputs(&python_output, &rust_output);
}
