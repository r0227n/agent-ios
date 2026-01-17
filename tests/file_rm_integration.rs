mod common;

use std::process::Command;

/// Integration tests for file rm command
/// Compares Python idb output with agent-mobile output to ensure compatibility

#[test]
#[ignore] // Run with: cargo test --test file_rm_integration -- --ignored
fn test_rm_single_file_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let test_file = format!("/tmp/test_rm_single_{}", pid);

    // Create test file and remove with Python idb
    let _create = common::run_idb_file_command(&["push", "/dev/null", &test_file, "--udid", &udid]);
    let python_output = common::run_idb_file_command(&["rm", &test_file, "--udid", &udid]);

    // Create test file and remove with agent-mobile
    let _create = common::run_idb_file_command(&["push", "/dev/null", &test_file, "--udid", &udid]);
    let rust_output = common::run_agent_mobile_file_command(&["rm", &test_file, "--udid", &udid]);

    common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
#[ignore]
fn test_rm_multiple_files_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let file1 = format!("/tmp/test_rm_multi1_{}", pid);
    let file2 = format!("/tmp/test_rm_multi2_{}", pid);

    // Create test files and remove with Python idb
    let _create1 = common::run_idb_file_command(&["push", "/dev/null", &file1, "--udid", &udid]);
    let _create2 = common::run_idb_file_command(&["push", "/dev/null", &file2, "--udid", &udid]);
    let python_output = common::run_idb_file_command(&["rm", &file1, &file2, "--udid", &udid]);

    // Create test files and remove with agent-mobile
    let _create1 = common::run_idb_file_command(&["push", "/dev/null", &file1, "--udid", &udid]);
    let _create2 = common::run_idb_file_command(&["push", "/dev/null", &file2, "--udid", &udid]);
    let rust_output =
        common::run_agent_mobile_file_command(&["rm", &file1, &file2, "--udid", &udid]);

    common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
#[ignore]
fn test_rm_directory_recursive_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let test_dir = format!("/tmp/test_rm_dir_{}", pid);
    let test_file = format!("{}/test_file", test_dir);

    // Create directory with file and remove with Python idb
    let _mkdir = common::run_idb_file_command(&["mkdir", &test_dir, "--udid", &udid]);
    let _create = common::run_idb_file_command(&["push", "/dev/null", &test_file, "--udid", &udid]);
    let python_output = common::run_idb_file_command(&["rm", &test_dir, "--udid", &udid]);

    // Create directory with file and remove with agent-mobile
    let _mkdir = common::run_idb_file_command(&["mkdir", &test_dir, "--udid", &udid]);
    let _create = common::run_idb_file_command(&["push", "/dev/null", &test_file, "--udid", &udid]);
    let rust_output = common::run_agent_mobile_file_command(&["rm", &test_dir, "--udid", &udid]);

    common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
#[ignore]
fn test_rm_nonexistent_file_error() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_file = "/tmp/this_file_does_not_exist_12345";

    // Both should fail
    let python_output = common::run_idb_file_command(&["rm", nonexistent_file, "--udid", &udid]);
    let rust_output =
        common::run_agent_mobile_file_command(&["rm", nonexistent_file, "--udid", &udid]);

    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());

    common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
#[ignore]
fn test_rm_with_bundle_id_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);
    let bundle_id = common::get_test_bundle_id();

    let pid = std::process::id();
    let test_file = format!("/tmp/test_rm_bundle_{}", pid);

    // Create test file in app container and remove with Python idb
    let _create = common::run_idb_file_command(&[
        "push",
        "/dev/null",
        &test_file,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);
    let python_output = common::run_idb_file_command(&[
        "rm",
        &test_file,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    // Create test file in app container and remove with agent-mobile
    let _create = common::run_idb_file_command(&[
        "push",
        "/dev/null",
        &test_file,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);
    let rust_output = common::run_agent_mobile_file_command(&[
        "rm",
        &test_file,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
#[ignore]
fn test_rm_empty_directory_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let test_dir = format!("/tmp/test_rm_empty_dir_{}", pid);

    // Create empty directory and remove with Python idb
    let _mkdir = common::run_idb_file_command(&["mkdir", &test_dir, "--udid", &udid]);
    let python_output = common::run_idb_file_command(&["rm", &test_dir, "--udid", &udid]);

    // Create empty directory and remove with agent-mobile
    let _mkdir = common::run_idb_file_command(&["mkdir", &test_dir, "--udid", &udid]);
    let rust_output = common::run_agent_mobile_file_command(&["rm", &test_dir, "--udid", &udid]);

    common::compare_file_command_outputs(&python_output, &rust_output);
}
